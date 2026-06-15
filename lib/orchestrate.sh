#!/usr/bin/env bash
#
# APS orchestration commands
#

declare -a ORCH_ITEM_IDS=()
declare -a ORCH_ITEM_TITLES=()
declare -a ORCH_ITEM_STATUSES=()
declare -a ORCH_ITEM_DEPS=()
declare -a ORCH_ITEM_LINES=()
declare -a ORCH_ITEM_MODULES=()
declare -a ORCH_ITEM_FILES=()
declare -A ORCH_MODULE_STATUSES=()

orch_reset_state() {
  ORCH_ITEM_IDS=()
  ORCH_ITEM_TITLES=()
  ORCH_ITEM_STATUSES=()
  ORCH_ITEM_DEPS=()
  ORCH_ITEM_LINES=()
  ORCH_ITEM_MODULES=()
  ORCH_ITEM_FILES=()
  ORCH_MODULE_STATUSES=()
}

orch_trim() {
  local value="$1"
  value="${value#"${value%%[![:space:]]*}"}"
  value="${value%"${value##*[![:space:]]}"}"
  printf '%s' "$value"
}

orch_field_value() {
  local content="$1"
  local field="$2"

  printf '%s\n' "$content" | awk -v field="$field" '
    $0 ~ "^- \\*\\*" field ":\\*\\*" {
      sub("^- \\*\\*" field ":\\*\\*[[:space:]]*", "")
      if ($0 != "") print
      found = 1
      next
    }
    found && /^[[:space:]]+[^[:space:]]/ {
      gsub(/^[[:space:]]+/, "")
      sub(/^- /, "")
      print
      next
    }
    found { exit }
  '
}

orch_item_content() {
  local file="$1"
  local start_line="$2"

  awk -v start="$start_line" '
    NR == start { found=1; next }
    found && /^## / { exit }
    found && /^### / { exit }
    found { print }
  ' "$file"
}

orch_normalize_status() {
  local raw="$1"
  local fallback="${2:-Ready}"

  [[ -z "$raw" ]] && echo "$fallback" && return
  raw=$(printf '%s' "$raw" | sed -E 's/^[^A-Za-z]+//')

  case "$raw" in
    Complete*) echo "Complete" ;;
    "In Progress"*) echo "In Progress" ;;
    Ready*) echo "Ready" ;;
    Draft*) echo "Draft" ;;
    Blocked*) echo "Blocked" ;;
    *) echo "Unknown" ;;
  esac
}

orch_item_matches_module() {
  local item_index="$1"
  local filter="$2"

  [[ -z "$filter" ]] && return 0

  local file="${ORCH_ITEM_FILES[$item_index]}"
  local module_id="${ORCH_ITEM_MODULES[$item_index]}"
  local base
  base=$(basename "$file" .aps.md)

  [[ "${module_id,,}" == "${filter,,}" || "${base,,}" == "${filter,,}" ]]
}

