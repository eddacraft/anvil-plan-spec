#!/usr/bin/env bash
#
# Orchestration helpers — work item collection, status resolution,
# dependency-aware "next ready" selection. Used by `aps next`.
#

# orch_module_status <module_file>
# Reads the metadata table (| ID | ... | Status |) and prints the value
# of the Status column. Empty string if not found.
orch_module_status() {
  local file="$1"
  awk '
    /^\| *ID *\|/ {
      n = split($0, cols, "|")
      for (i = 1; i <= n; i++) {
        gsub(/^ +| +$/, "", cols[i])
        if (cols[i] == "Status") status_col = i
      }
      next
    }
    status_col && /^\|/ && !/^\| *ID *\|/ {
      row = $0
      gsub(/[|: -]/, "", row)
      if (row == "") next
      n = split($0, vals, "|")
      gsub(/^ +| +$/, "", vals[status_col])
      print vals[status_col]
      exit
    }
  ' "$file"
}

# orch_extract_items <module_file>
# Emits TSV rows: file<TAB>id<TAB>title<TAB>status<TAB>deps_csv
# Status precedence (highest first):
#   1. `- **Status:** <word>` field in the work item body
#   2. " — <Status>" suffix in the `### ID: title` header
#   3. Module-level Status from the metadata table (when Draft, Archived,
#      or Proposed — these suppress the implicit-Ready promotion)
#   4. "Ready" (default for items in Ready / In Progress modules)
# Recognised statuses: Complete, In Progress, Ready, Draft, Blocked.
# Dependencies are extracted as IDs matching `[A-Z]+-[0-9]+`.
orch_extract_items() {
  local file="$1"
  [[ -f "$file" ]] || return 0

  local module_status
  module_status="$(orch_module_status "$file")"
  local default_status="Ready"
  case "$module_status" in
    Draft|Archived|Proposed) default_status="$module_status" ;;
  esac

  awk -v file="$file" -v default_status="$default_status" '
    BEGIN { id = "" }

    function emit() {
      if (id == "") return
      effective = explicit_status
      if (effective == "") effective = header_status
      if (effective == "") effective = default_status

      deps_str = ""
      for (i = 0; i < dep_n; i++) {
        if (i > 0) deps_str = deps_str ","
        deps_str = deps_str dep_arr[i]
      }
      printf "%s\t%s\t%s\t%s\t%s\n", file, id, title, effective, deps_str
    }

    function reset() {
      id = ""; title = ""; explicit_status = ""; header_status = ""
      delete dep_arr; dep_n = 0
    }

    function detect_status(line) {
      if (line ~ / In Progress/) return "In Progress"
      if (line ~ / Complete/)    return "Complete"
      if (line ~ / Blocked/)     return "Blocked"
      if (line ~ / Ready/)       return "Ready"
      if (line ~ / Draft/)       return "Draft"
      return ""
    }

    /^### [A-Z]+-[0-9]+:/ {
      emit()
      reset()
      line = $0
      sub(/^### /, "", line)
      match(line, /^[A-Z]+-[0-9]+/)
      id = substr(line, RSTART, RLENGTH)
      rest = substr(line, RLENGTH + 1)
      sub(/^: */, "", rest)
      # Optional " — STATUS [date]" trailing marker
      if (match(rest, / — [A-Z][A-Za-z ]+( [0-9-]+)?$/)) {
        marker = substr(rest, RSTART)
        header_status = detect_status(marker)
        rest = substr(rest, 1, RSTART - 1)
      }
      title = rest
      next
    }

    /^### / {
      emit()
      reset()
      next
    }

    /^## / {
      emit()
      reset()
      next
    }

    id != "" && /^- \*\*Status:\*\*/ {
      line = $0
      sub(/^- \*\*Status:\*\* */, "", line)
      explicit_status = detect_status(" " line)
      next
    }

    id != "" && /^- \*\*Dependencies:\*\*/ {
      line = $0
      while (match(line, /[A-Z]+-[0-9]+/)) {
        dep_arr[dep_n++] = substr(line, RSTART, RLENGTH)
        line = substr(line, RSTART + RLENGTH)
      }
      next
    }

    END { emit() }
  ' "$file"
}

# orch_collect_work_items <plans_dir>
# Walks plans/modules/*.aps.md and emits the union of items.
orch_collect_work_items() {
  local plans_dir="${1:-plans}"
  local f
  shopt -s nullglob
  for f in "$plans_dir"/modules/*.aps.md; do
    orch_extract_items "$f"
  done
  shopt -u nullglob
}

# orch_resolve_next <plans_dir> [module_filter]
# Prints the next Ready item whose [A-Z]+-NNN dependencies are all Complete.
# Output: file<TAB>id<TAB>title<TAB>status<TAB>deps_csv (TSV).
# Returns 1 if no candidate found.
# Module filter matches the basename (without extension) of the module file,
# or any substring of the file path — case-insensitive.
orch_resolve_next() {
  local plans_dir="${1:-plans}"
  local filter="${2:-}"
  local items
  items="$(orch_collect_work_items "$plans_dir")"
  [[ -z "$items" ]] && return 1

  # Build status index: ID -> STATUS
  # On duplicate IDs, prefer Complete (so legitimate completed work isn't
  # masked by a stale Ready entry elsewhere). Warn to stderr either way.
  local -A status_by_id=()
  while IFS=$'\t' read -r f id title status deps; do
    [[ -z "$id" ]] && continue
    if [[ -n "${status_by_id[$id]+x}" ]]; then
      printf 'aps next: warning: duplicate work item id %s (keeping %s, ignoring %s in %s)\n' \
        "$id" "${status_by_id[$id]}" "$status" "$f" >&2
      [[ "${status_by_id[$id]}" == "Complete" ]] && continue
      [[ "$status" != "Complete" ]] && continue
    fi
    status_by_id["$id"]="$status"
  done <<< "$items"

  # Filter and pick first satisfying candidate
  # Skip duplicate IDs during candidate selection — only the first
  # occurrence is considered. The status index already has the
  # Complete-wins resolution baked in, so checking against it correctly
  # treats subsequent occurrences as already-known.
  local picked=""
  local -A seen_candidate=()
  while IFS=$'\t' read -r f id title status deps; do
    [[ -z "$id" ]] && continue
    if [[ -n "${seen_candidate[$id]+x}" ]]; then
      continue
    fi
    seen_candidate["$id"]=1
    # Use the resolved status from the dedup'd index, not the row's status
    status="${status_by_id[$id]}"
    [[ "$status" != "Ready" ]] && continue

    if [[ -n "$filter" ]]; then
      local base
      base="$(basename "$f" .aps.md)"
      if ! [[ "${base,,}" == *"${filter,,}"* || "${f,,}" == *"${filter,,}"* ]]; then
        continue
      fi
    fi

    # Check dependencies
    local satisfied=1
    if [[ -n "$deps" ]]; then
      local IFS=','
      local dep
      for dep in $deps; do
        [[ -z "$dep" ]] && continue
        local dep_status="${status_by_id[$dep]:-}"
        # Unknown dep IDs treated as unsatisfied (likely a typo; surface it).
        if [[ "$dep_status" != "Complete" ]]; then
          satisfied=0
          break
        fi
      done
    fi

    if (( satisfied == 1 )); then
      picked="$f"$'\t'"$id"$'\t'"$title"$'\t'"$status"$'\t'"$deps"
      break
    fi
  done <<< "$items"

  [[ -z "$picked" ]] && return 1
  printf '%s\n' "$picked"
}
