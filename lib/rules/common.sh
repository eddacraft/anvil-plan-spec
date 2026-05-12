#!/usr/bin/env bash
#
# Common validation helpers
#

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
    status_col && /^\|[^-]/ && !/^\| *ID *\|/ {
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
