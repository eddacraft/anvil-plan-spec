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
# Path-derived child-plan name each item belongs to (MONO-003). Empty in the
# common single-root case; used to scope and disambiguate across federated trees.
declare -a ORCH_ITEM_CHILDREN=()
# Item-level Packages: scope tags (PKG-001); empty = inherit from the module.
declare -a ORCH_ITEM_PACKAGES=()
declare -A ORCH_MODULE_STATUSES=()
# Module-level Packages: metadata column, keyed like ORCH_MODULE_STATUSES.
declare -A ORCH_MODULE_PACKAGES=()

orch_reset_state() {
  ORCH_ITEM_IDS=()
  ORCH_ITEM_TITLES=()
  ORCH_ITEM_STATUSES=()
  ORCH_ITEM_DEPS=()
  ORCH_ITEM_LINES=()
  ORCH_ITEM_MODULES=()
  ORCH_ITEM_FILES=()
  ORCH_ITEM_CHILDREN=()
  ORCH_ITEM_PACKAGES=()
  ORCH_MODULE_STATUSES=()
  ORCH_MODULE_PACKAGES=()
}

# --- Packages: scope tags (PKG-001, tagged monorepo tier) --------------------

# Normalise one Packages: entry for matching: trim whitespace/backticks, strip
# a leading packages/ or apps/ root, lowercase. `packages/Core` matches `core`.
orch_pkg_normalize() {
  local entry="$1"
  # shellcheck disable=SC2016 # the backtick is a literal in the sed class
  entry=$(printf '%s' "$entry" | sed -E 's/^[[:space:]`]+//; s/[[:space:]`]+$//')
  entry="${entry#packages/}"
  entry="${entry#apps/}"
  printf '%s' "${entry,,}"
}

# Effective Packages: value for item $1 — the item's own field, else its
# module's metadata-table column (docs/monorepo.md: items inherit from the
# module when the field is omitted).
orch_item_packages() {
  local i="$1"
  local pkgs="${ORCH_ITEM_PACKAGES[$i]}"
  if [[ -z "$pkgs" ]]; then
    local key
    key=$(orch_module_status_key "${ORCH_ITEM_MODULES[$i]}" "${ORCH_ITEM_CHILDREN[$i]}")
    pkgs="${ORCH_MODULE_PACKAGES[$key]:-}"
  fi
  printf '%s' "$pkgs"
}

# True when item $1's effective Packages: include $2 (normalised comparison).
# An untagged item matches no package filter.
orch_item_matches_package() {
  local i="$1" filter="$2"
  [[ -z "$filter" ]] && return 0
  local want pkgs entry
  want=$(orch_pkg_normalize "$filter")
  pkgs=$(orch_item_packages "$i")
  [[ -n "$pkgs" ]] || return 1
  local entries
  IFS=',' read -ra entries <<< "$pkgs"
  for entry in "${entries[@]}"; do
    [[ "$(orch_pkg_normalize "$entry")" == "$want" ]] && return 0
  done
  return 1
}

# Module IDs are bare within a child plan (D-002), so federated status lookups
# must retain the owning child as well. Single-root plans keep their historical
# bare key for compatibility.
orch_module_status_key() {
  local module="$1" child="${2:-}"
  if [[ -n "$child" ]]; then
    printf '%s:%s' "${child,,}" "${module^^}"
  else
    printf '%s' "${module^^}"
  fi
}

orch_set_module_status() {
  local module="$1" status="$2" child="${3:-}" key
  key=$(orch_module_status_key "$module" "$child")
  ORCH_MODULE_STATUSES["$key"]="$status"
}

orch_module_status() {
  local module="$1" child="${2:-}" key
  if [[ "$module" == *:* ]]; then
    child="${module%%:*}"
    module="${module#*:}"
  fi
  if [[ -n "$child" ]]; then
    key=$(orch_module_status_key "$module" "$child")
    if [[ -n "${ORCH_MODULE_STATUSES[$key]+x}" ]]; then
      printf '%s' "${ORCH_MODULE_STATUSES[$key]}"
      return 0
    fi
  fi
  key=$(orch_module_status_key "$module")
  printf '%s' "${ORCH_MODULE_STATUSES[$key]:-Unknown}"
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
    Done*) echo "Complete" ;;
    "In Progress"*) echo "In Progress" ;;
    Ready*) echo "Ready" ;;
    Proposed*) echo "Draft" ;;
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