orch_load_index_modules() {
  local plan_root="$1"
  local index_file="$plan_root/index.aps.md"

  [[ -f "$index_file" ]] || return 0

  while IFS='|' read -r module status; do
    ORCH_MODULE_STATUSES["$module"]="$status"
  done < <(awk -F '|' '
    /^\| *\[/ {
      module = $2
      status = $4
      gsub(/.*\[/, "", module)
      gsub(/\].*/, "", module)
      gsub(/^[[:space:]]+|[[:space:]]+$/, "", status)
      if (module != "" && status != "") print toupper(module) "|" status
    }
  ' "$index_file")
}

orch_load_work_items() {
  local plan_root="$1"
  local load_all="${2:-false}"
  local module_dir="$plan_root/modules"

  [[ -d "$module_dir" ]] || return 1

  local file
  while IFS= read -r file; do
    local module_id module_status
    module_id=$(get_module_id "$file")
    module_status=$(get_status "$file")
    module_status=$(orch_normalize_status "$module_status" "Draft")

    [[ -n "$module_id" ]] || module_id=$(basename "$file" .aps.md | tr '[:lower:]' '[:upper:]')
    ORCH_MODULE_STATUSES["$module_id"]="$module_status"

    if [[ "$load_all" != "true" ]]; then
      [[ "$module_status" == "Complete" || "$module_status" == "Draft" || "$module_status" == "Blocked" ]] && continue
    fi

    while IFS=: read -r line_num header; do
      [[ -n "$header" ]] || continue

      local id title content status deps
      header=$(orch_trim "$header")
      id=$(printf '%s\n' "$header" | sed -E 's/^### ([A-Za-z]+-[0-9]+):.*/\1/')
      title=$(printf '%s\n' "$header" | sed -E 's/^### [A-Za-z]+-[0-9]+:[[:space:]]*//; s/[[:space:]]+[^[:alnum:][:space:]]+[[:space:]]+Complete.*$//')
      content=$(orch_item_content "$file" "$line_num")
      status=$(orch_field_value "$content" "Status")

      if [[ -z "$status" && "$header" == *"Complete"* ]]; then
        status="Complete"
      fi

      status=$(orch_normalize_status "$status" "Ready")
      deps=$(orch_field_value "$content" "Dependencies")

      ORCH_ITEM_IDS+=("$id")
      ORCH_ITEM_TITLES+=("$title")
      ORCH_ITEM_STATUSES+=("$status")
      ORCH_ITEM_DEPS+=("$deps")
      ORCH_ITEM_LINES+=("$line_num")
      ORCH_ITEM_MODULES+=("$module_id")
      ORCH_ITEM_FILES+=("$file")
    done <<< "$(get_work_items "$file")"
  done < <(find "$module_dir" -type f -name "*.aps.md" ! -name ".*" 2>/dev/null | sort)
}

orch_item_index() {
  local id="$1"
  local i

  for i in "${!ORCH_ITEM_IDS[@]}"; do
    [[ "${ORCH_ITEM_IDS[$i]}" == "$id" ]] && echo "$i" && return 0
  done

  return 1
}

orch_dependency_complete() {
  local dep="$1"

  if [[ "$dep" =~ ^[A-Z]+-[0-9]+$ ]]; then
    # Decision dependencies (D-NNN) are resolved in the plan text, not as work items.
    [[ "$dep" == D-* ]] && return 0

    local idx
    idx=$(orch_item_index "$dep" || true)
    [[ -n "$idx" && "${ORCH_ITEM_STATUSES[$idx]}" == "Complete" ]]
    return
  fi

  local module_status="${ORCH_MODULE_STATUSES[$dep]:-}"
  [[ "$module_status" == "Complete" ]]
}

orch_deps_complete() {
  local deps="$1"
  local dep_ids=()
  local dep

  [[ -z "$deps" || "$deps" == "None" || "$deps" == "-" ]] && return 0
  [[ ! "$deps" =~ [[:alnum:]] ]] && return 0

  while IFS= read -r dep; do
    [[ -n "$dep" ]] && dep_ids+=("$dep")
  done < <(printf '%s\n' "$deps" | grep -oE '[A-Z]+-[0-9]+|[A-Z]{2,}' || true)

  [[ ${#dep_ids[@]} -eq 0 ]] && return 1

  for dep in "${dep_ids[@]}"; do
    orch_dependency_complete "$dep" || return 1
  done

  return 0
}

orch_deps_display() {
  local deps="$1"

  deps=${deps//$'\n'/, }
  echo "${deps:-None}"
}

orch_dep_ids() {
  local deps="$1"

  printf '%s\n' "$deps" | grep -oE '[A-Z]+-[0-9]+|[A-Z]{2,}' || true
}

orch_context_root() {
  local plan_root="$1"
  local parent

  parent=$(dirname "$plan_root")
  printf '%s/.aps/context' "$parent"
}

orch_emit_section() {
  local file="$1"
  local section="$2"

  awk -v section="$section" '
    $0 == "## " section { found=1; print; next }
    found && /^## / { exit }
    found { print }
  ' "$file"
}

orch_context_package() {
  local plan_root="$1"
  local idx="$2"
  local id="${ORCH_ITEM_IDS[$idx]}"
  local title="${ORCH_ITEM_TITLES[$idx]}"
  local file="${ORCH_ITEM_FILES[$idx]}"
  local line="${ORCH_ITEM_LINES[$idx]}"
  local deps="${ORCH_ITEM_DEPS[$idx]}"
  local context_dir context_file dep dep_idx related_files

  context_dir=$(orch_context_root "$plan_root")
  mkdir -p "$context_dir" || { error "Cannot create context directory: $context_dir"; return 1; }
  context_file="$context_dir/$id.md"
  related_files=$(orch_field_value "$(orch_item_content "$file" "$line")" "Files")

  {
    echo "# Context: $id - $title"
    echo
    echo "## Work Item"
    orch_item_content "$file" "$line"
    echo
    echo "## Module Scope"
    orch_emit_section "$file" "Purpose"
    echo
    orch_emit_section "$file" "In Scope"
    echo
    orch_emit_section "$file" "Out of Scope"
    echo
    orch_emit_section "$file" "Interfaces"
    echo
    echo "## Decisions"
    orch_emit_section "$file" "Decisions" || true
    echo
    echo "## Dependency Learnings"
    local found_learning="false"
    while IFS= read -r dep; do
      [[ -n "$dep" ]] || continue
      dep_idx=$(orch_item_index "$dep" || true)
      [[ -n "$dep_idx" ]] || continue
      local dep_content dep_learning
      dep_content=$(orch_item_content "${ORCH_ITEM_FILES[$dep_idx]}" "${ORCH_ITEM_LINES[$dep_idx]}")
      dep_learning=$(orch_field_value "$dep_content" "Learning")
      if [[ -n "$dep_learning" ]]; then
        echo "- $dep: $dep_learning"
        found_learning="true"
      fi
    done < <(orch_dep_ids "$deps")
    [[ "$found_learning" == "true" ]] || echo "- None"
    echo
    echo "## Related Files"
    if [[ -n "$related_files" ]]; then
      printf '%s\n' "$related_files" | sed 's/^/- /'
    else
      echo "- None specified"
    fi
  } > "$context_file"

  printf '%s' "$context_file"
}

cmd_next() {
  local plan_root="" strict=false
  local module_filter=""

  while [[ $# -gt 0 ]]; do
    case "$1" in
      --plans)
        plan_root="${2:-}"
        [[ -n "$plan_root" ]] || { error "--plans requires a directory"; return 1; }
        shift 2
        ;;
      --strict)
        strict=true
        shift
        ;;
      --help|-h)
        cat <<EOF
Usage: aps next [module] [options]

Show the next Ready work item whose dependencies are Complete.

Arguments:
  module    Optional module ID or module file name, e.g. AUTH or auth

Options:
  --plans DIR  Plan root directory (default: plans)
  --help       Show this help
EOF
        return 0
        ;;
      -*)
        error "Unknown option: $1"
        return 1
        ;;
      *)
        module_filter="$1"
        shift
        ;;
    esac
  done

  if [[ -z "$plan_root" ]]; then
    plan_root="$(aps_default_plans)"
    aps_check_cli_version "$strict"
  fi

  if [[ ! -d "$plan_root" ]]; then
    error "Path not found: $plan_root"
    return 1
  fi

  orch_reset_state
  orch_load_index_modules "$plan_root"
  orch_load_work_items "$plan_root" "true" || {
    error "No modules directory found: $plan_root/modules"
    return 1
  }

  local i
  for i in "${!ORCH_ITEM_IDS[@]}"; do
    orch_item_matches_module "$i" "$module_filter" || continue
    case "${ORCH_MODULE_STATUSES[${ORCH_ITEM_MODULES[$i]}]:-Unknown}" in
      Ready|"In Progress") ;;
      *) continue ;;
    esac
    [[ "${ORCH_ITEM_STATUSES[$i]}" == "Ready" ]] || continue
    orch_deps_complete "${ORCH_ITEM_DEPS[$i]}" || continue

    echo "${ORCH_ITEM_IDS[$i]}: ${ORCH_ITEM_TITLES[$i]}"
    echo "Module: ${ORCH_ITEM_MODULES[$i]} | Dependencies: $(orch_deps_display "${ORCH_ITEM_DEPS[$i]}") | Status: ${ORCH_ITEM_STATUSES[$i]}"
    echo "File: ${ORCH_ITEM_FILES[$i]}"
    return 0
  done

  if [[ -n "$module_filter" ]]; then
    warn "No ready work item found for module: $module_filter"
  else
    warn "No ready work item found"
  fi
  return 1
}

orch_today() {
  date -u +%Y-%m-%d
}

orch_rewrite_work_item() {
  local file="$1"
  local id="$2"
  local mode="$3"   # "status" or "learning"
  local value="$4"

  [[ -f "$file" ]] || { error "Cannot rewrite: file not found: $file"; return 1; }

  local tmp
  tmp=$(mktemp) || { error "Cannot create temp file"; return 1; }

  awk -v target="$id" -v mode="$mode" -v value="$value" '
    function emit_buffer(   i) {
      for (i = 0; i < bcount; i++) print buffer[i]
      bcount = 0
    }

    function meta_line(idx) {
      return buffer[idx] ~ /^- \*\*[A-Za-z][^*]*:\*\*/
    }

    function continuation_line(idx) {
      return buffer[idx] ~ /^[[:space:]]+[^[:space:]]/
    }

    function flush_target(   i, status_line, learning_line, status_idx, last_meta, validation_idx, insert_idx) {
      status_idx = -1
      validation_idx = -1
      last_meta = -1
      for (i = 0; i < bcount; i++) {
        if (buffer[i] ~ /^- \*\*Status:\*\*/) status_idx = i
        if (buffer[i] ~ /^- \*\*Validation:\*\*/) validation_idx = i
        if (meta_line(i)) last_meta = i
        else if (continuation_line(i) && last_meta >= 0) last_meta = i
      }

      if (mode == "status") {
        status_line = "- **Status:** " value
        if (status_idx >= 0) {
          buffer[status_idx] = status_line
          emit_buffer()
          return
        }
        if (last_meta < 0) last_meta = bcount - 1
        for (i = 0; i <= last_meta; i++) print buffer[i]
        print status_line
        for (i = last_meta + 1; i < bcount; i++) print buffer[i]
        bcount = 0
        return
      }

      if (mode == "learning") {
        learning_line = "- **Learning:** \"" value "\""
        if (validation_idx >= 0) {
          insert_idx = validation_idx
          # advance past any multi-line continuation under Validation
          while (insert_idx + 1 < bcount && continuation_line(insert_idx + 1)) insert_idx++
        } else if (last_meta >= 0) {
          insert_idx = last_meta
        } else {
          insert_idx = bcount - 1
        }
        for (i = 0; i <= insert_idx; i++) print buffer[i]
        print learning_line
        for (i = insert_idx + 1; i < bcount; i++) print buffer[i]
        bcount = 0
        return
      }

      emit_buffer()
    }

    /^### / {
      if (state == "in") flush_target()
      if ($0 ~ "^### " target ":") {
        state = "in"
        bcount = 0
        buffer[bcount++] = $0
        next
      }
      state = "out"
      print
      next
    }

    /^## / && state == "in" {
      flush_target()
      state = "out"
      print
      next
    }

    state == "in" {
      buffer[bcount++] = $0
      next
    }

    { print }

    END {
      if (state == "in") flush_target()
    }
  ' "$file" > "$tmp" || { rm -f "$tmp"; error "Rewrite failed for $id in $file"; return 1; }

  mv "$tmp" "$file"
}

