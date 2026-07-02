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

# Cross-file ID index: work item and decision IDs from the whole plan tree.
# W003 resolves dependencies against this when the in-file check misses.
APS_TREE_IDS=""

# Usage: build_id_index file1 [file2 ...]
build_id_index() {
  # Fence-aware: IDs inside ``` / ~~~ code blocks are examples, not
  # definitions — indexing them would let a fake ID in a snippet vouch for
  # a genuinely missing dependency.
  APS_TREE_IDS=$(awk '
    FNR == 1 { fence = 0 }
    /^(```|~~~)/ { fence = !fence; next }
    fence { next }
    # Work item headers: ### AUTH-001: title
    match($0, /^### [A-Za-z]+-[0-9]+:/) {
      id = substr($0, 5, RLENGTH - 5)
      print id
    }
    # Decision entries: - **D-026:** text
    match($0, /^- \*\*D-[0-9]+:/) {
      id = substr($0, 5, RLENGTH - 5)
      print id
    }
  ' "$@" 2>/dev/null | sort -u | tr '\n' ' ')
  return 0
}

# Lexically normalise a path: collapse '.' and '..' segments without touching
# the filesystem, preserving whether the path is relative or absolute. Used so
# child-plan links (joined onto a parent dir, e.g. "plans/../packages/...")
# dedupe cleanly against recursively-found paths and keep tidy relative output.
# Usage: normalize_path "a/b/../c"
normalize_path() {
  local path="$1"
  local abs=false
  [[ "$path" == /* ]] && abs=true
  local out=() part
  local IFS=/
  for part in $path; do
    case "$part" in
      ''|.) ;;
      ..)
        if [[ ${#out[@]} -gt 0 && "${out[-1]}" != ".." ]]; then
          unset 'out[-1]'
        elif [[ "$abs" == false ]]; then
          out+=("..")
        fi
        ;;
      *) out+=("$part") ;;
    esac
  done
  local result="${out[*]}"
  if [[ "$abs" == true ]]; then
    echo "/$result"
  else
    echo "${result:-.}"
  fi
}

# Emit child-plan index paths declared in a parent index's "## Child Plans"
# section (MONO-002). Each list item links a child index.aps.md relative to the
# parent; paths are resolved against the parent dir and normalised.
# Usage: resolve_child_plan_links "path/to/index.aps.md"
resolve_child_plan_links() {
  local index_file="$1"
  local dir
  dir=$(dirname "$index_file")
  awk '
    /^## / { in_section = ($0 ~ /^## Child Plans[[:space:]]*$/) ? 1 : 0; next }
    in_section && match($0, /\]\([^)]+\)/) {
      print substr($0, RSTART + 2, RLENGTH - 3)
    }
  ' "$index_file" 2>/dev/null | while IFS= read -r link; do
    [[ -z "$link" ]] && continue
    local resolved
    resolved=$(normalize_path "$dir/$link")
    [[ -f "$resolved" ]] && echo "$resolved"
  done
}

# Expand a file list in place by following ## Child Plans links transitively.
# Reads and rewrites the global `files` array; deduped on normalised paths.
# Usage: expand_child_plans
expand_child_plans() {
  local -A seen=()
  local f key
  local result=()
  for f in "${files[@]}"; do
    key=$(normalize_path "$f")
    if [[ -z "${seen[$key]:-}" ]]; then
      seen["$key"]=1
      result+=("$f")
    fi
  done

  local queue=("${result[@]}")
  while [[ ${#queue[@]} -gt 0 ]]; do
    local current="${queue[0]}"
    queue=("${queue[@]:1}")
    [[ "$(basename "$current")" == "index.aps.md" ]] || continue
    local child_index
    while IFS= read -r child_index; do
      [[ -z "$child_index" ]] && continue
      local cf
      while IFS= read -r cf; do
        key=$(normalize_path "$cf")
        if [[ -z "${seen[$key]:-}" ]]; then
          seen["$key"]=1
          result+=("$cf")
          queue+=("$cf")
        fi
      done < <(find_aps_files "$(dirname "$child_index")")
    done < <(resolve_child_plan_links "$current")
  done

  files=("${result[@]}")
}

# Per-child work-item ID registry for cross-tree (`<name>:<ID>`) resolution.
# Keyed by path-derived child name (the segment above a child's plans/ dir).
# W003 reads this to validate prefixed deps; empty when no child trees are in
# scope (a child linted alone), which is how isolated cross-tree refs stay
# silent. (MONO-002)
declare -gA APS_CHILD_IDS=()

# Build APS_CHILD_IDS from the final `files` list. For each plan root present
# (an index.aps.md), derive its child name and collect the work-item IDs
# defined under that root.
# Usage: build_child_registry
build_child_registry() {
  APS_CHILD_IDS=()
  local f root name ids rf
  for f in "${files[@]}"; do
    [[ "$(basename "$f")" == "index.aps.md" ]] || continue
    root=$(dirname "$f")                       # .../<name>/plans
    name=$(basename "$(dirname "$root")")      # <name>
    [[ -n "$name" && "$name" != "." && "$name" != "/" ]] || continue
    # Collect this root's own files (index + modules), tolerating an absent
    # modules/ dir; keep the pipeline from tripping `set -eo pipefail`.
    local root_files=()
    while IFS= read -r rf; do
      [[ -n "$rf" ]] && root_files+=("$rf")
    done < <(find "$root" -maxdepth 2 -type f -name '*.aps.md' 2>/dev/null || true)
    [[ ${#root_files[@]} -gt 0 ]] || continue
    ids=$(awk '
      FNR == 1 { fence = 0 }
      /^(```|~~~)/ { fence = !fence; next }
      fence { next }
      match($0, /^### [A-Za-z]+-[0-9]+:/) { print substr($0, 5, RLENGTH - 5) }
    ' "${root_files[@]}" 2>/dev/null | sort -u | tr '\n' ' ' || true)
    APS_CHILD_IDS["$name"]="${APS_CHILD_IDS[$name]:-} $ids"
  done
}

# W020: the same work-item ID is defined in more than one child tree. Per D-002
# IDs are bare per tree and may legitimately collide across trees, so this is a
# warning (each tree stays independently valid) — but it makes cross-tree
# references ambiguous, so surface it. Only fires when a federation parent
# (a `## Child Plans` index) is in scope. (MONO-002)
check_cross_tree_collisions() {
  [[ ${#APS_CHILD_IDS[@]} -gt 0 ]] || return 0

  # Attach warnings to the federation parent (the index declaring children).
  local parent_file="" f
  for f in "${files[@]}"; do
    [[ "$(basename "$f")" == "index.aps.md" ]] || continue
    if grep -qE '^## Child Plans[[:space:]]*$' "$f" 2>/dev/null; then
      parent_file="$f"
      break
    fi
  done
  [[ -n "$parent_file" ]] || return 0

  # Map each ID to the distinct child names that define it.
  local -A id_owners=()
  local name id
  for name in "${!APS_CHILD_IDS[@]}"; do
    for id in ${APS_CHILD_IDS[$name]}; do
      [[ -n "$id" ]] || continue
      case " ${id_owners[$id]:-} " in
        *" $name "*) ;;
        *) id_owners["$id"]="${id_owners[$id]:-} $name" ;;
      esac
    done
  done

  # Emit deterministically: sort the colliding IDs, and sort each ID's owner
  # names, so all three linters (bash/PowerShell/Rust) produce byte-identical
  # W020 lines regardless of hash-iteration order (D-039).
  local sorted_id owners_sorted
  while IFS= read -r sorted_id; do
    [[ -n "$sorted_id" ]] || continue
    owners_sorted=$(printf '%s\n' ${id_owners[$sorted_id]} | sort | tr '\n' ' ' | sed 's/ *$//')
    if [[ $(echo "$owners_sorted" | wc -w) -gt 1 ]]; then
      add_result "$parent_file" "warning" "W020" \
        "Work-item ID '$sorted_id' defined in multiple child trees: $owners_sorted"
    fi
  done < <(printf '%s\n' "${!id_owners[@]}" | sort)
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
  local target=""
  local json_output=false
  local strict=false

  # Parse arguments
  while [[ $# -gt 0 ]]; do
    case $1 in
      --json)
        json_output=true
        shift
        ;;
      --strict)
        strict=true
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
  --strict    Fail on a cli_version mismatch with .aps/config.yml
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

  # Default to the discovered plans_dir (INSTALL-016); explicit target wins.
  if [[ -z "$target" ]]; then
    target="$(aps_default_plans)"
    aps_check_cli_version "$strict"
  fi

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

    # MONO-002: follow ## Child Plans links so a federated parent root validates
    # its child plan trees as one plan (children live outside the parent dir).
    expand_child_plans
  fi

  if [[ ${#files[@]} -eq 0 ]]; then
    error "No APS files found in: $target"
    return 1
  fi

  # Build the cross-file ID index. For a single-file target, widen the index
  # to the surrounding plan tree so cross-module dependencies still resolve.
  local index_files=("${files[@]}")
  if [[ -f "$target" ]]; then
    local tdir troot
    tdir=$(cd "$(dirname "$target")" && pwd)
    # Climb out of modules/ (including nested subdirectories) to the plan root
    case "$tdir" in
      */modules|*/modules/*) troot="${tdir%/modules*}" ;;
      *) troot="$tdir" ;;
    esac
    while IFS= read -r file; do
      index_files+=("$file")
    done < <(find_aps_files "$troot")
  fi
  build_id_index "${index_files[@]}"
  build_child_registry
  check_cross_tree_collisions

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