# --- Federated nested-plan traversal (MONO-003) ---
#
# Orchestration reuses the child-plan link grammar MONO-001/002 established for
# lint (## Child Plans links, path-derived child names, <name>:<ID> cross-tree
# refs). normalize_path / resolve_child_plan_links come from lib/lint.sh, which
# bin/aps sources before this file.

# Emit every plan root in a federation, one per line: the given root plus every
# child root reachable transitively via `## Child Plans` links. Deduped on
# normalised paths. A plain single-root plan yields just itself.
# Usage: orch_plan_roots "path/to/plans"
orch_plan_roots() {
  local start="$1"
  local -A seen=()
  local queue=("$start")
  local roots=()

  while [[ ${#queue[@]} -gt 0 ]]; do
    local root="${queue[0]}"
    queue=("${queue[@]:1}")
    local key
    key=$(normalize_path "$root")
    [[ -n "${seen[$key]:-}" ]] && continue
    seen["$key"]=1
    roots+=("$root")

    local index_file="$root/index.aps.md"
    [[ -f "$index_file" ]] || continue
    local child_index
    while IFS= read -r child_index; do
      [[ -n "$child_index" ]] || continue
      queue+=("$(dirname "$child_index")")
    done < <(resolve_child_plan_links "$index_file")
  done

  printf '%s\n' "${roots[@]}"
}

# Path-derived child name for a plan root (the directory segment above plans/).
# Matches lint's build_child_registry naming so cross-tree refs line up. A root
# with no directory component (the common single-root "plans" case) has no
# child segment and yields "" — matching the Rust `child_name` and the
# ORCH_ITEM_CHILDREN "empty in single-root" invariant.
# Usage: orch_child_name "path/to/plans"
orch_child_name() {
  local root parent
  root=$(normalize_path "$1")
  parent=$(dirname "$root")
  [[ "$parent" == "." || "$parent" == "/" ]] && { printf ''; return; }
  basename "$parent"
}

orch_item_matches_child() {
  local item_index="$1"
  local child="$2"

  [[ -z "$child" ]] && return 0
  [[ "${ORCH_ITEM_CHILDREN[$item_index],,}" == "${child,,}" ]]
}

# Split a possibly-prefixed reference into its child name / bare ID.
orch_ref_child() { local r="$1"; [[ "$r" == *:* ]] && printf '%s' "${r%%:*}" || printf ''; }
orch_ref_id() { local r="$1"; printf '%s' "${r##*:}"; }

# Resolve a work-item reference to a loaded item index. Accepts a bare ID
# ("AUTH-001") or a cross-tree ref ("core:AUTH-001"); an optional scope child
# constrains a bare ID to one tree. Echoes the index and returns 0 on a unique
# match; returns 1 when nothing matches; returns 2 (echoing the space-joined
# candidate indices) when a bare ID is ambiguous across child trees.
# Usage: rc=0; idx=$(orch_resolve_ref "$ref" "$scope") || rc=$?
#   (guard the substitution with `|| rc=$?` so `set -e` does not abort on the
#   not-found (1) / ambiguous (2) return codes)
orch_resolve_ref() {
  local ref="$1"
  local scope="${2:-}"
  local rchild rid
  rchild=$(orch_ref_child "$ref")
  rid=$(orch_ref_id "$ref")
  [[ -n "$scope" && -z "$rchild" ]] && rchild="$scope"

  local i
  local matches=()
  for i in "${!ORCH_ITEM_IDS[@]}"; do
    [[ "${ORCH_ITEM_IDS[$i]}" == "$rid" ]] || continue
    if [[ -n "$rchild" ]]; then
      [[ "${ORCH_ITEM_CHILDREN[$i],,}" == "${rchild,,}" ]] || continue
    fi
    matches+=("$i")
  done

  case ${#matches[@]} in
    0) return 1 ;;
    1) printf '%s' "${matches[0]}"; return 0 ;;
    *) printf '%s' "${matches[*]}"; return 2 ;;
  esac
}

orch_load_index_modules() {
  local plan_root="$1"
  local child_name="${2:-}"
  local index_file="$plan_root/index.aps.md"

  [[ -f "$index_file" ]] || return 0

  while IFS='|' read -r module status; do
    orch_set_module_status "$module" "$status" "$child_name"
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

# Load work items from a single plan root's modules/ directory, tagging each
# with the given child name. Returns 1 when the root has no modules/ dir.
orch_load_root_work_items() {
  local plan_root="$1"
  local load_all="${2:-false}"
  local child_name="${3:-}"
  local module_dir="$plan_root/modules"

  [[ -d "$module_dir" ]] || return 1

  local file
  while IFS= read -r file; do
    local module_id module_status module_packages
    module_id=$(get_module_id "$file")
    module_status=$(get_status "$file")
    module_status=$(orch_normalize_status "$module_status" "Draft")
    module_packages=$(get_module_packages "$file")

    [[ -n "$module_id" ]] || module_id=$(basename "$file" .aps.md | tr '[:lower:]' '[:upper:]')
    orch_set_module_status "$module_id" "$module_status" "$child_name"
    ORCH_MODULE_PACKAGES["$(orch_module_status_key "$module_id" "$child_name")"]="$module_packages"

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
      ORCH_ITEM_CHILDREN+=("$child_name")
      ORCH_ITEM_PACKAGES+=("$(orch_field_value "$content" "Packages")")
    done <<< "$(get_work_items "$file")"
  done < <(find "$module_dir" -type f -name "*.aps.md" ! -name ".*" 2>/dev/null | sort)
}

# Load work items across a whole federation: the given root plus every child
# plan reachable via `## Child Plans` (MONO-003). Each root's index-module
# statuses are loaded too. Returns 1 only when no root in the tree owns a
# modules/ directory (a federation parent owns none of its own — its children
# supply the work). In a plain single-root plan this loads exactly that root.
orch_load_work_items() {
  local plan_root="$1"
  local load_all="${2:-false}"
  local loaded_any=false
  local root child

  while IFS= read -r root; do
    [[ -n "$root" ]] || continue
    child=$(orch_child_name "$root")
    orch_load_index_modules "$root" "$child"
    if orch_load_root_work_items "$root" "$load_all" "$child"; then
      loaded_any=true
    fi
  done < <(orch_plan_roots "$plan_root")

  [[ "$loaded_any" == true ]] && return 0 || return 1
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
  local self_child="${2:-}"

  # Cross-tree work-item ref (child:ID) — resolve within the named child tree.
  if [[ "$dep" == *:* ]]; then
    local idx rc=0
    idx=$(orch_resolve_ref "$dep") || rc=$?
    [[ $rc -eq 0 && "${ORCH_ITEM_STATUSES[$idx]}" == "Complete" ]]
    return
  fi

  if [[ "$dep" =~ ^[A-Z]+-[0-9]+$ ]]; then
    # Decision dependencies (D-NNN) are resolved in the plan text, not as work items.
    [[ "$dep" == D-* ]] && return 0

    # A bare ID means "an item in my own tree" (D-002: the same bare ID may
    # exist in sibling trees). Resolve within the depending item's child first
    # so declaration order can't misattribute it; only fall back to a
    # federation-wide first match when the ID isn't defined in-tree.
    local idx rc=0
    idx=$(orch_resolve_ref "$dep" "$self_child") || rc=$?
    [[ $rc -eq 0 ]] || idx=$(orch_item_index "$dep" || true)
    [[ -n "$idx" && "${ORCH_ITEM_STATUSES[$idx]}" == "Complete" ]]
    return
  fi

  local module_status
  module_status=$(orch_module_status "$dep" "$self_child")
  [[ "$module_status" == "Complete" ]]
}

orch_deps_complete() {
  local deps="$1"
  local self_child="${2:-}"
  local dep_ids=()
  local dep

  [[ -z "$deps" || "$deps" == "None" || "$deps" == "-" ]] && return 0
  [[ ! "$deps" =~ [[:alnum:]] ]] && return 0

  while IFS= read -r dep; do
    [[ -n "$dep" ]] && dep_ids+=("$dep")
  done < <(orch_dep_refs "$deps")

  [[ ${#dep_ids[@]} -eq 0 ]] && return 1

  for dep in "${dep_ids[@]}"; do
    orch_dependency_complete "$dep" "$self_child" || return 1
  done

  return 0
}

orch_deps_display() {
  local deps="$1"

  deps=${deps//$'\n'/, }
  echo "${deps:-None}"
}

# Extract dependency tokens, preserving an optional <name>: cross-tree prefix
# (MONO-003), so "core:AUTH-001" survives as one token for federated resolution
# and graph edges. The prefix is matched case-insensitively (child names are
# path-derived and compared case-insensitively) so an all-caps ref like
# "CORE:AUTH-001" isn't silently split into a bogus module dep + bare ID. Bare
# IDs and module deps pass through unchanged.
orch_dep_refs() {
  local deps="$1"

  printf '%s\n' "$deps" | grep -oE '([A-Za-z0-9][A-Za-z0-9-]*:)?[A-Z]+-[0-9]+|[A-Z]{2,}' || true
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
    local self_child="${ORCH_ITEM_CHILDREN[$idx]}"
    local found_learning="false" dep_rc
    while IFS= read -r dep; do
      [[ -n "$dep" ]] || continue
      # Resolve cross-tree (child:ID) refs within the named tree and bare IDs
      # within the depending item's own tree, matching orch_dependency_complete
      # — so a learning is pulled from the right sibling, not a same-ID first
      # match in another tree.
      if [[ "$dep" == *:* ]]; then
        dep_rc=0; dep_idx=$(orch_resolve_ref "$dep") || dep_rc=$?
      else
        dep_rc=0; dep_idx=$(orch_resolve_ref "$dep" "$self_child") || dep_rc=$?
        [[ $dep_rc -eq 0 ]] || dep_idx=$(orch_item_index "$dep" || true)
      fi
      [[ -n "$dep_idx" ]] || continue
      local dep_content dep_learning
      dep_content=$(orch_item_content "${ORCH_ITEM_FILES[$dep_idx]}" "${ORCH_ITEM_LINES[$dep_idx]}")
      dep_learning=$(orch_field_value "$dep_content" "Learning")
      if [[ -n "$dep_learning" ]]; then
        echo "- $dep: $dep_learning"
        found_learning="true"
      fi
    done < <(orch_dep_refs "$deps")
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
  local child_scope=""
  local package_filter=""
  local by_package=false

  while [[ $# -gt 0 ]]; do
    case "$1" in
      --plans)
        plan_root="${2:-}"
        [[ -n "$plan_root" ]] || { error "--plans requires a directory"; return 1; }
        shift 2
        ;;
      --child)
        child_scope="${2:-}"
        [[ -n "$child_scope" ]] || { error "--child requires a child plan name"; return 1; }
        shift 2
        ;;
      --package)
        package_filter="${2:-}"
        [[ -n "$package_filter" ]] || { error "--package requires a package name"; return 1; }
        shift 2
        ;;
      --by-package)
        by_package=true
        shift
        ;;
      --strict)
        strict=true
        shift
        ;;
      --help|-h)
        cat <<EOF
Usage: aps next [module] [options]

Show the next Ready work item whose dependencies are Complete. From a
federated root (a plan with a ## Child Plans section) this searches the whole
nested tree; --child scopes it to one child plan.

Arguments:
  module    Optional module ID or module file name, e.g. AUTH or auth

Options:
  --plans DIR      Plan root directory (default: plans)
  --child NAME     Scope to one child plan (path-derived name, e.g. core)
  --package NAME   Only items whose Packages: tags include NAME (item field,
                   else module metadata; tagged monorepo tier)
  --by-package     List every ready item, grouped by package; untagged items
                   appear under (untagged)
  --help           Show this help
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
  orch_load_work_items "$plan_root" "true" || {
    error "No modules directory found: $plan_root/modules"
    return 1
  }

  # Collect the candidate indexes (same gating for both output modes).
  local i candidates=()
  for i in "${!ORCH_ITEM_IDS[@]}"; do
    orch_item_matches_child "$i" "$child_scope" || continue
    orch_item_matches_module "$i" "$module_filter" || continue
    orch_item_matches_package "$i" "$package_filter" || continue
    case "$(orch_module_status "${ORCH_ITEM_MODULES[$i]}" "${ORCH_ITEM_CHILDREN[$i]}")" in
      Ready|"In Progress") ;;
      *) continue ;;
    esac
    [[ "${ORCH_ITEM_STATUSES[$i]}" == "Ready" ]] || continue
    orch_deps_complete "${ORCH_ITEM_DEPS[$i]}" "${ORCH_ITEM_CHILDREN[$i]}" || continue
    candidates+=("$i")
    # The default mode only needs the first candidate.
    [[ "$by_package" == true ]] || break
  done

  if [[ ${#candidates[@]} -gt 0 ]]; then
    if [[ "$by_package" == true ]]; then
      # Group by normalised package name; a multi-tagged item appears under
      # each of its packages. Headings sort lexically; (untagged) comes last.
      local -A groups=()
      local pkgs entry name line
      for i in "${candidates[@]}"; do
        line="  ${ORCH_ITEM_IDS[$i]}: ${ORCH_ITEM_TITLES[$i]} (${ORCH_ITEM_MODULES[$i]})"
        pkgs=$(orch_item_packages "$i")
        if [[ -z "$pkgs" ]]; then
          groups["(untagged)"]+="$line"$'\n'
          continue
        fi
        local entries
        IFS=',' read -ra entries <<< "$pkgs"
        for entry in "${entries[@]}"; do
          name=$(orch_pkg_normalize "$entry")
          [[ -n "$name" ]] || continue
          groups["$name"]+="$line"$'\n'
        done
      done
      local first=true
      while IFS= read -r name; do
        [[ -n "$name" && "$name" != "(untagged)" ]] || continue
        [[ "$first" == true ]] || echo ""
        first=false
        echo "$name:"
        printf '%s' "${groups[$name]}"
      done < <(printf '%s\n' "${!groups[@]}" | sort)
      if [[ -n "${groups[(untagged)]:-}" ]]; then
        [[ "$first" == true ]] || echo ""
        echo "(untagged):"
        printf '%s' "${groups[(untagged)]}"
      fi
    else
      i="${candidates[0]}"
      echo "${ORCH_ITEM_IDS[$i]}: ${ORCH_ITEM_TITLES[$i]}"
      echo "Module: ${ORCH_ITEM_MODULES[$i]} | Dependencies: $(orch_deps_display "${ORCH_ITEM_DEPS[$i]}") | Status: ${ORCH_ITEM_STATUSES[$i]}"
      echo "File: ${ORCH_ITEM_FILES[$i]}"
    fi
    return 0
  fi

  local note=""
  [[ -n "$module_filter" ]] && note+=" for module: $module_filter"
  [[ -n "$package_filter" ]] && note+=" for package: $package_filter"
  [[ -n "$child_scope" ]] && note+=" in child: $child_scope"
  warn "No ready work item found$note"
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
  local child_scope=""

  while [[ $# -gt 0 ]]; do
    case "$1" in
      --plans)
        plan_root="${2:-}"
        [[ -n "$plan_root" ]] || { error "--plans requires a directory"; return 1; }
        shift 2
        ;;
      --child)
        child_scope="${2:-}"
        [[ -n "$child_scope" ]] || { error "--child requires a child plan name"; return 1; }
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
  ID    Work item ID (e.g. AUTH-003), or a cross-tree ref (e.g. core:AUTH-003)
        in a federated plan

Options:
  --plans DIR    Plan root directory (default: plans)
  --child NAME   Scope resolution to one child plan (disambiguates a bare ID
                 that collides across child trees)
  --help         Show this help

Validates that the item is Ready and its dependencies are Complete before
mutating the markdown. Suggests a branch name (work/<id>) - branch creation
is left to the user per ORCH D-003. In a federated tree the item is updated in
its owning child module file, never a parent or same-ID sibling.
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
  orch_load_work_items "$plan_root" "true" || {
    error "No modules directory found: $plan_root/modules"
    return 1
  }

  local idx rc
  local rc=0
  idx=$(orch_resolve_ref "$id" "$child_scope") || rc=$?
  case $rc in
    1) error "Work item not found: $id"; return 1 ;;
    2) error "Ambiguous work item '$id' defined in multiple child trees; disambiguate with <child>:$id or --child <name>"; return 1 ;;
  esac

  # Use the resolved bare ID for markdown rewrites and messaging; the user's
  # input may carry a <child>: prefix that does not appear in the file.
  local resolved_id="${ORCH_ITEM_IDS[$idx]}"
  local current="${ORCH_ITEM_STATUSES[$idx]}"
  local file="${ORCH_ITEM_FILES[$idx]}"
  local module_id="${ORCH_ITEM_MODULES[$idx]}"
  local module_status
  module_status=$(orch_module_status "$module_id" "${ORCH_ITEM_CHILDREN[$idx]}")
  local deps="${ORCH_ITEM_DEPS[$idx]}"
  local already_started="false"

  case "$module_status" in
    Ready|"In Progress") ;;
    *)
      error "$resolved_id belongs to module $module_id (status: $module_status) - module must be Ready or In Progress to start work items"
      return 1
      ;;
  esac

  case "$current" in
    Ready) ;;
    "In Progress")
      already_started="true"
      ;;
    Complete)
      error "$resolved_id is already Complete - cannot restart"
      return 1
      ;;
    *)
      error "$resolved_id has status '$current' - cannot start (must be Ready)"
      return 1
      ;;
  esac

  if ! orch_deps_complete "$deps" "${ORCH_ITEM_CHILDREN[$idx]}"; then
    error "$resolved_id has unmet dependencies: $(orch_deps_display "$deps")"
    return 1
  fi

  if [[ "$already_started" != "true" ]]; then
    orch_rewrite_status "$file" "$resolved_id" "In Progress" || return 1
    ORCH_ITEM_STATUSES[$idx]="In Progress"
  fi

  local context_file
  context_file=$(orch_context_package "$plan_root" "$idx") || return 1

  local lower_id="${resolved_id,,}"
  if [[ "$already_started" == "true" ]]; then
    warn "$resolved_id is already In Progress (no status change)"
  else
    echo "Marked $resolved_id as In Progress"
  fi
  echo "Suggested branch: work/$lower_id"
  echo "File: $file"
  echo "Context package: $context_file"
}

