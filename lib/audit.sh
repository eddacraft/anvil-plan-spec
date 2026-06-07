#!/usr/bin/env bash
#
# APS completion audit (DOGFOOD-002)
#
# Formalizes the anvil-001 audit pattern: verify that recorded plan state
# matches reality. Finding codes:
#
#   A001  overstated   Complete item whose Validation command fails
#   A002  understated  Draft item whose Files already exist with content
#   A003  stale        Ready item in a module with no recent Last reviewed
#   A004  broken-link  Index module link pointing to a non-existent file
#
# Complete items without a runnable Validation command report PARTIAL —
# they are not failures, but their completion cannot be machine-verified.

# Findings (parallel arrays)
AUDIT_CODES=()
AUDIT_ITEMS=()
AUDIT_MODULES=()
AUDIT_DETAILS=()
# Verification results for Complete items: "ID|PASS/FAIL/PARTIAL|detail"
AUDIT_VERIFICATIONS=()

audit_reset_state() {
  AUDIT_CODES=()
  AUDIT_ITEMS=()
  AUDIT_MODULES=()
  AUDIT_DETAILS=()
  AUDIT_VERIFICATIONS=()
}

audit_add_finding() {
  AUDIT_CODES+=("$1")
  AUDIT_ITEMS+=("$2")
  AUDIT_MODULES+=("$3")
  AUDIT_DETAILS+=("$4")
}

audit_json_escape() {
  local s="$1"
  s="${s//\\/\\\\}"
  s="${s//\"/\\\"}"
  s="${s//$'\n'/\\n}"
  s="${s//$'\t'/\\t}"
  s="${s//$'\r'/\\r}"
  # Remaining C0 control characters are illegal in JSON strings — drop them
  printf '%s' "$s" | tr -d '\000-\010\013\014\016-\037'
}

# Extract the first backtick-quoted command from a Validation field value.
# Prose validations ("Manual verification") yield nothing.
audit_extract_command() {
  printf '%s' "$1" | grep -oE '`[^`]+`' | head -1 | tr -d '`' || true
}

# Last reviewed date from a module file (empty when absent)
audit_last_reviewed() {
  grep -m1 -oE '^\*\*Last reviewed:\*\* *[0-9]{4}-[0-9]{2}-[0-9]{2}' "$1" 2>/dev/null \
    | grep -oE '[0-9]{4}-[0-9]{2}-[0-9]{2}' || true
}

audit_date_age_days() {
  local d="$1"
  local then_epoch now_epoch
  then_epoch=$(date -d "$d" +%s 2>/dev/null \
    || date -j -f "%Y-%m-%d" "$d" +%s 2>/dev/null) || { echo ""; return 0; }
  now_epoch=$(date +%s)
  echo $(( (now_epoch - then_epoch) / 86400 ))
}