orch_rewrite_status() {
  orch_rewrite_work_item "$1" "$2" "status" "$3"
}

orch_append_learning() {
  orch_rewrite_work_item "$1" "$2" "learning" "$3"
}

cmd_start() {
  local plan_root="" strict=false
  local id=""

  while [[ $# -gt 0 ]]; do
    case "$1" in
      --plans)
        plan_root="${2:-}"
        [[ -n "$plan_root" ]] || { error "--plans requires a directory"; return 1; }
        shift 2
        ;;
      --strict)
        strict=true
        shift
        ;;
      --help|-h)
        cat <<EOF
Usage: aps start <ID> [options]

Mark a Ready work item as In Progress in its .aps.md file.

Arguments:
  ID    Work item ID, e.g. AUTH-003

Options:
  --plans DIR  Plan root directory (default: plans)
  --help       Show this help

Validates that the item is Ready and its dependencies are Complete before
mutating the markdown. Suggests a branch name (work/<id>) - branch creation
is left to the user per ORCH D-003.
EOF
        return 0
        ;;
      -*)
        error "Unknown option: $1"
        return 1
        ;;
      *)
        [[ -z "$id" ]] || { error "Unexpected argument: $1"; return 1; }
        id="$1"
        shift
        ;;
    esac
  done

  [[ -n "$id" ]] || { error "Usage: aps start <ID>"; return 1; }

  if [[ -z "$plan_root" ]]; then
    plan_root="$(aps_default_plans)"
    aps_check_cli_version "$strict"
  fi

  if [[ ! -d "$plan_root" ]]; then
    error "Path not found: $plan_root"
    return 1
  fi

  orch_reset_state
  orch_load_index_modules "$plan_root"
  orch_load_work_items "$plan_root" "true" || {
    error "No modules directory found: $plan_root/modules"
    return 1
  }

  local idx
  idx=$(orch_item_index "$id" || true)
  [[ -n "$idx" ]] || { error "Work item not found: $id"; return 1; }

  local current="${ORCH_ITEM_STATUSES[$idx]}"
  local file="${ORCH_ITEM_FILES[$idx]}"
  local module_id="${ORCH_ITEM_MODULES[$idx]}"
  local module_status="${ORCH_MODULE_STATUSES[$module_id]:-Unknown}"
  local deps="${ORCH_ITEM_DEPS[$idx]}"
  local already_started="false"

  case "$module_status" in
    Ready|"In Progress") ;;
    *)
      error "$id belongs to module $module_id (status: $module_status) - module must be Ready or In Progress to start work items"
      return 1
      ;;
  esac

  case "$current" in
    Ready) ;;
    "In Progress")
      already_started="true"
      ;;
    Complete)
      error "$id is already Complete - cannot restart"
      return 1
      ;;
    *)
      error "$id has status '$current' - cannot start (must be Ready)"
      return 1
      ;;
  esac

  if ! orch_deps_complete "$deps"; then
    error "$id has unmet dependencies: $(orch_deps_display "$deps")"
    return 1
  fi

  if [[ "$already_started" != "true" ]]; then
    orch_rewrite_status "$file" "$id" "In Progress" || return 1
    ORCH_ITEM_STATUSES[$idx]="In Progress"
  fi

  local context_file
  context_file=$(orch_context_package "$plan_root" "$idx") || return 1

  local lower_id="${id,,}"
  if [[ "$already_started" == "true" ]]; then
    warn "$id is already In Progress (no status change)"
  else
    echo "Marked $id as In Progress"
  fi
  echo "Suggested branch: work/$lower_id"
  echo "File: $file"
  echo "Context package: $context_file"
}

