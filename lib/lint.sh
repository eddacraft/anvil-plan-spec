#!/usr/bin/env bash
#
# Core linting logic
#

# Associative array to store file types for JSON output
declare -A FILE_TYPES

# Determine file type based on path
# Usage: get_file_type "path/to/file.aps.md"
get_file_type() {
  local file="$1"
  local basename
  basename=$(basename "$file")
  local dirname
  dirname=$(dirname "$file")

  # Skip template files
  if [[ "$basename" == .* ]]; then
    echo "template"
    return
  fi

  # Index files
  if [[ "$basename" == "index.aps.md" ]]; then
    echo "index"
    return
  fi

  # Completed-work archive (parallel to index.aps.md)
  if [[ "$basename" == "completed.aps.md" ]]; then
    echo "archive"
    return
  fi

  # Issues tracker files
  if [[ "$basename" == "issues.md" ]]; then
    echo "issues"
    return
  fi

  # Design files (in designs/ directory)
  if [[ "$basename" == *.design.md && ( "$file" == *"/designs/"* || "$file" == designs/* ) ]]; then
    echo "design"
    return
  fi

  # Actions files
  if [[ "$file" == *"/execution/"* && "$basename" == *.actions.md ]]; then
    echo "actions"
    return
  fi

  # Module files (in modules/ directory)
  if [[ "$dirname" == *"/modules" || "$dirname" == *"/modules/"* ]]; then
    echo "module"
    return
  fi

  # Default to simple for other .aps.md files
  if [[ "$basename" == *.aps.md ]]; then
    echo "simple"
    return
  fi

  echo "unknown"
}

# Find all APS files in a directory
# Usage: find_aps_files "directory"
find_aps_files() {
  local dir="$1"

  # Find .aps.md, .actions.md, .design.md, and issues.md files, excluding dotfiles
  find "$dir" -type f \( -name "*.aps.md" -o -name "*.actions.md" -o -name "*.design.md" -o -name "issues.md" \) ! -name ".*" 2>/dev/null | sort
}

# Lint a single file
# Usage: lint_file "path/to/file.aps.md"
lint_file() {
  local file="$1"
  local file_type
  file_type=$(get_file_type "$file")

  FILE_TYPES["$file"]="$file_type"
  ((TOTAL_FILES++)) || true

  case "$file_type" in
    index)
      lint_index "$file"
      ;;
    module|simple)
      lint_module "$file"
      ;;
    issues)
      lint_issues "$file"
      ;;
    design)
      lint_design "$file"
      ;;
    actions)
      # Actions files have minimal validation for now
      # Could add checkpoint format validation later
      return 0
      ;;
    archive)
      # Completed-work archive — markdown-only, no module structure expected
      return 0
      ;;
    template)
      # Skip templates
      return 0
      ;;
    *)
      add_result "$file" "warning" "W000" "Unknown file type, skipping validation"
      return 0
      ;;
  esac
}

# Main lint command
cmd_lint() {
  local target="plans"
  local json_output=false

  # Parse arguments
  while [[ $# -gt 0 ]]; do
    case $1 in
      --json)
        json_output=true
        shift
        ;;
      --help|-h)
        cat <<EOF
Usage: aps lint [file|dir] [options]

Validate APS documents against expected structure.

Arguments:
  file|dir    File or directory to lint (default: plans/)

Options:
  --json      Output results in JSON format
  --help      Show this help

Exit codes:
  0    No errors (may include warnings)
  1    One or more errors found

Examples:
  aps lint                        # Lint plans/ directory
  aps lint plans/index.aps.md     # Lint specific file
  aps lint plans/modules/         # Lint all modules
  aps lint . --json               # JSON output
EOF
        return 0
        ;;
      -*)
        error "Unknown option: $1"
        return 1
        ;;
      *)
        target="$1"
        shift
        ;;
    esac
  done

  # Validate target exists
  if [[ ! -e "$target" ]]; then
    error "Path not found: $target"
    return 1
  fi

  # Collect files to lint
  local files=()
  if [[ -f "$target" ]]; then
    files+=("$target")
  else
    while IFS= read -r file; do
      files+=("$file")
    done < <(find_aps_files "$target")

    # Also scan designs/ when the target is specifically plans/
    # (find_aps_files already recurses, so this only adds the sibling designs/ dir)
    if [[ "$target" == "plans" || "$target" == "plans/" ]]; then
      if [[ -d "designs" ]]; then
        while IFS= read -r file; do
          files+=("$file")
        done < <(find_aps_files "designs")
      fi
    fi
  fi

  if [[ ${#files[@]} -eq 0 ]]; then
    error "No APS files found in: $target"
    return 1
  fi

  # Lint each file
  for file in "${files[@]}"; do
    lint_file "$file" || true  # Continue on errors, we track them in FILE_RESULTS

    # Mark file as valid if no issues were added
    local has_issues=false
    for result in "${FILE_RESULTS[@]}"; do
      if [[ "$result" == "$file|"* ]]; then
        has_issues=true
        break
      fi
    done

    if [[ "$has_issues" == false ]]; then
      FILE_RESULTS+=("$file|ok|OK||")
    fi
  done

  # Output results
  if [[ "$json_output" == true ]]; then
    print_json_results
  else
    print_text_results
  fi

  # Exit with error if any errors found
  [[ $TOTAL_ERRORS -gt 0 ]] && return 1
  return 0
}
