#!/usr/bin/env bash
#
# Validation rules for release plan files (REL-003).
#
# Release narratives live under a `releases/` directory as `v<version>.md`.
# Unlike design documents these rules are ERRORS, not warnings: a malformed
# release plan must fail CI rather than whisper. Hand-port of `lint_release`
# in cli/src/lint.rs — same codes, order, severities and messages (D-039).
#

# True when a basename is `v<digit>...md` — `v0.3.0.md`, `v1.2.0-beta.md`.
# Mirrors is_release_filename() in cli/src/lint.rs: a literal `v` followed by
# an ASCII digit, and a `.md` extension. Case-sensitive, like the Rust check.
is_release_filename() {
  local basename="$1"
  [[ "$basename" == v[0-9]*.md ]]
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
# The header table drives the release — Target (which version) and Status
# (where it is in the lifecycle). Both rows must appear in the first 20 lines.
check_r002_header_table() {
  local file="$1"
  if ! ( head -20 "$file" | grep -qE '^\| *Target *\|' && \
         head -20 "$file" | grep -qE '^\| *Status *\|' ); then
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
