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
  check_w017_last_reviewed "$file"

  # Check work items if the section exists
  if has_section "$file" "## Work Items"; then
    lint_work_items "$file" || has_errors=true
  fi

  $has_errors && return 1
  return 0
}
