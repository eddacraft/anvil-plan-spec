#!/usr/bin/env bash
#
# Validation rules for work items
#

# E005: Missing required work item fields (Intent, Expected Outcome, Validation)
check_e005_required_fields() {
  local file="$1"
  local item_header="$2"
  local item_line="$3"
  local has_errors=false

  # Extract work item content until next ### or ## or EOF
  local content
  content=$(awk -v start="$item_line" '
    NR == start { found=1; next }
    found && /^###? / { exit }
    found { print }
  ' "$file")

  # Terminal (completed) work items are commonly compacted to Status + a short
  # summary once shipped, with their full Intent/Expected Outcome/Validation
  # detail preserved in version history. Exempt them from the required-field
  # checks so closeout compaction does not reopen E005. Active states
  # (Proposed/Ready/In Progress/Blocked/Draft/Deferred) are still checked.
  local status
  status=$(echo "$content" | grep -m1 -E '^\- \*\*Status:\*\*' | sed -E 's/^- \*\*Status:\*\*[[:space:]]*//')
  if echo "$status" | grep -qiE '^(done|complete|merged|released|shipped)\b'; then
    return 0
  fi

  # Check for required fields
  if ! echo "$content" | grep -qE '^\- \*\*Intent:\*\*'; then
    add_result "$file" "error" "E005" "$item_header: Missing **Intent:** field" "$item_line"
    has_errors=true
  fi

  if ! echo "$content" | grep -qE '^\- \*\*Expected Outcome:\*\*'; then
    add_result "$file" "error" "E005" "$item_header: Missing **Expected Outcome:** field" "$item_line"
    has_errors=true
  fi

  if ! echo "$content" | grep -qE '^\- \*\*Validation:\*\*'; then
    add_result "$file" "error" "E005" "$item_header: Missing **Validation:** field" "$item_line"
    has_errors=true
  fi

  $has_errors && return 1
  return 0
}

# W001: Work item ID format check
check_w001_id_format() {
  local file="$1"
  local item_header="$2"
  local item_line="$3"

  # Extract the ID from "### AUTH-001: title"
  local item_id
  item_id=$(echo "$item_header" | sed 's/^### \([A-Za-z0-9-]*\):.*/\1/')

  # Check if it matches the expected pattern [A-Z]+-[0-9]{3}
  if ! echo "$item_id" | grep -qE '^[A-Z]+-[0-9]{3}$'; then
    add_result "$file" "warning" "W001" "Work item ID '$item_id' should match pattern PREFIX-NNN (e.g., AUTH-001)" "$item_line"
  fi
}

# W003: Dependency references unknown task ID
check_w003_dependencies() {
  local file="$1"
  local item_line="$2"
  local all_ids="$3"

  # Extract Dependencies field content
  local deps_line
  deps_line=$(awk -v start="$item_line" '
    NR > start && /^\- \*\*Dependencies:\*\*/ { print; exit }
    NR > start && /^###? / { exit }
  ' "$file")

  if [[ -n "$deps_line" ]]; then
    # Extract task IDs from the line (e.g., "AUTH-001, AUTH-002" or just "AUTH-001")
    local dep_ids
    dep_ids=$(echo "$deps_line" | grep -oE '[A-Z]+-[0-9]{3}' || true)

    for dep_id in $dep_ids; do
      # Resolve in-file first, then against the plan-tree index (cross-module
      # dependencies and decision references are legitimate)
      if ! echo "$all_ids" | grep -qw "$dep_id" \
        && ! echo "${APS_TREE_IDS:-}" | grep -qw "$dep_id"; then
        local line_num
        line_num=$(grep -n "Dependencies:.*$dep_id" "$file" | head -1 | cut -d: -f1)
        add_result "$file" "warning" "W003" "Dependency '$dep_id' not found in plan" "$line_num"
      fi
    done
  fi
}

# W018: Terminal work item missing Validation in an active module
#
# E005 deliberately exempts terminal items from required fields (closeout
# compaction). But a Complete item with no Validation in a module that is
# still active is exactly the "overstated completion" risk DOGFOOD-002
# targets — the audit cannot verify it. Warning only; fully Complete modules
# are archives and stay exempt.
check_w018_terminal_validation() {
  local file="$1"
  local item_header="$2"
  local item_line="$3"
  local module_status="$4"

  # Skip when the whole module is terminal (archive compaction is sanctioned)
  if echo "$module_status" | grep -qiE '^(done|complete|merged|released|shipped|archived)'; then
    return 0
  fi

  # Extract work item content until next ### or ## or EOF
  local content
  content=$(awk -v start="$item_line" '
    NR == start { found=1; next }
    found && /^###? / { exit }
    found { print }
  ' "$file")

  # Terminal status: an explicit Status field is authoritative; the
  # "— Complete <date>" header suffix only counts when no field is present
  # (otherwise a stale header would contradict E005's field-based check)
  local terminal=false
  local status
  status=$(echo "$content" | grep -m1 -E '^\- \*\*Status:\*\*' | sed -E 's/^- \*\*Status:\*\*[[:space:]]*//')
  if [[ -n "$status" ]]; then
    echo "$status" | grep -qiE '^(done|complete|merged|released|shipped)\b' && terminal=true
  elif echo "$item_header" | grep -qiE '(—|--) *(done|complete|merged|released|shipped)\b'; then
    terminal=true
  fi
  [[ "$terminal" == true ]] || return 0

  if ! echo "$content" | grep -qE '^\- \*\*Validation'; then
    add_result "$file" "warning" "W018" "$item_header: Complete item has no Validation — completion cannot be audited" "$item_line"
  fi
}

# Lint all work items in a file
lint_work_items() {
  local file="$1"
  local has_errors=false

  # Collect all work item IDs in the file first (for dependency checking)
  local all_ids
  all_ids=$(grep -oE '^### [A-Z]+-[0-9]+:' "$file" | sed 's/^### \([A-Z]*-[0-9]*\):.*/\1/' | tr '\n' ' ')

  # Module status gates W018 (terminal modules are exempt archives)
  local module_status
  module_status=$(get_status "$file")

  # Process each work item
  while IFS=: read -r line_num header; do
    [[ -z "$header" ]] && continue

    # Clean up the header
    header=$(echo "$header" | sed 's/^[[:space:]]*//')

    check_w001_id_format "$file" "$header" "$line_num"
    check_e005_required_fields "$file" "$header" "$line_num" || has_errors=true
    check_w003_dependencies "$file" "$line_num" "$all_ids"
    check_w018_terminal_validation "$file" "$header" "$line_num" "$module_status"
  done <<< "$(get_work_items "$file")"

  $has_errors && return 1
  return 0
}