# Audit one Complete item: run its Validation command (unless --no-run).
# A missing Validation field is PARTIAL (unverifiable), not a finding —
# lint rule W018 owns that warning.
audit_complete_item() {
  local i="$1" run_validation="$2" timeout_secs="$3"
  local id="${ORCH_ITEM_IDS[$i]}"
  local module="${ORCH_ITEM_MODULES[$i]}"
  local content
  content=$(orch_item_content "${ORCH_ITEM_FILES[$i]}" "${ORCH_ITEM_LINES[$i]}")

  local validation
  validation=$(orch_field_value "$content" "Validation")
  if [[ -z "$validation" ]]; then
    AUDIT_VERIFICATIONS+=("$id|PARTIAL|no Validation field")
    return 0
  fi

  local cmd
  cmd=$(audit_extract_command "$validation")
  cmd="${cmd//$'\r'/}"
  if [[ -z "$cmd" ]]; then
    AUDIT_VERIFICATIONS+=("$id|PARTIAL|Validation is not a runnable command")
    return 0
  fi

  # Backticks in Validation prose are often paths or examples, not commands.
  # Only execute when the first word resolves to something runnable. This is
  # a prose-vs-command heuristic, NOT a security control: the command string
  # executes with full shell semantics (pipes, &&, subshells are legitimate
  # in validation commands). The trust boundary is the plan file itself —
  # see --help and docs. type -P does a PATH-only lookup so functions and
  # builtins inherited from this script cannot vouch for plan content.
  local first_word rest
  read -r first_word rest <<< "$cmd"
  if ! type -P "$first_word" > /dev/null 2>&1 \
    && ! [[ -f "$first_word" && -x "$first_word" ]]; then
    AUDIT_VERIFICATIONS+=("$id|PARTIAL|command not found: $first_word")
    return 0
  fi

  if [[ "$run_validation" != "true" ]]; then
    AUDIT_VERIFICATIONS+=("$id|PARTIAL|not run (--no-run)")
    return 0
  fi

  local rc=0
  if command -v timeout > /dev/null 2>&1; then
    timeout -k 5 "$timeout_secs" bash -c "$cmd" > /dev/null 2>&1 || rc=$?
  else
    bash -c "$cmd" > /dev/null 2>&1 || rc=$?
  fi

  if [[ $rc -eq 0 ]]; then
    AUDIT_VERIFICATIONS+=("$id|PASS|$cmd")
  elif [[ $rc -eq 124 ]]; then
    AUDIT_VERIFICATIONS+=("$id|FAIL|timed out after ${timeout_secs}s: $cmd")
    audit_add_finding "A001" "$id" "$module" "overstated: Validation timed out: $cmd"
  else
    AUDIT_VERIFICATIONS+=("$id|FAIL|$cmd")
    audit_add_finding "A001" "$id" "$module" "overstated: Validation failed: $cmd"
  fi
}