cmd_complete() {
  local plan_root="" strict=false
  local id=""
  local learning=""
  local child_scope=""

  while [[ $# -gt 0 ]]; do
    case "$1" in
      --plans)
        plan_root="${2:-}"
        [[ -n "$plan_root" ]] || { error "--plans requires a directory"; return 1; }
        shift 2
        ;;
      --child)
        child_scope="${2:-}"
        [[ -n "$child_scope" ]] || { error "--child requires a child plan name"; return 1; }
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
  ID    Work item ID (e.g. AUTH-003), or a cross-tree ref (e.g. core:AUTH-003)
        in a federated plan

Options:
  --plans DIR        Plan root directory (default: plans)
  --child NAME       Scope resolution to one child plan (disambiguates a bare
                     ID that collides across child trees)
  --learning "..."   Append a learning line after Validation (ORCH D-002)
  --help             Show this help

Validates that the item is In Progress before mutating the markdown.
Stamps Status as "Complete: YYYY-MM-DD" using today's UTC date. In a federated
tree the item is updated in its owning child module file.
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
  orch_load_work_items "$plan_root" "true" || {
    error "No modules directory found: $plan_root/modules"
    return 1
  }

  local idx rc
  local rc=0
  idx=$(orch_resolve_ref "$id" "$child_scope") || rc=$?
  case $rc in
    1) error "Work item not found: $id"; return 1 ;;
    2) error "Ambiguous work item '$id' defined in multiple child trees; disambiguate with <child>:$id or --child <name>"; return 1 ;;
  esac

  local resolved_id="${ORCH_ITEM_IDS[$idx]}"
  local current="${ORCH_ITEM_STATUSES[$idx]}"
  local file="${ORCH_ITEM_FILES[$idx]}"

  case "$current" in
    "In Progress") ;;
    Complete)
      warn "$resolved_id is already Complete (no change)"
      return 0
      ;;
    *)
      error "$resolved_id has status '$current' - cannot complete (must be In Progress)"
      return 1
      ;;
  esac

  if [[ -z "$learning" && -t 0 ]]; then
    read -r -p "Learning (optional): " learning
  fi

  local today
  today=$(orch_today)
  orch_rewrite_status "$file" "$resolved_id" "Complete: $today" || return 1

  if [[ -n "$learning" ]]; then
    orch_append_learning "$file" "$resolved_id" "$learning" || return 1
  fi

  echo "Marked $resolved_id as Complete: $today"
  [[ -n "$learning" ]] && echo "Learning recorded for $resolved_id"
  echo "File: $file"
}