cmd_complete() {
  local plan_root="" strict=false
  local id=""
  local learning=""

  while [[ $# -gt 0 ]]; do
    case "$1" in
      --plans)
        plan_root="${2:-}"
        [[ -n "$plan_root" ]] || { error "--plans requires a directory"; return 1; }
        shift 2
        ;;
      --strict)
        strict=true
        shift
        ;;
      --learning)
        learning="${2:-}"
        [[ -n "$learning" ]] || { error "--learning requires a value"; return 1; }
        shift 2
        ;;
      --help|-h)
        cat <<EOF
Usage: aps complete <ID> [options]

Mark an In Progress work item as Complete in its .aps.md file.

Arguments:
  ID    Work item ID, e.g. AUTH-003

Options:
  --plans DIR        Plan root directory (default: plans)
  --learning "..."   Append a learning line after Validation (ORCH D-002)
  --help             Show this help

Validates that the item is In Progress before mutating the markdown.
Stamps Status as "Complete: YYYY-MM-DD" using today's UTC date.
EOF
        return 0
        ;;
      -*)
        error "Unknown option: $1"
        return 1
        ;;
      *)
        [[ -z "$id" ]] || { error "Unexpected argument: $1"; return 1; }
        id="$1"
        shift
        ;;
    esac
  done

  [[ -n "$id" ]] || { error "Usage: aps complete <ID>"; return 1; }

  if [[ -z "$plan_root" ]]; then
    plan_root="$(aps_default_plans)"
    aps_check_cli_version "$strict"
  fi

  if [[ ! -d "$plan_root" ]]; then
    error "Path not found: $plan_root"
    return 1
  fi

  orch_reset_state
  orch_load_index_modules "$plan_root"
  orch_load_work_items "$plan_root" "true" || {
    error "No modules directory found: $plan_root/modules"
    return 1
  }

  local idx
  idx=$(orch_item_index "$id" || true)
  [[ -n "$idx" ]] || { error "Work item not found: $id"; return 1; }

  local current="${ORCH_ITEM_STATUSES[$idx]}"
  local file="${ORCH_ITEM_FILES[$idx]}"

  case "$current" in
    "In Progress") ;;
    Complete)
      warn "$id is already Complete (no change)"
      return 0
      ;;
    *)
      error "$id has status '$current' - cannot complete (must be In Progress)"
      return 1
      ;;
  esac

  if [[ -z "$learning" && -t 0 ]]; then
    read -r -p "Learning (optional): " learning
  fi

  local today
  today=$(orch_today)
  orch_rewrite_status "$file" "$id" "Complete: $today" || return 1

  if [[ -n "$learning" ]]; then
    orch_append_learning "$file" "$id" "$learning" || return 1
  fi

  echo "Marked $id as Complete: $today"
  [[ -n "$learning" ]] && echo "Learning recorded for $id"
  echo "File: $file"
}

