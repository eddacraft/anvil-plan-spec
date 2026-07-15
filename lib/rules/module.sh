#!/usr/bin/env bash
#
# Validation rules for module and simple templates
#

# E001: Missing ## Purpose section
check_e001_purpose() {
  local file="$1"
  if ! has_section "$file" "## Purpose"; then
    add_result "$file" "error" "E001" "Missing ## Purpose section"
    return 1
  fi
  return 0
}

# E002: Missing ## Work Items section
check_e002_work_items() {
  local file="$1"
  if ! has_section "$file" "## Work Items"; then
    add_result "$file" "error" "E002" "Missing ## Work Items section"
    return 1
  fi
  return 0
}

# E003: Missing ID/Status metadata table
check_e003_metadata() {
  local file="$1"
  if ! has_metadata_table "$file"; then
    add_result "$file" "error" "E003" "Missing ID/Status metadata table"
    return 1
  fi
  return 0
}

# W004: Empty section check (for module-specific sections)
check_w004_empty_sections_module() {
  local file="$1"
  local sections=("## Purpose" "## In Scope")

  for section in "${sections[@]}"; do
    if has_section "$file" "$section" && ! section_has_content "$file" "$section"; then
      local line
      line=$(get_line_number "$file" "^${section}$")
      add_result "$file" "warning" "W004" "Empty section: $section" "$line"
    fi
  done
}

# W017: Active module missing or stale Last reviewed: field
#
# Modules that are Ready or In Progress should carry
# `**Last reviewed:** YYYY-MM-DD` near the top so staleness is detectable.
# Threshold configurable via APS_STALE_DAYS (default 60).
check_w017_last_reviewed() {
  local file="$1"
  local status
  status=$(get_status "$file")

  # Only active modules are required to be fresh
  echo "$status" | grep -qiE '^(ready|in progress)' || return 0

  local reviewed
  reviewed=$(grep -m1 -oE '^\*\*Last reviewed:\*\* *[0-9]{4}-[0-9]{2}-[0-9]{2}' "$file" \
    | grep -oE '[0-9]{4}-[0-9]{2}-[0-9]{2}' || true)

  if [[ -z "$reviewed" ]]; then
    add_result "$file" "warning" "W017" "Active module has no **Last reviewed:** field"
    return 0
  fi

  local stale_days="${APS_STALE_DAYS:-60}"
  [[ "$stale_days" =~ ^[0-9]+$ ]] || stale_days=60
  local reviewed_epoch now_epoch
  # GNU date first, BSD date fallback
  reviewed_epoch=$(date -d "$reviewed" +%s 2>/dev/null \
    || date -j -f "%Y-%m-%d" "$reviewed" +%s 2>/dev/null) || return 0
  now_epoch=$(date +%s)

  local age_days=$(( (now_epoch - reviewed_epoch) / 86400 ))
  if (( age_days > stale_days )); then
    local line
    line=$(get_line_number "$file" '^\*\*Last reviewed:\*\*')
    add_result "$file" "warning" "W017" "Last reviewed $reviewed is ${age_days} days old (threshold: ${stale_days})" "$line"
  fi
}