cmd_graph() {
  local plan_root="" strict=false
  local module_filter=""
  local child_scope=""

  while [[ $# -gt 0 ]]; do
    case "$1" in
      --plans)
        plan_root="${2:-}"
        [[ -n "$plan_root" ]] || { error "--plans requires a directory"; return 1; }
        shift 2
        ;;
      --child)
        child_scope="${2:-}"
        [[ -n "$child_scope" ]] || { error "--child requires a child plan name"; return 1; }
        shift 2
        ;;
      --strict)
        strict=true
        shift
        ;;
      --help|-h)
        cat <<EOF
Usage: aps graph [module] [options]

Show work items and dependency arrows. From a federated root the graph spans
the whole nested tree and renders cross-tree (<name>:<ID>) dependency edges;
--child scopes it to one child plan.

Arguments:
  module    Optional module ID or module file name, e.g. AUTH or auth

Options:
  --plans DIR    Plan root directory (default: plans)
  --child NAME   Scope to one child plan (path-derived name, e.g. core)
  --help         Show this help
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
  orch_load_work_items "$plan_root" "true" || {
    error "No modules directory found: $plan_root/modules"
    return 1
  }

  local i dep dep_idx rc shown="false" deps_display
  for i in "${!ORCH_ITEM_IDS[@]}"; do
    orch_item_matches_child "$i" "$child_scope" || continue
    orch_item_matches_module "$i" "$module_filter" || continue
    shown="true"
    echo "${ORCH_ITEM_IDS[$i]} [${ORCH_ITEM_STATUSES[$i]}] ${ORCH_ITEM_TITLES[$i]}"

    deps_display=""
    while IFS= read -r dep; do
      [[ -n "$dep" ]] || continue
      if [[ "$dep" == *:* ]]; then
        # Cross-tree ref: keep the <name>: prefix and resolve within that child.
        rc=0
        dep_idx=$(orch_resolve_ref "$dep") || rc=$?
        if [[ $rc -eq 0 ]]; then
          deps_display+=" ${dep}[${ORCH_ITEM_STATUSES[$dep_idx]}]"
        else
          deps_display+=" ${dep}[Unknown]"
        fi
        continue
      fi
      dep_idx=$(orch_item_index "$dep" || true)
      if [[ -n "$dep_idx" ]]; then
        deps_display+=" ${ORCH_ITEM_IDS[$dep_idx]}[${ORCH_ITEM_STATUSES[$dep_idx]}]"
      else
        deps_display+=" ${dep}[$(orch_module_status "$dep" "${ORCH_ITEM_CHILDREN[$i]}")]"
      fi
    done < <(orch_dep_refs "${ORCH_ITEM_DEPS[$i]}")

    if [[ -n "$deps_display" ]]; then
      echo "  <-${deps_display}"
    else
      echo "  <- none"
    fi
  done

  if [[ "$shown" != "true" ]]; then
    local scope_note=""
    [[ -n "$child_scope" ]] && scope_note=" in child: $child_scope"
    if [[ -n "$module_filter" ]]; then
      warn "No work items found for module: $module_filter$scope_note"
    else
      warn "No work items found$scope_note"
    fi
    return 1
  fi
}

