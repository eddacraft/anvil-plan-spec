#!/usr/bin/env bash
#
# Validation rules for index template
#

# E004: Missing ## Modules section
check_e004_modules() {
  local file="$1"
  if ! has_section "$file" "## Modules"; then
    add_result "$file" "error" "E004" "Missing ## Modules section"
    return 1
  fi
  return 0
}

# W019: Link in ## Modules points to a non-existent file
#
# Warning (not error) because the scaffold seed index intentionally links a
# placeholder module that the user creates later. `aps audit` reports the
# same condition as finding A004 with a non-zero exit for hard gating.
check_w019_module_links() {
  local file="$1"
  local dir
  dir=$(dirname "$file")

  while IFS=: read -r line_num target; do
    [[ -z "$target" ]] && continue
    # Skip external links and pure anchors
    case "$target" in
      http://*|https://*|mailto:*|\#*) continue ;;
    esac
    # Strip anchor fragment
    target="${target%%#*}"
    [[ -z "$target" ]] && continue
    if [[ ! -e "$dir/$target" ]]; then
      add_result "$file" "warning" "W019" "Module link target not found: $target" "$line_num"
    fi
  done < <(awk '
    /^## Modules/ { in_mod=1; next }
    in_mod && /^## /  { in_mod=0 }
    in_mod {
      line = $0
      while (match(line, /\]\([^)]+\)/)) {
        print NR ":" substr(line, RSTART+2, RLENGTH-3)
        line = substr(line, RSTART+RLENGTH)
      }
    }
  ' "$file")

  return 0
}

# W004: Empty section check (for index-specific sections)
check_w004_empty_sections_index() {
  local file="$1"
  local sections=("## Overview" "## Problem & Success Criteria" "## Modules")

  for section in "${sections[@]}"; do
    if has_section "$file" "$section" && ! section_has_content "$file" "$section"; then
      local line
      line=$(get_line_number "$file" "^${section}$")
      add_result "$file" "warning" "W004" "Empty section: $section" "$line"
    fi
  done
}

# Run all index rules
lint_index() {
  local file="$1"
  local has_errors=false

  check_e004_modules "$file" || has_errors=true
  check_w019_module_links "$file"
  check_w004_empty_sections_index "$file"

  $has_errors && return 1
  return 0
}
