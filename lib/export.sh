#!/usr/bin/env bash
#
# aps export - machine-readable snapshot of a plan tree (INTEGRATIONS-002)
#
# Emits compact JSON (schema aps-export/v1) on stdout: modules that define
# work items, in file order, each with its items in document order. Reuses
# the orchestration loader, so statuses, titles, children, and effective
# Packages: tags match `aps next`/`graph` exactly. Byte-identical to the
# Rust binary's `aps export` (D-039); running it twice byte-matches.

# Escape a string for a JSON value: backslash, quote, and the control
# characters a plan file can realistically contain.
export_json_escape() {
  local s="$1"
  s="${s//\\/\\\\}"
  s="${s//\"/\\\"}"
  s="${s//$'\n'/\\n}"
  s="${s//$'\r'/\\r}"
  s="${s//$'\t'/\\t}"
  printf '%s' "$s"
}

# A raw Dependencies field -> comma-free JSON array elements
# ("A, B" -> "\"A\",\"B\""). Empty field -> empty string.
export_deps_elements() {
  local deps="$1" out="" tok toks
  deps="${deps//$'\n'/ }"
  IFS=',' read -ra toks <<< "$deps"
  for tok in "${toks[@]}"; do
    tok="${tok#"${tok%%[![:space:]]*}"}"
    tok="${tok%"${tok##*[![:space:]]}"}"
    [[ -n "$tok" ]] || continue
    out+="\"$(export_json_escape "$tok")\","
  done
  printf '%s' "${out%,}"
}

# "value" -> "value" JSON string, "" -> null
export_string_or_null() {
  local v="$1"
  if [[ -n "$v" ]]; then
    printf '"%s"' "$(export_json_escape "$v")"
  else
    printf 'null'
  fi
}

cmd_export() {
  local plan_root="" strict=false

  while [[ $# -gt 0 ]]; do
    case "$1" in
      --plans)
        plan_root="${2:-}"
        [[ -n "$plan_root" ]] || { error "--plans requires a directory"; return 1; }
        shift 2
        ;;
      --json)
        # JSON is the only output format; the flag is accepted for clarity.
        shift
        ;;
      --strict)
        strict=true
        shift
        ;;
      --help|-h)
        cat <<EOF
Usage: aps export [--json] [options]

Emit a machine-readable JSON snapshot (schema aps-export/v1) of the plan
tree on stdout: modules that define work items, their statuses, and each
work item's status, line, dependencies, and effective Packages: tags.
Output is compact JSON — pipe through 'jq .' for readability.

Options:
  --plans DIR   Plan root (default: nearest plans/ dir)
  --json        Accepted for clarity; JSON is the only format
  --strict      Fail on a cli_version pin mismatch
  --help        Show this help
EOF
        return 0
        ;;
      *)
        error "Unknown option: $1"
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

  local json='{"schema":"aps-export/v1","generated_by":"aps '"${APS_CLI_VERSION:-0.7.0}"'","plans_dir":"'"$(export_json_escape "$plan_root")"'","modules":['

  local i cur_key="" first_module=true first_item=true
  for i in "${!ORCH_ITEM_IDS[@]}"; do
    local module="${ORCH_ITEM_MODULES[$i]}"
    local child="${ORCH_ITEM_CHILDREN[$i]}"
    local file="${ORCH_ITEM_FILES[$i]}"
    local key="$child|$module|$file"

    if [[ "$key" != "$cur_key" ]]; then
      if [[ -n "$cur_key" ]]; then
        json+=']}'
      fi
      $first_module || json+=','
      first_module=false
      cur_key="$key"
      first_item=true

      local mstatus mtype mpkgs
      mstatus=$(orch_module_status "$module" "$child")
      mtype=$(get_module_type "$file")
      mpkgs="${ORCH_MODULE_PACKAGES[$(orch_module_status_key "$module" "$child")]:-}"

      json+='{"id":"'"$(export_json_escape "$module")"'"'
      json+=',"child":'"$(export_string_or_null "$child")"
      json+=',"file":"'"$(export_json_escape "$file")"'"'
      json+=',"status":"'"$(export_json_escape "$mstatus")"'"'
      json+=',"type":'"$(export_string_or_null "$mtype")"
      json+=',"packages":'"$(export_string_or_null "$mpkgs")"
      json+=',"work_items":['
    fi

    $first_item || json+=','
    first_item=false

    local pkgs
    pkgs=$(orch_item_packages "$i")

    json+='{"id":"'"$(export_json_escape "${ORCH_ITEM_IDS[$i]}")"'"'
    json+=',"title":"'"$(export_json_escape "${ORCH_ITEM_TITLES[$i]}")"'"'
    json+=',"status":"'"$(export_json_escape "${ORCH_ITEM_STATUSES[$i]}")"'"'
    json+=',"line":'"${ORCH_ITEM_LINES[$i]}"
    json+=',"dependencies":['"$(export_deps_elements "${ORCH_ITEM_DEPS[$i]}")"']'
    json+=',"packages":'"$(export_string_or_null "$pkgs")"
    json+='}'
  done

  if [[ -n "$cur_key" ]]; then
    json+=']}'
  fi
  json+=']}'

  printf '%s\n' "$json"
}