cmd_graph() {
  local plan_root="" strict=false
  local module_filter=""

  while [[ $# -gt 0 ]]; do
    case "$1" in
      --plans)
        plan_root="${2:-}"
        [[ -n "$plan_root" ]] || { error "--plans requires a directory"; return 1; }
        shift 2
        ;;
      --strict)
        strict=true
        shift
        ;;
      --help|-h)
        cat <<EOF
Usage: aps graph [module] [options]

Show work items and dependency arrows.

Arguments:
  module    Optional module ID or module file name, e.g. AUTH or auth

Options:
  --plans DIR  Plan root directory (default: plans)
  --help       Show this help
EOF
        return 0
        ;;
      -*)
        error "Unknown option: $1"
        return 1
        ;;
      *)
        [[ -z "$module_filter" ]] || { error "Unexpected argument: $1"; return 1; }
        module_filter="$1"
        shift
        ;;
    esac
  done

  if [[ -z "$plan_root" ]]; then
    plan_root="$(aps_default_plans)"
    aps_check_cli_version "$strict"
  fi

  if [[ ! -d "$plan_root" ]]; then
    error "Path not found: $plan_root"
    return 1
  fi

  orch_reset_state
  orch_load_index_modules "$plan_root"
  orch_load_work_items "$plan_root" "true" || {
    error "No modules directory found: $plan_root/modules"
    return 1
  }

  local i dep dep_idx shown="false" deps_display
  for i in "${!ORCH_ITEM_IDS[@]}"; do
    orch_item_matches_module "$i" "$module_filter" || continue
    shown="true"
    echo "${ORCH_ITEM_IDS[$i]} [${ORCH_ITEM_STATUSES[$i]}] ${ORCH_ITEM_TITLES[$i]}"

    deps_display=""
    while IFS= read -r dep; do
      [[ -n "$dep" ]] || continue
      dep_idx=$(orch_item_index "$dep" || true)
      if [[ -n "$dep_idx" ]]; then
        deps_display+=" ${ORCH_ITEM_IDS[$dep_idx]}[${ORCH_ITEM_STATUSES[$dep_idx]}]"
      else
        deps_display+=" $dep[${ORCH_MODULE_STATUSES[$dep]:-Unknown}]"
      fi
    done < <(orch_dep_ids "${ORCH_ITEM_DEPS[$i]}")

    if [[ -n "$deps_display" ]]; then
      echo "  <-${deps_display}"
    else
      echo "  <- none"
    fi
  done

  if [[ "$shown" != "true" ]]; then
    if [[ -n "$module_filter" ]]; then
      warn "No work items found for module: $module_filter"
    else
      warn "No work items found"
    fi
    return 1
  fi
}