# First actionable item's ID within a child scope, or "—" when none — the same
# selection cmd_next makes, reused for the roll-up "Next ready item" column.
orch_child_next_ready() {
  local child="$1"
  local i
  for i in "${!ORCH_ITEM_IDS[@]}"; do
    orch_item_matches_child "$i" "$child" || continue
    case "$(orch_module_status "${ORCH_ITEM_MODULES[$i]}" "${ORCH_ITEM_CHILDREN[$i]}")" in
      Ready|"In Progress") ;;
      *) continue ;;
    esac
    [[ "${ORCH_ITEM_STATUSES[$i]}" == "Ready" ]] || continue
    orch_deps_complete "${ORCH_ITEM_DEPS[$i]}" "${ORCH_ITEM_CHILDREN[$i]}" || continue
    printf '%s' "${ORCH_ITEM_IDS[$i]}"
    return 0
  done
  printf '%s' "—"
}

cmd_rollup() {
  local plan_root="" strict=false
  local by_package=false

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
Usage: aps rollup [options]

Print a Markdown roll-up table for a federated (nested-plans) parent: one row
per child plan with modules complete/total, the next ready item, and an overall
status. The root index stays hand-authored — copy this table into the parent's
## Roll-up section at session end to keep it current.

Options:
  --plans DIR   Plan root directory (default: plans)
  --by-package  Print modules grouped by Packages: tag instead (tagged
                monorepo tier; untagged modules appear under (untagged))
  --help        Show this help
EOF
        return 0
        ;;
      --by-package)
        by_package=true
        shift
        ;;
      -*)
        error "Unknown option: $1"
        return 1
        ;;
      *)
        error "Unexpected argument: $1"
        return 1
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
  orch_load_work_items "$plan_root" "true" || {
    error "No modules directory found: $plan_root/modules"
    return 1
  }

  # PKG-003: grouped module view for the tagged tier — the generated form of
  # docs/monorepo.md's "Modules by Package" section. Headings sort lexically;
  # (untagged) comes last. Works on plain single roots and federations alike.
  if [[ "$by_package" == true ]]; then
    local -A pgroups=()
    local proot pfile pid pst ppkgs pentry pname prow pchild
    while IFS= read -r proot; do
      [[ -d "$proot/modules" ]] || continue
      pchild=$(orch_child_name "$proot")
      while IFS= read -r pfile; do
        pid=$(get_module_id "$pfile")
        [[ -n "$pid" ]] || pid=$(basename "$pfile" .aps.md | tr '[:lower:]' '[:upper:]')
        [[ -n "$pchild" ]] && pid="$pchild:$pid"
        pst=$(orch_normalize_status "$(get_status "$pfile")" "Draft")
        prow="| $pid | $pst |"$'\n'
        ppkgs=$(get_module_packages "$pfile")
        if [[ -z "$ppkgs" ]]; then
          pgroups["(untagged)"]+="$prow"
          continue
        fi
        local pentries
        IFS=',' read -ra pentries <<< "$ppkgs"
        for pentry in "${pentries[@]}"; do
          pname=$(orch_pkg_normalize "$pentry")
          [[ -n "$pname" ]] || continue
          pgroups["$pname"]+="$prow"
        done
      done < <(find "$proot/modules" -type f -name "*.aps.md" ! -name ".*" 2>/dev/null | sort)
    done < <(orch_plan_roots "$plan_root")

    local pfirst=true
    while IFS= read -r pname; do
      [[ -n "$pname" && "$pname" != "(untagged)" ]] || continue
      [[ "$pfirst" == true ]] || echo ""
      pfirst=false
      echo "### $pname"
      echo ""
      echo "| Module | Status |"
      echo "| ------ | ------ |"
      printf '%s' "${pgroups[$pname]}"
    done < <(printf '%s\n' "${!pgroups[@]}" | sort)
    if [[ -n "${pgroups[(untagged)]:-}" ]]; then
      [[ "$pfirst" == true ]] || echo ""
      echo "### (untagged)"
      echo ""
      echo "| Module | Status |"
      echo "| ------ | ------ |"
      printf '%s' "${pgroups[(untagged)]}"
    fi
    return 0
  fi

  echo "| Child | Modules (complete/total) | Next ready item | Status |"
  echo "| ----- | ------------------------ | --------------- | ------ |"

  local root child first="true" shown="false"
  while IFS= read -r root; do
    # The first root is the federation parent itself; roll-up covers children.
    if [[ "$first" == "true" ]]; then
      first="false"
      continue
    fi
    local module_dir="$root/modules"
    [[ -d "$module_dir" ]] || continue
    shown="true"
    child=$(orch_child_name "$root")

    local total=0 complete=0 inprogress=0 mf st
    while IFS= read -r mf; do
      [[ -n "$mf" ]] || continue
      st=$(orch_normalize_status "$(get_status "$mf")" "Draft")
      total=$((total + 1))
      [[ "$st" == "Complete" ]] && complete=$((complete + 1))
      [[ "$st" == "In Progress" ]] && inprogress=$((inprogress + 1))
    done < <(find "$module_dir" -type f -name "*.aps.md" ! -name ".*" 2>/dev/null | sort)

    local next_ready overall
    next_ready=$(orch_child_next_ready "$child")
    if [[ "$total" -gt 0 && "$complete" -eq "$total" ]]; then
      overall="Complete"
    elif [[ "$inprogress" -gt 0 ]]; then
      overall="In Progress"
    else
      overall="Ready"
    fi

    printf '| %s | %s/%s | %s | %s |\n' "$child" "$complete" "$total" "$next_ready" "$overall"
  done < <(orch_plan_roots "$plan_root")

  if [[ "$shown" != "true" ]]; then
    warn "No child plans found under $plan_root (rollup is for federated parents)"
    return 1
  fi
}