# Audit one Draft item: flag when its Files already exist with content.
# Relative paths resolve against the plan root's parent (the repo root in
# the conventional plans/ layout), so the audit works from any CWD.
audit_draft_item() {
  local i="$1" repo_root="$2"
  local id="${ORCH_ITEM_IDS[$i]}"
  local module="${ORCH_ITEM_MODULES[$i]}"
  local content
  content=$(orch_item_content "${ORCH_ITEM_FILES[$i]}" "${ORCH_ITEM_LINES[$i]}")

  local files_field
  files_field=$(orch_field_value "$content" "Files")
  [[ -n "$files_field" ]] || return 0

  local existing=""
  local path resolved candidate
  while IFS= read -r path; do
    path=$(orch_trim "$path")
    path="${path#- }"   # tolerate bullet-list notation in Files fields
    [[ -n "$path" ]] || continue
    [[ "$path" == /* ]] && resolved="$path" || resolved="$repo_root/$path"
    # Expand globs; literal paths pass through compgen unchanged
    while IFS= read -r candidate; do
      [[ -n "$candidate" ]] || continue
      # Only regular files with content count as "substantive"
      if [[ -s "$candidate" ]]; then
        existing+="${existing:+, }$path"
        break
      fi
    done < <(compgen -G "$resolved" 2>/dev/null || true)
  done < <(printf '%s\n' "$files_field" | tr ',' '\n')

  if [[ -n "$existing" ]]; then
    audit_add_finding "A002" "$id" "$module" "understated: Draft but files exist: $existing"
  fi
}

# Audit one Ready item: flag when its module review date is missing/stale
audit_ready_item() {
  local i="$1" stale_days="$2"
  local id="${ORCH_ITEM_IDS[$i]}"
  local module="${ORCH_ITEM_MODULES[$i]}"
  local file="${ORCH_ITEM_FILES[$i]}"

  # Items default to Ready when unmarked; inside a Draft/Blocked module they
  # are not actionable yet, so staleness is the module's concern, not theirs.
  local module_status="${ORCH_MODULE_STATUSES[$module]:-}"
  case "$module_status" in
    Ready|"In Progress") ;;
    *) return 0 ;;
  esac

  local reviewed
  reviewed=$(audit_last_reviewed "$file")
  if [[ -z "$reviewed" ]]; then
    audit_add_finding "A003" "$id" "$module" "stale: Ready item in module with no Last reviewed field"
    return 0
  fi

  local age
  age=$(audit_date_age_days "$reviewed")
  if [[ -n "$age" ]] && (( age > stale_days )); then
    audit_add_finding "A003" "$id" "$module" "stale: module last reviewed $reviewed (${age} days ago, threshold ${stale_days})"
  fi
}

# Audit index links (same contract as lint E012)
audit_index_links() {
  local plan_root="$1"
  local index_file="$plan_root/index.aps.md"
  [[ -f "$index_file" ]] || return 0

  local dir
  dir=$(dirname "$index_file")

  local line_num target
  while IFS=: read -r line_num target; do
    [[ -n "$target" ]] || continue
    # Skip pure anchors and any URI scheme (http, mailto, file, vscode, ...)
    [[ "$target" == \#* ]] && continue
    [[ "$target" =~ ^[A-Za-z][A-Za-z0-9+.-]*: ]] && continue
    target="${target%%#*}"
    [[ -n "$target" ]] || continue
    if [[ ! -e "$dir/$target" ]]; then
      audit_add_finding "A004" "index" "-" "broken-link: $target (line $line_num)"
    fi
  done < <(awk '
    /^## Modules/ { in_mod=1; next }
    in_mod && /^## /  { in_mod=0 }
    in_mod {
      line = $0
      while (match(line, /\]\([^)]+\)/)) {
        target = substr(line, RSTART+2, RLENGTH-3)
        sub(/[ \t]+["'\''].*$/, "", target)   # strip markdown link titles
        print NR ":" target
        line = substr(line, RSTART+RLENGTH)
      }
    }
  ' "$index_file")
}

audit_print_text() {
  local audited="$1"

  echo "APS Audit"
  echo ""

  if [[ ${#AUDIT_VERIFICATIONS[@]} -gt 0 ]]; then
    echo "Complete-item verification:"
    local entry
    for entry in "${AUDIT_VERIFICATIONS[@]}"; do
      IFS='|' read -r id result detail <<< "$entry"
      printf '  %-12s %-8s %s\n' "$id" "$result" "$detail"
    done
    echo ""
  fi

  if [[ ${#AUDIT_CODES[@]} -eq 0 ]]; then
    echo "Findings: none ($audited items audited)"
    return 0
  fi

  echo "Findings:"
  local i
  for i in "${!AUDIT_CODES[@]}"; do
    printf '  %s  %-12s %s\n' "${AUDIT_CODES[$i]}" "${AUDIT_ITEMS[$i]}" "${AUDIT_DETAILS[$i]}"
  done
  echo ""
  echo "Findings: ${#AUDIT_CODES[@]} ($audited items audited)"
}

audit_print_json() {
  local audited="$1"

  echo "{"
  echo "  \"summary\": {"
  echo "    \"items_audited\": $audited,"
  echo "    \"findings\": ${#AUDIT_CODES[@]}"
  echo "  },"

  echo "  \"verifications\": ["
  local first=true entry
  for entry in "${AUDIT_VERIFICATIONS[@]}"; do
    IFS='|' read -r id result detail <<< "$entry"
    [[ "$first" == true ]] && first=false || echo ","
    printf '    {"item": "%s", "result": "%s", "detail": "%s"}' \
      "$(audit_json_escape "$id")" "$(audit_json_escape "$result")" "$(audit_json_escape "$detail")"
  done
  [[ "$first" == true ]] || echo ""
  echo "  ],"

  echo "  \"findings\": ["
  first=true
  local i
  for i in "${!AUDIT_CODES[@]}"; do
    [[ "$first" == true ]] && first=false || echo ","
    printf '    {"code": "%s", "item": "%s", "module": "%s", "detail": "%s"}' \
      "${AUDIT_CODES[$i]}" \
      "$(audit_json_escape "${AUDIT_ITEMS[$i]}")" \
      "$(audit_json_escape "${AUDIT_MODULES[$i]}")" \
      "$(audit_json_escape "${AUDIT_DETAILS[$i]}")"
  done
  [[ "$first" == true ]] || echo ""
  echo "  ]"
  echo "}"
}

cmd_audit() {
  local plan_root="plans"
  local module_filter=""
  local json_output=false
  local run_validation=true
  local stale_days="${APS_STALE_DAYS:-60}"
  local timeout_secs="${APS_AUDIT_TIMEOUT:-60}"

  while [[ $# -gt 0 ]]; do
    case "$1" in
      --plans)
        plan_root="${2:-}"
        [[ -n "$plan_root" ]] || { error "--plans requires a directory"; return 1; }
        shift 2
        ;;
      --json)
        json_output=true
        shift
        ;;
      --no-run)
        run_validation=false
        shift
        ;;
      --stale-days)
        stale_days="${2:-}"
        [[ "$stale_days" =~ ^[0-9]+$ ]] || { error "--stale-days requires a number"; return 1; }
        shift 2
        ;;
      --help|-h)
        cat <<EOF
Usage: aps audit [module] [options]

Audit plan state against reality (the anvil-001 completion audit pattern).

Checks:
  A001  Complete items whose Validation command fails (overstated)
  A002  Draft items whose Files already exist with content (understated)
  A003  Ready items in modules with no recent Last reviewed (stale)
  A004  Index module links pointing to non-existent files (broken-link)

Arguments:
  module    Optional module ID or file name to scope the audit

Options:
  --plans DIR      Plan root directory (default: plans)
  --json           Output results in JSON format
  --no-run         Do not execute Validation commands (verification reports
                   PARTIAL; no A001 findings are produced)
  --stale-days N   Staleness threshold in days (default: 60)
  --help           Show this help

Note: by default the audit EXECUTES backtick Validation commands found in
Complete work items. Only run it on plans you trust.

Exit codes:
  0    No findings
  1    One or more findings
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

  if [[ ! -d "$plan_root" ]]; then
    error "Path not found: $plan_root"
    return 1
  fi

  # Env-supplied values bypass flag validation — guard them here so a bad
  # value degrades to the default instead of corrupting every verdict
  # (a non-numeric timeout makes `timeout` exit 125, which reads as FAIL)
  if ! [[ "$stale_days" =~ ^[0-9]+$ ]]; then
    warn "APS_STALE_DAYS must be a number; using 60"
    stale_days=60
  fi
  if ! [[ "$timeout_secs" =~ ^[0-9]+$ ]]; then
    warn "APS_AUDIT_TIMEOUT must be a number; using 60"
    timeout_secs=60
  fi

  if [[ "$run_validation" == "true" ]]; then
    warn "executing Validation commands from plan files (use --no-run to skip)"
  fi

  audit_reset_state
  orch_reset_state
  orch_load_work_items "$plan_root" true || {
    error "No modules directory under: $plan_root"
    return 1
  }

  local repo_root
  repo_root=$(dirname "$plan_root")

  local audited=0
  local i
  for i in "${!ORCH_ITEM_IDS[@]}"; do
    orch_item_matches_module "$i" "$module_filter" || continue

    case "${ORCH_ITEM_STATUSES[$i]}" in
      Complete)
        audit_complete_item "$i" "$run_validation" "$timeout_secs"
        ((audited++)) || true
        ;;
      Draft)
        audit_draft_item "$i" "$repo_root"
        ((audited++)) || true
        ;;
      Ready)
        audit_ready_item "$i" "$stale_days"
        ((audited++)) || true
        ;;
    esac
  done

  # Index link integrity (skip when scoped to a single module)
  if [[ -z "$module_filter" ]]; then
    audit_index_links "$plan_root"
  fi

  if [[ "$json_output" == true ]]; then
    audit_print_json "$audited"
  else
    audit_print_text "$audited"
  fi

  [[ ${#AUDIT_CODES[@]} -gt 0 ]] && return 1
  return 0
}
