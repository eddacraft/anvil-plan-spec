#!/usr/bin/env bash
#
# Validation rules for release plan files (REL-003).
#
# Release narratives live under a `releases/` directory as `v<version>.md`.
# Unlike design documents these rules are ERRORS, not warnings: a malformed
# release plan must fail CI rather than whisper. Hand-port of `lint_release`
# in cli/src/lint.rs — same codes, order, severities and messages (D-039).
#

# Three numeric components, with optional ASCII prerelease/build identifiers.
# Version shape only; mirrors is_release_filename() in cli/src/lint.rs.
is_release_filename() {
  local basename="$1"
  local LC_ALL=C
  local pattern='^v[0-9]+\.[0-9]+\.[0-9]+(-[0-9A-Za-z-]+(\.[0-9A-Za-z-]+)*)?(\+[0-9A-Za-z-]+(\.[0-9A-Za-z-]+)*)?\.md$'
  [[ "$basename" =~ $pattern ]]
}

# R001: Release file must be named v<version>.md
check_r001_naming() {
  local file="$1"
  local basename
  basename=$(basename "$file")
  if ! is_release_filename "$basename"; then
    add_result "$file" "error" "R001" \
      "Release file must be named v<version>.md (e.g. v0.3.0.md)"
  fi
}

# R002: Missing release header table with Target and Status fields
# Require Target and Status in one table body within the first 20 lines.
check_r002_header_table() {
  local file="$1"
  if ! awk '
    NR > 20 { exit }
    /^(```|~~~)/ { fence = !fence }
    fence || !/^\|/ { previous = table = target = status = 0; next }
    {
      if (previous && /^\| *:?----*:? *(\| *:?----*:? *)+\|? *\r?$/) {
        table = 1; target = status = 0
      } else if (table) {
        if (/^\| *Target *\|/) target = 1
        if (/^\| *Status *\|/) status = 1
        if (target && status) { found = 1; exit }
      }
      previous = 1
    }
    END { exit !found }
  ' "$file"; then
    add_result "$file" "error" "R002" \
      "Missing release header table with Target and Status fields"
  fi
}

# R003: Missing ## Release Theme section
check_r003_release_theme() {
  local file="$1"
  if ! has_section "$file" "## Release Theme"; then
    add_result "$file" "error" "R003" "Missing ## Release Theme section"
  fi
}

# R004: Missing ## What Ships section
check_r004_what_ships() {
  local file="$1"
  if ! has_section "$file" "## What Ships"; then
    add_result "$file" "error" "R004" "Missing ## What Ships section"
  fi
}

# Run all release rules. Emission order is load-bearing for cross-CLI parity:
# R001, R002, R003, R004 — matching lint_release in cli/src/lint.rs.
lint_release() {
  local file="$1"

  check_r001_naming "$file"
  check_r002_header_table "$file"
  check_r003_release_theme "$file"
  check_r004_what_ships "$file"

  return 0
}
