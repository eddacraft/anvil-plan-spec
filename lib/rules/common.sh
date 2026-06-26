#!/usr/bin/env bash
#
# Common validation helpers
#

# --- Project config discovery (INSTALL-016) ---
#
# The global `aps` binary discovers the nearest `.aps/config.yml` by walking up
# from the current directory, then defaults `--plans` to its `plans_dir` and
# checks the project's `cli_version` pin. Explicit flags and APS_PLANS override.

# Walk up from $1 (default cwd) for a directory containing .aps/config.yml.
# Echoes that directory and returns 0, or returns 1 when none is found.
aps_find_config() {
  local dir="${1:-$PWD}"
  while [[ -n "$dir" && "$dir" != "/" ]]; do
    [[ -f "$dir/.aps/config.yml" ]] && { echo "$dir"; return 0; }
    dir="$(dirname "$dir")"
  done
  [[ -f "/.aps/config.yml" ]] && { echo "/"; return 0; }
  return 1
}

# Read a top-level scalar key from a config file (ignores indented keys).
# Usage: aps_config_get <file> <key>
aps_config_get() {
  local file="$1" key="$2"
  [[ -f "$file" ]] || return 1
  sed -n "s/^${key}:[[:space:]]*//p" "$file" | head -1 | tr -d "\"'" | sed 's/[[:space:]]*$//'
}

# Resolve the effective plans directory:
#   APS_PLANS (MCP/manual override) > discovered plans_dir > plans/
aps_default_plans() {
  if [[ -n "${APS_PLANS:-}" ]]; then
    echo "${APS_PLANS%/}"
    return 0
  fi
  local cfgdir plans
  if cfgdir="$(aps_find_config)"; then
    plans="$(aps_config_get "$cfgdir/.aps/config.yml" plans_dir)"
    if [[ -n "$plans" ]]; then
      plans="${plans%/}"
      if [[ "$cfgdir" == "$PWD" ]]; then
        echo "$plans"
      else
        echo "${cfgdir%/}/$plans"
      fi
      return 0
    fi
  fi
  echo "plans"
}

# Warn when the project's cli_version pin differs from this CLI. Under strict
# mode (arg "true" or APS_STRICT=1) a mismatch exits non-zero for CI.
aps_check_cli_version() {
  local strict="${1:-${APS_STRICT:-false}}"
  [[ "$strict" == "1" ]] && strict=true
  local cfgdir pin
  cfgdir="$(aps_find_config)" || return 0
  pin="$(aps_config_get "$cfgdir/.aps/config.yml" cli_version)"
  [[ -z "$pin" ]] && return 0
  if [[ "$pin" != "${APS_CLI_VERSION:-0.4.0}" ]]; then
    warn "project pins cli_version $pin but this CLI is ${APS_CLI_VERSION:-0.4.0}"
    if [[ "$strict" == true ]]; then
      error "cli_version mismatch under --strict"
      exit 1
    fi
  fi
  return 0
}

# Check if a section exists in the file
# Usage: has_section "file" "## Section Name"
has_section() {
  local file="$1"
  local section="$2"
  grep -q "^${section}$" "$file" 2>/dev/null
}

# Check if file has a metadata table (| ID | ... |)
# Usage: has_metadata_table "file"
has_metadata_table() {
  local file="$1"
  # Look for a table with ID column in first few lines
  head -20 "$file" | grep -qE '^\| *ID *\|'
}

# Get section content (lines between this section and next ## heading)
# Usage: get_section_content "file" "## Section Name"
get_section_content() {
  local file="$1"
  local section="$2"

  awk -v section="$section" '
    $0 == section { found=1; next }
    found && /^## / { exit }
    found { print }
  ' "$file"
}

# Check if section has non-empty content (not just whitespace/comments)
# Usage: section_has_content "file" "## Section Name"
section_has_content() {
  local file="$1"
  local section="$2"
  local content
  content=$(get_section_content "$file" "$section")

  # Remove HTML comments, blank lines, and check if anything remains
  echo "$content" | grep -vE '^[[:space:]]*$|^[[:space:]]*<!--.*-->$|^<!--' | grep -q .
}

# Extract all work item headers (### PREFIX-NNN: ...)
# Usage: get_work_items "file"
get_work_items() {
  local file="$1"
  grep -nE '^### [A-Za-z]+-[0-9]+:' "$file" 2>/dev/null || true
}

# Extract module ID from metadata table
# Usage: get_module_id "file"
get_module_id() {
  local file="$1"
  # Find the header row, skip the separator row (|---|---|), then
  # extract the first cell of the first data row.
  awk -F '|' '
    /^\| *ID *\|/ { found = 1; next }
    found && /^\|[- :|]+$/ { next }                  # skip separator row (|---|---|)
    found && /^\|/ {
      # $2 is the first cell (leading | produces empty $1)
      id = $2
      gsub(/^[[:space:]]+|[[:space:]]+$/, "", id)
      print id
      exit
    }
  ' "$file"
}

# Extract status from metadata table
# Usage: get_status "file"
get_status() {
  local file="$1"
  # Find Status column position and extract value
  awk '
    /^\| *ID *\|/ {
      n = split($0, cols, "|")
      for (i=1; i<=n; i++) {
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

# Get line number of a pattern
# Usage: get_line_number "file" "pattern"
get_line_number() {
  local file="$1"
  local pattern="$2"
  grep -n "$pattern" "$file" 2>/dev/null | head -1 | cut -d: -f1
}