# W002: a conductor module's coordination sections reference a work-item ID
# that resolves nowhere in the plan tree — most likely a typo. Conductor
# modules legitimately reference IDs owned by other modules (that is the point),
# so cross-file references are expected; only unresolved ones are flagged.
# Vertical-module dependency typos remain W003's job, so this only runs for
# `Type: Conductor` modules. Mirrors the Rust check_w002_conductor_refs.
check_w002_conductor_refs() {
  local file="$1"
  is_conductor "$file" || return 0

  local section
  for section in "## Coordinated Modules" "## Cross-Module Work Items"; do
    has_section "$file" "$section" || continue
    # Walk the section body with absolute line numbers, skipping HTML comments.
    local line_num content id
    while IFS=: read -r line_num content; do
      for id in $(echo "$content" | grep -oE '[A-Z]+-[0-9]{3}'); do
        if ! echo " ${APS_TREE_IDS:-} " | grep -qw "$id"; then
          add_result "$file" "warning" "W002" "Cross-module reference '$id' not found in plan tree" "$line_num"
        fi
      done
    done < <(awk -v sect="$section" '
      $0 == sect { found = 1; next }
      found && /^## / { exit }
      found {
        if (incomment) { if ($0 ~ /-->/) incomment = 0; next }
        t = $0; sub(/^[[:space:]]+/, "", t)
        if (t ~ /^<!--/) { if (t !~ /-->/) incomment = 1; next }
        print NR ":" $0
      }
    ' "$file")
  done
  return 0
}

# W022: a `Packages:` scope tag (metadata-table column or work-item field)
# that resolves to no directory in the workspace — most likely a typo
# (PKG-002, the tagged-monorepo analogue of W002/W006). Resolution tries the
# entry as given plus the conventional packages/<entry> and apps/<entry>
# roots. Silent when the workspace has no packages/ or apps/ directory, so
# single-package projects never pay for the check. Mirrors the Rust
# check_w022_packages.
check_w022_packages() {
  local file="$1"

  # Workspace root: the path prefix before the last /plans/ component.
  local root
  case "$file" in
    */plans/*) root="${file%/plans/*}" ;;
    plans/*)   root="." ;;
    *)         return 0 ;;
  esac
  [[ -d "$root/packages" || -d "$root/apps" ]] || return 0

  local line_num value raw entry
  while IFS=: read -r line_num value; do
    [[ -z "$value" ]] && continue
    local entries
    IFS=',' read -ra entries <<< "$value"
    for raw in "${entries[@]}"; do
      # Trim whitespace and backticks; skip prose placeholders like
      # `_(monorepo only)_` (anything outside [A-Za-z0-9@._/-]).
      # shellcheck disable=SC2016 # the backtick is a literal in the sed class
      entry=$(printf '%s' "$raw" | sed -E 's/^[[:space:]`]+//; s/[[:space:]`]+$//')
      [[ -n "$entry" ]] || continue
      [[ "$entry" =~ ^[A-Za-z0-9@._/-]+$ ]] || continue
      if [[ ! -d "$root/$entry" && ! -d "$root/packages/$entry" && ! -d "$root/apps/$entry" ]]; then
        add_result "$file" "warning" "W022" "Packages entry '$entry' does not resolve to a workspace directory" "$line_num"
      fi
    done
  done < <(awk -F'|' '
    # Metadata table: first data row of the Packages column is authoritative
    # (mirrors get_module_type). Repeated headers and separators are skipped.
    !hdr && /^\|/ {
      c1 = $2; gsub(/^[[:space:]]+|[[:space:]]+$/, "", c1)
      if (c1 == "ID") {
        for (i = 1; i <= NF; i++) {
          c = $i; gsub(/^[[:space:]]+|[[:space:]]+$/, "", c)
          if (c == "Packages") pc = i
        }
        hdr = 1
      }
      next
    }
    hdr && !done && /^\|/ {
      if ($0 ~ /^[|: -]+$/) next
      c1 = $2; gsub(/^[[:space:]]+|[[:space:]]+$/, "", c1)
      if (c1 == "ID") next
      if (pc) {
        v = $pc; gsub(/^[[:space:]]+|[[:space:]]+$/, "", v)
        if (v != "") print NR ":" v
      }
      done = 1
      next
    }
    # Work-item fields: every `- **Packages:** ...` line.
    /^[[:space:]]*- \*\*Packages:\*\*/ {
      line = $0
      sub(/^[[:space:]]*- \*\*Packages:\*\*[[:space:]]*/, "", line)
      if (line != "") print NR ":" line
    }
  ' "$file")
  return 0
}

# W005: Status=Ready but no work items
check_w005_ready_no_items() {
  local file="$1"
  local status
  status=$(get_status "$file")

  if [[ "$status" == "Ready" ]]; then
    local items
    items=$(get_work_items "$file")
    if [[ -z "$items" ]]; then
      add_result "$file" "warning" "W005" "Status is Ready but no work items defined"
    fi
  fi
}

# Run all module/simple rules
lint_module() {
  local file="$1"
  local has_errors=false

  check_e001_purpose "$file" || has_errors=true
  check_e002_work_items "$file" || has_errors=true
  check_e003_metadata "$file" || has_errors=true

  check_w004_empty_sections_module "$file"
  check_w005_ready_no_items "$file"
  # W017 then W002 then W022 — mirror the Rust lint_module call order so
  # byte-level diffs of lint output stay identical across all three CLIs
  # (D-038/D-039).
  check_w017_last_reviewed "$file"
  check_w002_conductor_refs "$file"
  check_w022_packages "$file"

  # Check work items if the section exists
  if has_section "$file" "## Work Items"; then
    lint_work_items "$file" || has_errors=true
  fi

  $has_errors && return 1
  return 0
}
