#!/usr/bin/env bash
#
# Scaffold logic for `aps init`, `aps update`, and `aps migrate`
#

APS_VERSION="${APS_VERSION:-main}"
APS_BASE_URL="https://raw.githubusercontent.com/EddaCraft/anvil-plan-spec/$APS_VERSION"

# Semver of the bash CLI release, stamped into .aps/config.yml as the project
# contract's cli_version (INSTALL-014). The native binary stamps its own crate
# version; this is the bash-fallback equivalent.
APS_CLI_VERSION="${APS_CLI_VERSION:-0.7.0}"

# --- v2 file lists (.aps/ layout) ---

# Plan templates and rules for plans/
V2_PLAN_FILES=(
  "scaffold/plans/aps-rules-v2.md"
  "scaffold/plans/project-context.md"
  "scaffold/plans/issues.md"
  "scaffold/plans/modules/.module.template.md"
  "scaffold/plans/modules/.simple.template.md"
  "scaffold/plans/modules/.index-monorepo.template.md"
  "scaffold/plans/execution/.actions.template.md"
)

# Skill files for .claude/skills/aps-planning/
V2_SKILL_FILES=(
  "scaffold/aps-planning/SKILL.md"
  "scaffold/aps-planning/reference.md"
  "scaffold/aps-planning/examples.md"
)

# Hook scripts for .aps/scripts/
V2_SCRIPT_FILES=(
  "aps-planning/scripts/install-hooks.sh"
  "aps-planning/scripts/init-session.sh"
  "aps-planning/scripts/check-complete.sh"
  "aps-planning/scripts/pre-tool-check.sh"
  "aps-planning/scripts/post-tool-nudge.sh"
  "aps-planning/scripts/enforce-plan-update.sh"
)

# CLI files for .aps/bin/ and .aps/lib/ (bash runtime)
V2_CLI_FILES=(
  "bin/aps"
  "lib/output.sh"
  "lib/lint.sh"
  "lib/orchestrate.sh"
  "lib/audit.sh"
  "lib/export.sh"
  "lib/scaffold.sh"
  "lib/rules/common.sh"
  "lib/rules/module.sh"
  "lib/rules/index.sh"
  "lib/rules/workitem.sh"
  "lib/rules/issues.sh"
  "lib/rules/design.sh"
)

# Agent files (Claude Code)
V2_AGENT_FILES=(
  "scaffold/agents/claude-code/aps-planner.md"
  "scaffold/agents/claude-code/aps-librarian.md"
  "scaffold/agents/claude-code/aps-conductor.md"
)

# --- v1 file lists (backward compat for update) ---

PLAN_FILES=(
  "scaffold/plans/aps-rules.md"
  "scaffold/plans/modules/.module.template.md"
  "scaffold/plans/modules/.simple.template.md"
  "scaffold/plans/modules/.index-monorepo.template.md"
  "scaffold/plans/execution/.actions.template.md"
)

SKILL_FILES=(
  "scaffold/aps-planning/SKILL.md"
  "scaffold/aps-planning/reference.md"
  "scaffold/aps-planning/examples.md"
  "scaffold/aps-planning/hooks.md"
  "scaffold/aps-planning/scripts/install-hooks.sh"
  "scaffold/aps-planning/scripts/init-session.sh"
  "scaffold/aps-planning/scripts/check-complete.sh"
  "scaffold/aps-planning/scripts/pre-tool-check.sh"
  "scaffold/aps-planning/scripts/post-tool-nudge.sh"
  "scaffold/aps-planning/scripts/enforce-plan-update.sh"
)

COMMAND_FILES=(
  "scaffold/commands/plan.md"
  "scaffold/commands/plan-status.md"
)

CLI_FILES=(
  "bin/aps"
  "lib/output.sh"
  "lib/lint.sh"
  "lib/orchestrate.sh"
  "lib/audit.sh"
  "lib/export.sh"
  "lib/scaffold.sh"
  "lib/rules/common.sh"
  "lib/rules/module.sh"
  "lib/rules/index.sh"
  "lib/rules/workitem.sh"
  "lib/rules/issues.sh"
  "lib/rules/design.sh"
)

# Canonical tool identifiers
TOOL_NAMES=("claude-code" "copilot" "codex" "opencode" "grok" "antigravity" "generic")
TOOL_LABELS=("Claude Code" "GitHub Copilot" "Codex" "OpenCode" "Grok" "None / manual only")

# --- Utility functions ---

# Download a file from GitHub (or copy locally if APS_LOCAL is set)
download() {
  local src="$1"
  local dest="$2"

  mkdir -p "$(dirname "$dest")"

  # Local mode: copy from source repo instead of downloading
  if [[ -n "${APS_LOCAL:-}" ]]; then
    local local_path="$APS_LOCAL/$src"
    if [[ -f "$local_path" ]]; then
      cp "$local_path" "$dest"
      return 0
    else
      error "Local file not found: $local_path"
      exit 1
    fi
  fi

  local url="$APS_BASE_URL/$src"
  if ! curl -fsSL "$url" -o "$dest"; then
    error "Failed to download: $url"
    echo "  Check your network and ensure APS_VERSION='$APS_VERSION' is valid." >&2
    exit 1
  fi
}

# Prompt user with a yes/no question. Returns 0 for yes, 1 for no.
ask_yn() {
  local prompt="$1"
  local default="${2:-n}"

  if [[ -t 0 ]]; then
    local yn_hint
    if [[ "$default" == "y" ]]; then yn_hint="Y/n"; else yn_hint="y/N"; fi
    printf "%s [%s] " "$prompt" "$yn_hint"
    read -r answer
    answer="${answer:-$default}"
    [[ "$answer" =~ ^[Yy] ]]
  else
    [[ "$default" == "y" ]]
  fi
}

# Single-select prompt. Returns the 1-based selection number.
prompt_select() {
  local prompt="$1"
  shift
  local options=("$@")
  local count=${#options[@]}

  echo "" >&2
  echo "$prompt" >&2
  echo "" >&2
  for i in "${!options[@]}"; do
    printf "  %d) %s\n" $((i + 1)) "${options[$i]}" >&2
  done
  echo "" >&2

  if [[ -t 0 ]]; then
    while true; do
      printf "Choice [1-%d]: " "$count" >&2
      read -r choice
      if [[ "$choice" =~ ^[0-9]+$ ]] && (( choice >= 1 && choice <= count )); then
        echo "$choice"
        return
      fi
      echo "  Please enter a number between 1 and $count" >&2
    done
  else
    echo "1"
  fi
}

# Multi-select prompt. Returns comma-separated 1-based indices.
prompt_multi() {
  local prompt="$1"
  shift
  local options=("$@")
  local count=${#options[@]}

  echo "" >&2
  echo "$prompt" >&2
  echo "" >&2
  for i in "${!options[@]}"; do
    printf "  %d) %s\n" $((i + 1)) "${options[$i]}" >&2
  done
  echo "" >&2

  if [[ -t 0 ]]; then
    while true; do
      printf "Choice (comma-separated, e.g. 1,2,4): " >&2
      read -r choices
      # Validate all choices
      local valid=true
      IFS=',' read -ra parts <<< "$choices"
      for part in "${parts[@]}"; do
        part="${part// /}"
        if ! [[ "$part" =~ ^[0-9]+$ ]] || (( part < 1 || part > count )); then
          valid=false
          break
        fi
      done
      if $valid && [[ ${#parts[@]} -gt 0 ]]; then
        echo "$choices"
        return
      fi
      echo "  Please enter numbers between 1 and $count, separated by commas" >&2
    done
  else
    echo "1"
  fi
}

# Check if APS hooks are already configured in either settings file
has_aps_hooks() {
  local target="${1:-.}"
  local f
  for f in "$target/.claude/settings.local.json" "$target/.claude/settings.json"; do
    if [[ -f "$f" ]] && grep -q 'aps-planning/scripts\|\.aps/scripts\|\[APS\]' "$f" 2>/dev/null; then
      return 0
    fi
  done
  return 1
}

# Detect v1 layout — require APS-specific markers to avoid false positives
# on repos that happen to have bin/aps for unrelated purposes.
is_v1_layout() {
  local target="${1:-.}"
  local markers=0

  [[ -f "$target/bin/aps" ]]                  && ((markers++))
  [[ -d "$target/aps-planning" ]]             && ((markers++))
  [[ -f "$target/.claude/commands/plan.md" ]] && ((markers++))
  [[ -f "$target/lib/output.sh" ]]            && ((markers++))
  [[ -f "$target/plans/aps-rules.md" ]]       && ((markers++))

  # Require at least 2 markers to confidently identify a v1 install
  (( markers >= 2 ))
}

# Detect v2 layout
is_v2_layout() {
  local target="${1:-.}"
  [[ -f "$target/.aps/config.yml" ]]
}

# Detect monorepo tool
detect_monorepo_tool() {
  local target="${1:-.}"
  if [[ -f "$target/pnpm-workspace.yaml" ]]; then echo "pnpm"
  elif [[ -f "$target/turbo.json" ]]; then echo "turbo"
  elif [[ -f "$target/lerna.json" ]]; then echo "lerna"
  elif [[ -f "$target/nx.json" ]]; then echo "nx"
  else echo ""
  fi
}

# --- Managed skill markers (D-042 / INSTALL-020) ---
#
# Sidecar `.aps-managed.json` written next to each managed skill tree so
# installs and updates can distinguish APS-owned content from user edits.
# The JSON shape, per-file SHA-256 hashes, and bundle digest are
# byte-identical with the Rust implementation (cli/src/managed.rs) and the
# PowerShell port (lib/Scaffold.psm1): any CLI can verify a tree written by
# any other. Phase 1 covers the planning skill (SKILL.md, reference.md,
# examples.md); agent inventory is Phase 3.

APS_MANAGED_MARKER=".aps-managed.json"

# SHA-256 hex digest of a file / of stdin.
managed_sha256_file() {
  if command -v sha256sum >/dev/null 2>&1; then
    sha256sum "$1" | awk '{print $1}'
  elif command -v shasum >/dev/null 2>&1; then
    shasum -a 256 "$1" | awk '{print $1}'
  else
    error "sha256sum or shasum is required for managed skill markers"
    exit 1
  fi
}

managed_sha256_stdin() {
  if command -v sha256sum >/dev/null 2>&1; then
    sha256sum | awk '{print $1}'
  else
    shasum -a 256 | awk '{print $1}'
  fi
}

# Canonical skill payload, downloaded once per run and cleaned up on exit.
# Call outside command substitutions so the cache and trap persist.
APS_SKILL_PAYLOAD_DIR=""

managed_cleanup_payload() {
  [[ -n "$APS_SKILL_PAYLOAD_DIR" ]] && rm -rf "$APS_SKILL_PAYLOAD_DIR"
}

managed_ensure_payload() {
  [[ -n "$APS_SKILL_PAYLOAD_DIR" ]] && return 0
  APS_SKILL_PAYLOAD_DIR="$(mktemp -d "${TMPDIR:-/tmp}/aps-skill-payload.XXXXXX")"
  trap managed_cleanup_payload EXIT
  local f
  for f in "${V2_SKILL_FILES[@]}"; do
    download "$f" "$APS_SKILL_PAYLOAD_DIR/${f#scaffold/aps-planning/}"
  done
}

# Canonical marker JSON for a payload dir. Matches Rust
# SkillManifest::to_json byte-for-byte: camelCase keys, two-space indent,
# files sorted bytewise, trailing newline. bundleDigest is SHA-256 over
# sorted "name=hash\n" lines.
managed_manifest_json() {
  local payload="$1"
  local f rel sorted=()
  while IFS= read -r rel; do sorted+=("$rel"); done < <(
    for f in "${V2_SKILL_FILES[@]}"; do printf '%s\n' "${f#scaffold/aps-planning/}"; done | LC_ALL=C sort
  )
  local material="" hash json_files=()
  for rel in "${sorted[@]}"; do
    hash="$(managed_sha256_file "$payload/$rel")"
    material+="${rel}=${hash}"$'\n'
    json_files+=("    \"$rel\": \"$hash\"")
  done
  local digest
  digest="$(printf '%s' "$material" | managed_sha256_stdin)"
  printf '{\n'
  printf '  "schemaVersion": 1,\n'
  printf '  "kind": "skill",\n'
  printf '  "name": "aps-planning",\n'
  printf '  "cliVersion": "%s",\n' "$APS_CLI_VERSION"
  printf '  "bundleDigest": "%s",\n' "$digest"
  printf '  "files": {\n'
  local i last=$(( ${#json_files[@]} - 1 ))
  for i in "${!json_files[@]}"; do
    if (( i < last )); then
      printf '%s,\n' "${json_files[$i]}"
    else
      printf '%s\n' "${json_files[$i]}"
    fi
  done
  printf '  }\n}\n'
}

# Print "name<TAB>hash" lines from a marker's files map. Fails (exit 1) when
# the marker is not a readable skill manifest — schemaVersion 1, kind
# "skill", and a files map of plain string pairs.
managed_parse_marker() {
  local marker="$1"
  grep -q '"schemaVersion"[[:space:]]*:[[:space:]]*1' "$marker" 2>/dev/null || return 1
  grep -q '"kind"[[:space:]]*:[[:space:]]*"skill"' "$marker" 2>/dev/null || return 1
  grep -q '"files"' "$marker" 2>/dev/null || return 1
  awk '
    /"files"[[:space:]]*:[[:space:]]*\{/ { in_files = 1; next }
    in_files && /\}/ { in_files = 0 }
    in_files {
      line = $0
      if (line ~ /^[[:space:]]*"[^"]+"[[:space:]]*:[[:space:]]*"[^"]*",?[[:space:]]*$/) {
        k = line; sub(/^[[:space:]]*"/, "", k); sub(/".*$/, "", k)
        v = line; sub(/^[^:]*:[[:space:]]*"/, "", v); sub(/",?[[:space:]]*$/, "", v)
        printf "%s\t%s\n", k, v
      } else if (line !~ /^[[:space:]]*$/) {
        bad = 1
      }
    }
    END { exit bad }
  ' "$marker"
}

# Classify a skill dir against the canonical payload:
# absent | fresh | stale | dirty | unmanaged | broken
managed_eval_skill_dir() {
  local skill_dir="$1" payload="$2"
  if [[ ! -d "$skill_dir" ]]; then
    printf 'absent\n'
    return 0
  fi
  local marker="$skill_dir/$APS_MANAGED_MARKER"
  local f rel
  if [[ ! -f "$marker" ]]; then
    for f in "${V2_SKILL_FILES[@]}"; do
      rel="${f#scaffold/aps-planning/}"
      if [[ -f "$skill_dir/$rel" ]]; then
        printf 'unmanaged\n'
        return 0
      fi
    done
    # Empty dir (or only non-skill files) — safe to install.
    printf 'absent\n'
    return 0
  fi
  local entries
  if ! entries="$(managed_parse_marker "$marker")"; then
    printf 'broken\n'
    return 0
  fi
  # Dirty when any tracked file is missing or no longer matches the marker.
  local name hash
  while IFS=$'\t' read -r name hash; do
    [[ -n "$name" ]] || continue
    if [[ ! -f "$skill_dir/$name" ]] || [[ "$(managed_sha256_file "$skill_dir/$name")" != "$hash" ]]; then
      printf 'dirty\n'
      return 0
    fi
  done <<< "$entries"
  # Canonical serialisation makes equivalence a byte comparison (all three
  # CLIs write the same shape). A semantically-equivalent but non-canonical
  # marker classifies as stale and converges to canonical on update.
  if [[ "$(cat "$marker")" == "$(managed_manifest_json "$payload")" ]]; then
    printf 'fresh\n'
  else
    printf 'stale\n'
  fi
}

managed_files_match() {
  local skill_dir="$1" payload="$2" f rel
  for f in "${V2_SKILL_FILES[@]}"; do
    rel="${f#scaffold/aps-planning/}"
    cmp -s "$payload/$rel" "$skill_dir/$rel" || return 1
  done
  return 0
}

managed_copy_files() {
  local skill_dir="$1" payload="$2" f rel
  mkdir -p "$skill_dir"
  for f in "${V2_SKILL_FILES[@]}"; do
    rel="${f#scaffold/aps-planning/}"
    cp "$payload/$rel" "$skill_dir/$rel"
  done
}

# Reconcile one skill tree with managed-marker safety. Prints the outcome:
# added | updated | unchanged | adopted | dirty-skipped | unmanaged-skipped |
# broken-skipped (mirrors Rust reconcile_managed_skill).
managed_reconcile_skill() {
  local skill_dir="$1"
  managed_ensure_payload
  local payload="$APS_SKILL_PAYLOAD_DIR"
  local state
  state="$(managed_eval_skill_dir "$skill_dir" "$payload")"
  case "$state" in
    absent)
      managed_copy_files "$skill_dir" "$payload"
      managed_manifest_json "$payload" > "$skill_dir/$APS_MANAGED_MARKER"
      printf 'added\n'
      ;;
    fresh) printf 'unchanged\n' ;;
    stale)
      # Content may already match the payload (only the marker drifted) —
      # avoid needless rewrites/mtime churn.
      managed_files_match "$skill_dir" "$payload" || managed_copy_files "$skill_dir" "$payload"
      managed_manifest_json "$payload" > "$skill_dir/$APS_MANAGED_MARKER"
      printf 'updated\n'
      ;;
    dirty) printf 'dirty-skipped\n' ;;
    unmanaged)
      if managed_files_match "$skill_dir" "$payload"; then
        managed_manifest_json "$payload" > "$skill_dir/$APS_MANAGED_MARKER"
        printf 'adopted\n'
      else
        printf 'unmanaged-skipped\n'
      fi
      ;;
    broken) printf 'broken-skipped\n' ;;
  esac
}

# Reconcile + user-facing messaging for one skill tree.
v2_reconcile_skill_tree() {
  local skill_dir="$1" label="$2" result
  managed_ensure_payload
  result="$(managed_reconcile_skill "$skill_dir")"
  case "$result" in
    dirty-skipped)
      warn "$label has local edits — left untouched."
      echo "  Restore the files (or remove the directory) and re-run to refresh." >&2
      ;;
    unmanaged-skipped)
      warn "$label exists but was not installed by APS — left untouched."
      echo "  Remove the directory to let APS manage it." >&2
      ;;
    broken-skipped)
      warn "$label has an unreadable $APS_MANAGED_MARKER — left untouched."
      ;;
  esac
}

# --- v2 install functions ---

# Write config.yml
write_config() {
  local target="$1"
  local profile="$2"
  local scope="$3"
  shift 3
  local tools=("$@")

  local config_dir="$target/.aps"
  mkdir -p "$config_dir"

  # Ignore ephemeral CLI-generated context regardless of whether a local CLI
  # was vendored (the global binary writes here too).
  touch "$config_dir/.gitignore"
  grep -qxF 'context/' "$config_dir/.gitignore" || printf 'context/\n' >> "$config_dir/.gitignore"

  local project_type="simple"
  local monorepo_tool="~"
  if [[ "$scope" == "monorepo" || "$scope" == "nested" ]]; then
    project_type="monorepo"
    monorepo_tool="$(detect_monorepo_tool "$target")"
    [[ -z "$monorepo_tool" ]] && monorepo_tool="~"
  fi

  local today
  today="$(date +%Y-%m-%d)"

  {
    echo "# .aps/config.yml — written by installer, read by updater"
    echo ""
    echo "# Project contract (INSTALL-014 / D-035): toolchain pin + runtime path"
    echo "# defaults the global 'aps' binary discovers by walking up from cwd."
    echo "cli_version: \"$APS_CLI_VERSION\""
    echo "plans_dir: plans/"
    echo "docs_dir: docs/"
    echo "tooling_root: .aps/"
    echo ""
    echo "aps:"
    echo "  version: \"0.7.0\""
    echo "  config_schema: 1"
    echo "  installed: \"$today\""
    echo "  updated: \"$today\""
    echo ""
    echo "project:"
    echo "  type: $project_type"
    echo "  monorepo_tool: $monorepo_tool"
    echo "  profile: $profile"
    echo ""
    echo "tools:"
    for tool in "${tools[@]}"; do
      echo "  - name: $tool"
      case "$tool" in
        claude-code)
          echo "    skill: .claude/skills/aps-planning"
          echo "    hooks: full"
          echo "    agents:"
          echo "      - aps-planner"
          echo "      - aps-librarian"
          echo "      - aps-conductor"
          ;;
        copilot)
          echo "    skill: .claude/skills/aps-planning"
          echo "    instruction_file: AGENTS.md"
          ;;
        codex)
          echo "    skill: .agents/skills/aps-planning"
          echo "    instruction_file: AGENTS.md"
          ;;
        opencode)
          echo "    skill: .claude/skills/aps-planning"
          ;;
        grok)
          echo "    skill: .agents/skills/aps-planning"
          echo "    instruction_file: AGENTS.md"
          ;;
        antigravity)
          echo "    skill: .agents/skills/aps-planning"
          echo "    instruction_file: AGENTS.md"
          ;;
        generic)
          echo "    # No tool integration"
          ;;
      esac
    done
  } > "$config_dir/config.yml"
}

# Install v2 plans
v2_install_plans() {
  local target="$1"
  local plans_dir="$target/plans"

  mkdir -p "$plans_dir/modules" "$plans_dir/execution" "$plans_dir/decisions" "$plans_dir/designs"

  for f in "${V2_PLAN_FILES[@]}"; do
    local rel="${f#scaffold/plans/}"
    local dest="$plans_dir/$rel"
    # Rename aps-rules-v2.md to aps-rules.md at destination
    if [[ "$rel" == "aps-rules-v2.md" ]]; then
      dest="$plans_dir/aps-rules.md"
    fi
    # Don't overwrite project-context.md or issues.md if they exist
    if [[ "$rel" == "project-context.md" || "$rel" == "issues.md" ]] && [[ -f "$dest" ]]; then
      continue
    fi
    download "$f" "$dest"
  done
}

# Install v2 index (init only)
v2_install_index() {
  local target="$1"
  download "scaffold/plans/index.aps.md" "$target/plans/index.aps.md"
  touch "$target/plans/decisions/.gitkeep"
  touch "$target/plans/designs/.gitkeep"
}

# Install a federated nested-plan layout (MONO-005): a federation root plus two
# starter child plans wired via `## Child Plans`, each a complete standalone
# plan with one module. Child work-item prefixes are made distinct (per the
# package name) so cross-tree IDs don't collide (W020). The resulting tree
# lints clean out of the box; `aps lint plans` traverses it as one federation.
v2_install_nested() {
  local target="$1"

  # Federation root replaces the single-plan seed index.
  download "templates/index-nested.template.md" "$target/plans/index.aps.md"
  touch "$target/plans/decisions/.gitkeep"
  touch "$target/plans/designs/.gitkeep"

  local pkg prefix child_plans mf
  for pkg in core api; do
    prefix="$(printf '%s' "$pkg" | tr '[:lower:]' '[:upper:]')"
    child_plans="$target/packages/$pkg/plans"
    mkdir -p "$child_plans/modules"
    download "templates/index-child.template.md" "$child_plans/index.aps.md"
    download "templates/module.template.md" "$child_plans/modules/module-name.aps.md"
    # Give each child a package-specific work-item prefix (AUTH -> CORE / API)
    # so bare IDs stay unique across trees and the federation lints W020-clean.
    mf="$child_plans/modules/module-name.aps.md"
    sed "s/AUTH/$prefix/g" "$mf" > "$mf.tmp" && mv "$mf.tmp" "$mf"
  done
}

# Install v2 CLI to .aps/
v2_install_cli() {
  local target="$1"
  local aps_dir="$target/.aps"

  for f in "${V2_CLI_FILES[@]}"; do
    download "$f" "$aps_dir/$f"
  done
  touch "$aps_dir/.gitignore"
  grep -qxF 'context/' "$aps_dir/.gitignore" || printf 'context/\n' >> "$aps_dir/.gitignore"
  chmod +x "$aps_dir/bin/aps"
}

# Install v2 skill files to .claude/skills/aps-planning/ (managed, D-042)
v2_install_skill() {
  local target="$1"
  v2_reconcile_skill_tree "$target/.claude/skills/aps-planning" ".claude/skills/aps-planning"
}

# Install v2 hook scripts to .aps/scripts/
v2_install_scripts() {
  local target="$1"
  local scripts_dir="$target/.aps/scripts"

  mkdir -p "$scripts_dir"
  for f in "${V2_SCRIPT_FILES[@]}"; do
    local rel="${f#aps-planning/scripts/}"
    download "$f" "$scripts_dir/$rel"
  done
  chmod +x "$scripts_dir/"*.sh 2>/dev/null || true
}

# Install Claude Code agents to .claude/agents/
v2_install_agents() {
  local target="$1"
  local agents_dir="$target/.claude/agents"

  mkdir -p "$agents_dir"
  for f in "${V2_AGENT_FILES[@]}"; do
    local rel="${f#scaffold/agents/claude-code/}"
    download "$f" "$agents_dir/$rel"
  done
}

# Install Copilot agents to .github/agents/
v2_install_copilot_agents() {
  local target="$1"
  local agents_dir="$target/.github/agents"

  mkdir -p "$agents_dir"
  download "scaffold/agents/copilot/aps-planner.md" "$agents_dir/aps-planner.md"
  download "scaffold/agents/copilot/aps-librarian.md" "$agents_dir/aps-librarian.md"
  download "scaffold/agents/copilot/aps-conductor.md" "$agents_dir/aps-conductor.md"
}

# Install OpenCode agents to .opencode/agents/
v2_install_opencode_agents() {
  local target="$1"
  local agents_dir="$target/.opencode/agents"

  mkdir -p "$agents_dir"
  download "scaffold/agents/opencode/aps-planner.md" "$agents_dir/aps-planner.md"
  download "scaffold/agents/opencode/aps-librarian.md" "$agents_dir/aps-librarian.md"
  download "scaffold/agents/opencode/aps-conductor.md" "$agents_dir/aps-conductor.md"
}

# Install Codex agents to .codex/agents/ + skill to .agents/skills/
v2_install_codex() {
  local target="$1"

  # Agents
  local agents_dir="$target/.codex/agents"
  mkdir -p "$agents_dir"
  download "scaffold/agents/codex/aps-planner.toml" "$agents_dir/aps-planner.toml"
  download "scaffold/agents/codex/aps-librarian.toml" "$agents_dir/aps-librarian.toml"
  download "scaffold/agents/codex/aps-conductor.toml" "$agents_dir/aps-conductor.toml"
  rm -f "$agents_dir/codex-config-snippet.toml"

  # Skill at .agents/skills/ (shared with Grok)
  v2_install_agents_skill "$target"
}

# Install skill to .agents/skills/aps-planning/ (for Codex/Grok; managed, D-042)
v2_install_agents_skill() {
  local target="$1"
  v2_reconcile_skill_tree "$target/.agents/skills/aps-planning" ".agents/skills/aps-planning"
}

# Set up PATH for .aps/bin
v2_setup_path() {
  local target="$1"

  echo ""
  if command -v direnv &>/dev/null; then
    local envrc="$target/.envrc"
    if [[ -f "$envrc" ]] && grep -q 'PATH_add .aps/bin' "$envrc" 2>/dev/null; then
      info "PATH already configured in .envrc"
    elif ask_yn "Set up direnv so you can run 'aps' without .aps/bin/ prefix?" "y"; then
      # Remove old bin/ PATH if present
      if [[ -f "$envrc" ]]; then
        sed -i '/^PATH_add bin$/d' "$envrc"
        echo 'PATH_add .aps/bin' >> "$envrc"
      else
        echo 'PATH_add .aps/bin' > "$envrc"
      fi
      info "Added 'PATH_add .aps/bin' to .envrc"
      echo "  Run 'direnv allow' to activate"
    else
      info "To run aps without the path prefix, add to your .envrc:"
      echo "  PATH_add .aps/bin"
    fi
  else
    info "To run 'aps' without .aps/bin/ prefix, either:"
    echo "  - Install direnv and add 'PATH_add .aps/bin' to .envrc"
    echo "  - Or add 'export PATH=\"./.aps/bin:\$PATH\"' to your shell config"
  fi
}

# Install tool-specific files based on selections
v2_install_tools() {
  local target="$1"
  shift
  local tools=("$@")

  local post_install_msgs=()

  for tool in "${tools[@]}"; do
    case "$tool" in
      claude-code)
        v2_install_skill "$target"
        v2_install_agents "$target"
        info ".claude/skills/aps-planning/ (skill)"
        info ".claude/agents/ (planner, librarian, conductor)"
        ;;
      copilot)
        # Copilot reads .claude/skills/ too
        v2_install_skill "$target"
        v2_install_copilot_agents "$target"
        info ".claude/skills/aps-planning/ (skill — Copilot auto-discovers)"
        info ".github/agents/ (planner, librarian, conductor)"
        ;;
      opencode)
        # OpenCode reads .claude/skills/ too
        v2_install_skill "$target"
        v2_install_opencode_agents "$target"
        info ".claude/skills/aps-planning/ (skill — OpenCode auto-discovers)"
        info ".opencode/agents/ (planner, librarian, conductor)"
        ;;
      codex)
        v2_install_codex "$target"
        info ".codex/agents/ (planner, librarian, conductor TOML configs)"
        info ".agents/skills/aps-planning/ (skill)"
        post_install_msgs+=("Codex: run codex skills install .agents/skills/aps-planning")
        ;;
      grok)
        # Grok Build discovers .agents/skills/ and the AGENTS.md family natively (D-040)
        v2_install_agents_skill "$target"
        info ".agents/skills/aps-planning/ (skill — Grok Build auto-discovers)"
        ;;
      antigravity)
        # Antigravity discovers .agents/skills/ and the AGENTS.md family natively (D-045)
        v2_install_agents_skill "$target"
        info ".agents/skills/aps-planning/ (skill — Antigravity auto-discovers)"
        ;;
      generic)
        info "No tool integration (plans/ and CLI only)"
        ;;
    esac
  done

  if [[ ${#post_install_msgs[@]} -gt 0 ]]; then
    echo ""
    warn "Post-install steps required:"
    for msg in "${post_install_msgs[@]}"; do
      echo "  $msg"
    done
  fi
}

# --- v1 install functions (backward compat) ---

install_plans() {
  local target="$1"
  local plans_dir="$target/plans"

  mkdir -p "$plans_dir/modules" "$plans_dir/execution" "$plans_dir/decisions"

  for f in "${PLAN_FILES[@]}"; do
    local rel="${f#scaffold/plans/}"
    download "$f" "$plans_dir/$rel"
  done
}

install_index() {
  local target="$1"
  download "scaffold/plans/index.aps.md" "$target/plans/index.aps.md"
  touch "$target/plans/decisions/.gitkeep"
}

install_skill() {
  local target="$1"

  for f in "${SKILL_FILES[@]}"; do
    local rel="${f#scaffold/}"
    download "$f" "$target/$rel"
  done
  chmod +x "$target/aps-planning/scripts/"*.sh
}

install_commands() {
  local target="$1"
  local commands_dir="$target/.claude/commands"

  mkdir -p "$commands_dir"
  for f in "${COMMAND_FILES[@]}"; do
    local rel="${f#scaffold/commands/}"
    download "$f" "$commands_dir/$rel"
  done
}

install_cli() {
  local target="$1"

  for f in "${CLI_FILES[@]}"; do
    download "$f" "$target/$f"
  done
  chmod +x "$target/bin/aps"
}

setup_path() {
  local target="$1"

  echo ""
  if command -v direnv &>/dev/null; then
    local envrc="$target/.envrc"
    if [[ -f "$envrc" ]] && grep -q 'PATH_add bin' "$envrc" 2>/dev/null; then
      info "PATH already configured in .envrc"
    elif ask_yn "Set up direnv so you can run 'aps' without ./bin/ prefix?" "y"; then
      if [[ -f "$envrc" ]]; then
        echo 'PATH_add bin' >> "$envrc"
      else
        echo 'PATH_add bin' > "$envrc"
      fi
      info "Added 'PATH_add bin' to .envrc"
      echo "  Run 'direnv allow' to activate"
    else
      info "To run aps without the path prefix, add to your .envrc:"
      echo "  PATH_add bin"
    fi
  else
    info "To run 'aps' without ./bin/ prefix, either:"
    echo "  - Install direnv and add 'PATH_add bin' to .envrc"
    echo "  - Or add 'export PATH=\"./bin:\$PATH\"' to your shell config"
  fi
}

prompt_hooks() {
  local target="$1"

  echo ""
  if ask_yn "Install APS hooks into .claude/settings.local.json?" "y"; then
    (cd "$target" && bash aps-planning/scripts/install-hooks.sh)
  else
    if ask_yn "Would you like me to copy them for you to install/review later?" "y"; then
      info "Hook scripts are at: aps-planning/scripts/"
      echo "  Run ./aps-planning/scripts/install-hooks.sh when ready"
      echo "  See aps-planning/hooks.md for what each hook does"
    else
      info "Skipping hooks. You can install them later:"
      echo "  ./aps-planning/scripts/install-hooks.sh"
    fi
  fi
}

# --- Subcommands ---

cmd_init() {
  local target="."
  local opt_profile="" opt_scope="" opt_tools="" non_interactive=false
  local local_cli=false install_hooks=false

  while [[ $# -gt 0 ]]; do
    case $1 in
      --help|-h) cmd_init_help; exit 0 ;;
      --profile) opt_profile="$2"; shift 2 ;;
      --scope) opt_scope="$2"; shift 2 ;;
      --tools) opt_tools="$2"; shift 2 ;;
      --non-interactive) non_interactive=true; shift ;;
      --local-cli|--bash) local_cli=true; shift ;;
      --hooks) install_hooks=true; shift ;;
      *) target="$1"; shift ;;
    esac
  done

  local plans_dir="$target/plans"

  if [[ -d "$plans_dir" ]]; then
    if is_v1_layout "$target"; then
      error "Existing v1 APS installation detected."
      echo ""
      echo "To migrate to v2 layout:"
      echo "  aps migrate"
      echo ""
      echo "To update in-place (v1 layout):"
      echo "  aps update"
    else
      error "plans/ directory already exists at $target"
      echo ""
      echo "To update an existing project:"
      echo "  aps update"
    fi
    exit 1
  fi

  echo ""
  info "Initialising APS v2 in $target"

  # --- Step 1: Profile ---
  local profile
  if [[ -n "$opt_profile" ]]; then
    profile="$opt_profile"
  elif $non_interactive || ! [[ -t 0 ]]; then
    profile="solo"
  else
    local choice
    choice=$(prompt_select "What are you using APS for?" \
      "Solo dev — personal project" \
      "Team adoption — rolling out for a team" \
      "AI agent setup — planning layer for AI tools")
    case "$choice" in
      1) profile="solo" ;;
      2) profile="team" ;;
      3) profile="agent" ;;
    esac
  fi

  # --- Step 2: Scope ---
  local scope
  if [[ -n "$opt_scope" ]]; then
    scope="$opt_scope"
  elif $non_interactive || ! [[ -t 0 ]]; then
    scope="small"
  else
    local choice
    choice=$(prompt_select "What's the scope of your first plan?" \
      "Small feature (1-3 work items)" \
      "Module with boundaries" \
      "Multi-module initiative" \
      "Monorepo — tagged modules, one shared backlog" \
      "Nested monorepo — federated per-package plans")
    case "$choice" in
      1) scope="small" ;;
      2) scope="module" ;;
      3) scope="multi" ;;
      4) scope="monorepo" ;;
      5) scope="nested" ;;
    esac
  fi

  # --- Step 3: AI Tooling ---
  local selected_tools=()
  if [[ -n "$opt_tools" ]]; then
    IFS=',' read -ra selected_tools <<< "$opt_tools"
    local t
    for t in "${selected_tools[@]}"; do
      if [[ "$t" == "gemini" ]]; then
        error "'gemini' was retired in v0.7 (D-040); supported tools: ${TOOL_NAMES[*]}"
        exit 1
      fi
      if ! is_tool_key "$t"; then
        error "unknown tool '$t'; supported tools: ${TOOL_NAMES[*]}"
        exit 1
      fi
    done
  elif $non_interactive || ! [[ -t 0 ]]; then
    selected_tools=("generic")
  else
    local choices
    choices=$(prompt_multi "Which AI tools do you use? (comma-separated, e.g. 1,2,4)" \
      "${TOOL_LABELS[@]}")
    IFS=',' read -ra indices <<< "$choices"
    for idx in "${indices[@]}"; do
      idx="${idx// /}"
      selected_tools+=("${TOOL_NAMES[$((idx - 1))]}")
    done
  fi

  echo ""

  # --- Step 4: Scaffold (minimal by default — INSTALL-011) ---

  # Plans (always — the irreducible core of an APS project)
  v2_install_plans "$target"
  if [[ "$scope" == "nested" ]]; then
    v2_install_nested "$target"
    info "plans/ + packages/{core,api}/plans/ (federated nested layout)"
  else
    v2_install_index "$target"
    info "plans/ (templates, rules, project-context, designs)"
  fi

  # CLI runtime — opt-in only. Default is the global `aps` binary on PATH;
  # vendoring bash bin/ + lib/ into every project is no longer the default.
  if $local_cli; then
    v2_install_cli "$target"
    info ".aps/bin/aps + .aps/lib/ (vendored bash CLI)"
  fi

  # Hook scripts — opt-in only. Add later with `aps setup hooks`.
  if $install_hooks; then
    v2_install_scripts "$target"
    info ".aps/scripts/ (hook scripts)"
  fi

  # Tool-specific files (skills/agents) — installed only for selected tools
  v2_install_tools "$target" "${selected_tools[@]}"

  # Config (always — the per-project contract)
  write_config "$target" "$profile" "$scope" "${selected_tools[@]}"
  info ".aps/config.yml (install configuration)"

  # Print layout
  echo ""
  echo "  .aps/"
  echo "  ├── config.yml                       <- Project contract (cli_version, paths)"
  if $local_cli; then
    echo "  ├── bin/aps                           <- Vendored CLI (lint, init, update)"
    echo "  ├── lib/                              <- CLI internals"
  fi
  if $install_hooks; then
    echo "  └── scripts/                          <- Hook scripts"
  fi
  echo ""
  echo "  plans/"
  echo "  ├── aps-rules.md                      <- Agent guidance (APS-managed)"
  echo "  ├── project-context.md                <- Your project context (edit this)"
  echo "  ├── index.aps.md                      <- Your main plan (edit this)"
  echo "  ├── issues.md                         <- Issue & question tracker"
  echo "  ├── modules/                          <- Module specs"
  echo "  ├── execution/                        <- Action plans"
  echo "  ├── decisions/                        <- ADRs"
  echo "  └── designs/                          <- Technical designs"
  if [[ "$scope" == "nested" ]]; then
    echo ""
    echo "  packages/                            <- Federated child plans (MONO)"
    echo "  ├── core/plans/index.aps.md          <- Standalone child plan"
    echo "  │   └── modules/module-name.aps.md"
    echo "  └── api/plans/index.aps.md           <- Standalone child plan"
    echo "      └── modules/module-name.aps.md"
    echo ""
    echo "  The root plans/index.aps.md links these via '## Child Plans'."
    echo "  Refresh its '## Roll-up' with: aps rollup"
  fi

  # --- Step 5: Agent context bootstrap ---
  echo ""
  local has_claude=false
  for tool in "${selected_tools[@]}"; do
    [[ "$tool" == "claude-code" ]] && has_claude=true
  done

  if $has_claude; then
    info "Next: point Claude Code at plans/aps-rules.md and edit plans/project-context.md"
  else
    info "Next: edit plans/project-context.md with your project details"
  fi

  if ! $local_cli; then
    echo ""
    info "This repo uses the global 'aps' binary. Add hooks, agents, or a"
    echo "  vendored CLI later with: aps setup"
  fi

  # PATH setup (only relevant when a project-local CLI was vendored)
  if $local_cli; then
    v2_setup_path "$target"
  fi

  # --- Step 6: Verify ---
  echo ""
  info "Verifying scaffold..."
  if command -v aps &>/dev/null || [[ -x "$target/.aps/bin/aps" ]]; then
    local aps_cmd
    if [[ -x "$target/.aps/bin/aps" ]]; then
      aps_cmd="$target/.aps/bin/aps"
    else
      aps_cmd="aps"
    fi
    if "$aps_cmd" lint "$target/plans/" 2>/dev/null; then
      info "Scaffold validated successfully"
    else
      warn "Scaffold validation found issues (this is normal for a fresh install)"
    fi
  else
    info "Run 'aps lint plans/' after setting up PATH to validate"
  fi

  echo ""
}

cmd_init_help() {
  cat <<EOF
aps init - Create APS structure in a new project (v2 layout)

Usage:
  aps init [target-dir] [options]

Creates minimal planning content (plans/ + .aps/config.yml) via an
interactive wizard. By default it does NOT vendor a project-local CLI or
install hooks — the global 'aps' binary on PATH drives the repo. Add those
later with 'aps setup'. Non-interactive mode available via flags.

Options:
  --profile PROFILE     solo | team | agent
  --scope SCOPE         small | module | multi | monorepo | nested
  --tools TOOLS         Comma-separated: claude-code,copilot,codex,opencode,grok,antigravity,generic
  --local-cli, --bash   Also vendor the bash CLI into .aps/bin + .aps/lib
                        (for air-gapped or pinned-toolchain projects)
  --hooks               Also install hook scripts into .aps/scripts
  --non-interactive     Use defaults for any unspecified options
  --help                Show this help

Environment:
  APS_VERSION   Git ref to download from (default: main)

Examples:
  aps init                                        # Interactive wizard
  aps init --profile solo --scope small --tools claude-code  # Non-interactive
  aps init ./my-project                           # Init in a subdirectory
  aps init --non-interactive                      # Minimal default (solo, small, generic)
  aps init --non-interactive --local-cli          # Also vendor the bash CLI
EOF
}

cmd_update() {
  local target="."
  local global_install=false

  while [[ $# -gt 0 ]]; do
    case $1 in
      --help|-h) cmd_update_help; exit 0 ;;
      --global|-g) global_install=true; shift ;;
      *) target="$1"; shift ;;
    esac
  done

  if [[ "$global_install" == true ]]; then
    cmd_update_global
    return
  fi

  # Detect layout version
  if is_v2_layout "$target"; then
    cmd_update_v2 "$target"
  elif [[ -d "$target/plans" ]]; then
    cmd_update_v1 "$target"
  else
    error "No APS installation found at $target"
    echo ""
    echo "To create a new APS project:"
    echo "  aps init"
    exit 1
  fi
}

cmd_update_v2() {
  local target="$1"

  echo ""
  info "Updating APS v2 in $target"
  echo ""

  # CLI
  v2_install_cli "$target"
  info ".aps/bin/aps + .aps/lib/ (CLI)"

  # Plans (preserves user specs)
  v2_install_plans "$target"
  info "plans/ (templates, rules)"

  # D-044: retire the legacy plans/.aps-version stamp on update.
  if [[ -f "$target/plans/.aps-version" ]]; then
    rm -f "$target/plans/.aps-version"
    info "Removed legacy plans/.aps-version (superseded by .aps/config.yml)"
  fi

  # Scripts
  v2_install_scripts "$target"
  info ".aps/scripts/ (hook scripts)"

  # Read config.yml to determine which tool files to refresh
  if [[ -f "$target/.aps/config.yml" ]]; then
    local tools
    tools=$(grep '^\s*- name:' "$target/.aps/config.yml" | sed 's/.*name:\s*//' | tr -d ' ')
    while IFS= read -r tool; do
      case "$tool" in
        claude-code)
          v2_install_skill "$target"
          v2_install_agents "$target"
          ;;
        copilot)
          v2_install_skill "$target"
          v2_install_copilot_agents "$target"
          ;;
        opencode)
          v2_install_skill "$target"
          v2_install_opencode_agents "$target"
          ;;
        codex) v2_install_codex "$target" ;;
        grok) v2_install_agents_skill "$target" ;;
        antigravity) v2_install_agents_skill "$target" ;;
      esac
    done <<< "$tools"
    info "Tool-specific files refreshed per config.yml"

    # Update the updated timestamp
    local today
    today="$(date +%Y-%m-%d)"
    sed -i "s/updated:.*/updated: \"$today\"/" "$target/.aps/config.yml"
  fi

  echo ""
  info "Your specs (index.aps.md, modules/*.aps.md) were NOT modified."
  echo ""
}

cmd_update_v1() {
  local target="$1"

  echo ""
  info "Updating APS v1 in $target"
  warn "Consider migrating to v2 layout: aps migrate"
  echo ""

  # CLI (always update)
  install_cli "$target"
  info "bin/aps + lib/ (CLI)"

  # Templates and rules (preserves user specs)
  install_plans "$target"
  info "plans/ (templates, rules)"

  # Skill
  install_skill "$target"
  info "aps-planning/ (skill, reference, examples, hooks, scripts)"

  # Commands
  install_commands "$target"
  info ".claude/commands/ (plan, plan-status)"

  # Hooks: prompt only if not already configured
  if ! has_aps_hooks "$target"; then
    prompt_hooks "$target"
  else
    echo ""
    info "Hooks already configured (not modified)."
    echo "  To update: ./aps-planning/scripts/install-hooks.sh"
  fi

  echo ""
  info "Your specs (index.aps.md, modules/*.aps.md) were NOT modified."
  echo ""
}

cmd_update_global() {
  local aps_home="${APS_HOME:-$HOME/.aps}"

  if [[ ! -d "$aps_home/bin" ]]; then
    error "No global APS installation found at $aps_home"
    echo ""
    echo "To install globally:"
    echo "  curl -fsSL https://raw.githubusercontent.com/EddaCraft/anvil-plan-spec/main/scaffold/install | bash -s -- --global"
    echo ""
    exit 1
  fi

  echo ""
  info "Updating global APS CLI at $aps_home"
  echo ""

  for f in "${V2_CLI_FILES[@]}"; do
    download "$f" "$aps_home/$f"
  done
  chmod +x "$aps_home/bin/aps"

  echo ""
  info "Global update complete"
  info "bin/aps + lib/ updated at $aps_home"
  echo ""
}

cmd_update_help() {
  cat <<EOF
aps update - Update APS templates, skill, CLI, and tool files

Usage:
  aps update [target-dir]
  aps update --global

Updates APS-managed files without touching your specs.
Detects v1 or v2 layout and updates accordingly.

For v2: reads .aps/config.yml to determine which tool files to refresh.
For v1: updates in-place, suggests migration.

Options:
  --global  Update the global CLI installation (~/.aps/)
  --help    Show this help

Environment:
  APS_VERSION   Git ref to download from (default: main)
  APS_HOME      Custom global install location (default: ~/.aps)

Examples:
  aps update              # Update current directory
  aps update ./my-project # Update a subdirectory
  aps update --global     # Update global CLI
EOF
}

cmd_migrate() {
  local target="."
  local dry_run=false

  while [[ $# -gt 0 ]]; do
    case $1 in
      --help|-h) cmd_migrate_help; exit 0 ;;
      --dry-run) dry_run=true; shift ;;
      *) target="$1"; shift ;;
    esac
  done

  if ! is_v1_layout "$target"; then
    if is_v2_layout "$target"; then
      info "Already using v2 layout. Nothing to migrate."
      exit 0
    else
      error "No v1 APS installation found at $target"
      exit 1
    fi
  fi

  echo ""
  info "Migrating APS v1 -> v2 in $target"
  echo ""

  local moves=()
  local creates=()
  local removes=()
  local backups=()

  # Plan file moves
  if [[ -f "$target/bin/aps" ]]; then
    moves+=("bin/aps -> .aps/bin/aps")
  fi
  if [[ -d "$target/lib" ]]; then
    moves+=("lib/ -> .aps/lib/")
  fi
  if [[ -d "$target/aps-planning/scripts" ]]; then
    moves+=("aps-planning/scripts/ -> .aps/scripts/")
  fi
  if [[ -f "$target/aps-planning/SKILL.md" ]]; then
    moves+=("aps-planning/SKILL.md -> .claude/skills/aps-planning/SKILL.md")
  fi
  if [[ -f "$target/aps-planning/reference.md" ]]; then
    moves+=("aps-planning/reference.md -> .claude/skills/aps-planning/reference.md")
  fi
  if [[ -f "$target/aps-planning/examples.md" ]]; then
    moves+=("aps-planning/examples.md -> .claude/skills/aps-planning/examples.md")
  fi
  if [[ -d "$target/designs" ]]; then
    moves+=("designs/ -> plans/designs/")
  fi

  # Backup and remove deprecated files
  if [[ -f "$target/.claude/commands/plan.md" ]]; then
    backups+=(".claude/commands/plan.md -> .aps/backup/commands/plan.md")
    removes+=(".claude/commands/plan.md")
  fi
  if [[ -f "$target/.claude/commands/plan-status.md" ]]; then
    backups+=(".claude/commands/plan-status.md -> .aps/backup/commands/plan-status.md")
    removes+=(".claude/commands/plan-status.md")
  fi
  if [[ -f "$target/aps-planning/hooks.md" ]]; then
    removes+=("aps-planning/hooks.md (hook scripts are the source of truth)")
  fi

  # New files
  creates+=(".aps/config.yml (inferred from existing install)")
  if [[ ! -f "$target/plans/project-context.md" ]]; then
    creates+=("plans/project-context.md (template)")
  fi
  if [[ ! -f "$target/plans/issues.md" ]]; then
    creates+=("plans/issues.md (template)")
  fi

  # Display plan
  if [[ ${#moves[@]} -gt 0 ]]; then
    echo "  Files to move:"
    for m in "${moves[@]}"; do echo "    $m"; done
    echo ""
  fi
  if [[ ${#backups[@]} -gt 0 ]]; then
    echo "  Files to back up:"
    for b in "${backups[@]}"; do echo "    $b"; done
    echo ""
  fi
  if [[ ${#removes[@]} -gt 0 ]]; then
    echo "  Files to remove:"
    for r in "${removes[@]}"; do echo "    $r"; done
    echo ""
  fi
  if [[ ${#creates[@]} -gt 0 ]]; then
    echo "  Files to create:"
    for c in "${creates[@]}"; do echo "    $c"; done
    echo ""
  fi

  if $dry_run; then
    info "Dry run complete. No files were modified."
    exit 0
  fi

  if ! ask_yn "Proceed with migration?" "y"; then
    info "Migration cancelled."
    exit 0
  fi

  echo ""

  # Create directories
  mkdir -p "$target/.aps/bin" "$target/.aps/lib/rules" "$target/.aps/scripts" \
           "$target/.aps/backup/commands" \
           "$target/.claude/skills/aps-planning" \
           "$target/plans/designs"

  # Install fresh v2 CLI (don't copy stale v1 binaries)
  v2_install_cli "$target"
  info "Installed v2 CLI to .aps/bin/"

  # Install fresh hook scripts (don't copy old versions)
  v2_install_scripts "$target"
  info "Installed v2 hook scripts to .aps/scripts/"

  # Move skill files
  for f in SKILL.md reference.md examples.md; do
    if [[ -f "$target/aps-planning/$f" ]]; then
      cp -a "$target/aps-planning/$f" "$target/.claude/skills/aps-planning/$f"
    fi
  done
  info "Moved skill files to .claude/skills/aps-planning/"

  # Move designs (use /. to include dotfiles)
  if [[ -d "$target/designs" ]] && [[ "$(ls -A "$target/designs" 2>/dev/null)" ]]; then
    cp -a "$target/designs/." "$target/plans/designs/"
    info "Moved designs/ to plans/designs/"
  fi

  # Back up and remove deprecated commands
  for cmd_file in plan.md plan-status.md; do
    if [[ -f "$target/.claude/commands/$cmd_file" ]]; then
      cp -a "$target/.claude/commands/$cmd_file" "$target/.aps/backup/commands/$cmd_file"
      rm "$target/.claude/commands/$cmd_file"
    fi
  done
  rmdir "$target/.claude/commands" 2>/dev/null || true
  info "Backed up and removed deprecated commands"

  # Create new files
  if [[ ! -f "$target/plans/project-context.md" ]]; then
    download "scaffold/plans/project-context.md" "$target/plans/project-context.md"
    info "Created plans/project-context.md"
  fi
  if [[ ! -f "$target/plans/issues.md" ]]; then
    download "scaffold/plans/issues.md" "$target/plans/issues.md"
    info "Created plans/issues.md"
  fi

  # Update aps-rules.md to v2 version (back up existing first)
  if [[ -f "$target/plans/aps-rules.md" ]]; then
    cp -a "$target/plans/aps-rules.md" "$target/.aps/backup/aps-rules.md"
    info "Backed up existing plans/aps-rules.md to .aps/backup/"
  fi
  download "scaffold/plans/aps-rules-v2.md" "$target/plans/aps-rules.md"
  info "Updated plans/aps-rules.md to v2"
  # D-044: plans/.aps-version is retired — .aps/config.yml cli_version is the
  # only on-disk version surface.
  if [[ -f "$target/plans/.aps-version" ]]; then
    rm -f "$target/plans/.aps-version"
    info "Removed legacy plans/.aps-version (superseded by .aps/config.yml)"
  fi

  # Infer config.yml
  local inferred_tools=()
  if [[ -d "$target/.claude/agents" ]] && [[ -f "$target/.claude/agents/aps-planner.md" ]]; then
    inferred_tools+=("claude-code")
  elif [[ -d "$target/.claude/skills/aps-planning" ]]; then
    inferred_tools+=("claude-code")
  fi
  if [[ -f "$target/.github/agents/aps-planner.md" ]]; then
    inferred_tools+=("copilot")
  fi
  if [[ -f "$target/.codex/agents/aps-planner.toml" ]]; then
    inferred_tools+=("codex")
  fi
  if [[ -f "$target/.opencode/agents/aps-planner.md" ]]; then
    inferred_tools+=("opencode")
  fi
  if [[ ${#inferred_tools[@]} -eq 0 ]]; then
    inferred_tools=("generic")
  fi

  local scope="small"
  if [[ -f "$target/plans/index.aps.md" ]] && grep -q '^\| \[' "$target/plans/index.aps.md"; then
    scope="multi"
  fi

  write_config "$target" "solo" "$scope" "${inferred_tools[@]}"
  # Add inference comment
  sed -i '1s/^/# Inferred by aps migrate — review and adjust if needed\n/' "$target/.aps/config.yml"
  info "Created .aps/config.yml (inferred)"

  # Update hook paths in settings.local.json
  if [[ -f "$target/.claude/settings.local.json" ]]; then
    if grep -q 'aps-planning/scripts' "$target/.claude/settings.local.json"; then
      sed -i 's|aps-planning/scripts|.aps/scripts|g' "$target/.claude/settings.local.json"
      info "Updated hook paths in .claude/settings.local.json"
    fi
  fi

  # Clean up old directories
  # Back up custom hook scripts before removing aps-planning/
  if [[ -d "$target/aps-planning/scripts" ]]; then
    mkdir -p "$target/.aps/backup/scripts"
    cp -a "$target/aps-planning/scripts/." "$target/.aps/backup/scripts/"
    info "Backed up aps-planning/scripts/ to .aps/backup/scripts/"
  fi
  # Remove aps-planning/ entirely (skill files moved to .claude/skills/, scripts to .aps/scripts/)
  if [[ -d "$target/aps-planning" ]]; then
    rm -rf "$target/aps-planning"
    info "Removed old aps-planning/"
  fi
  # Remove only known APS files from bin/, then remove dir if empty
  if [[ -f "$target/bin/aps" ]]; then
    rm -f "$target/bin/aps" "$target/bin/aps.ps1"
    rmdir "$target/bin" 2>/dev/null && info "Removed old bin/" || \
      warn "bin/ contains non-APS files — removed only bin/aps"
  fi
  # Remove only known APS files from lib/, then remove dir if empty
  if [[ -d "$target/lib" ]] && [[ -f "$target/lib/output.sh" ]]; then
    local aps_lib_files=(output.sh Output.psm1 lint.sh Lint.psm1 orchestrate.sh scaffold.sh Scaffold.psm1)
    local aps_rule_files=(common.sh Common.psm1 module.sh Module.psm1 index.sh Index.psm1
                          workitem.sh WorkItem.psm1 issues.sh Issues.psm1 design.sh Design.psm1)
    for f in "${aps_lib_files[@]}"; do rm -f "$target/lib/$f"; done
    for f in "${aps_rule_files[@]}"; do rm -f "$target/lib/rules/$f"; done
    rmdir "$target/lib/rules" 2>/dev/null
    rmdir "$target/lib" 2>/dev/null && info "Removed old lib/" || \
      warn "lib/ contains non-APS files — removed only APS files"
  fi
  # Remove designs/ at root (contents copied to plans/designs/)
  if [[ -d "$target/designs" ]]; then
    rm -rf "$target/designs"
    info "Removed old designs/ (contents in plans/designs/)"
  fi

  # PATH update
  v2_setup_path "$target"

  echo ""
  info "Migration complete. Your specs in plans/ were NOT modified."
  info "Review .aps/config.yml and adjust if needed."
  echo ""
}

cmd_migrate_help() {
  cat <<EOF
aps migrate - Convert v1 APS layout to v2

Usage:
  aps migrate [target-dir] [options]

Detects v1 layout (bin/aps, aps-planning/, .claude/commands/) and migrates
to v2 layout (.aps/, .claude/skills/, plans/designs/).

Backs up removed files to .aps/backup/.
Creates .aps/config.yml with inferred settings.
Updates hook paths in .claude/settings.local.json.

Options:
  --dry-run   Preview changes without modifying files
  --help      Show this help

Examples:
  aps migrate              # Migrate current directory
  aps migrate --dry-run    # Preview what would change
  aps migrate ./my-project # Migrate a subdirectory
EOF
}

# --- aps setup (INSTALL-012) ---
#
# Adds optional integrations to a project after the minimal `aps init`.
# Bare `aps setup` opens a numbered picker (the Rust CLI ships a richer
# Ratatui frontend — TUI-007); shortcuts install exactly one component.

# True when "$1" is a recognised AI tool key.
is_tool_key() {
  local k="$1" t
  for t in "${TOOL_NAMES[@]}"; do
    [[ "$t" == "$k" ]] && return 0
  done
  return 1
}

# Write the agent bootstrap next-steps file (mirrors scaffold/install --agent).
write_agent_next_steps() {
  local target="$1"
  mkdir -p "$target/plans"
  cat > "$target/plans/agent-next-steps.md" <<'EOF'
# APS Agent Bootstrap — Next Steps

This repository was just initialized with a minimal APS planning layer
for an AI agent. Before implementing anything:

1. Read `plans/aps-rules.md` for the planning conventions.
2. Ask the operator for the project intent — what is being built and why.
3. Populate `plans/project-context.md` with that durable background.
4. Draft `plans/index.aps.md` (problem, outcomes, scope, modules).
5. Wait for an approved work item before writing any implementation code.

No hooks, agents, or tool integrations were installed. Run `aps setup`
to add them when needed.
EOF
}

setup_cli() {
  local target="$1"
  info "Vendoring the APS CLI into $target/.aps/"
  v2_install_cli "$target"
  info ".aps/bin/aps + .aps/lib/"
  v2_setup_path "$target"
}

setup_hooks() {
  local target="$1"
  v2_install_scripts "$target"
  info ".aps/scripts/ (hook scripts installed)"
  if [[ -f "$target/.aps/scripts/install-hooks.sh" ]]; then
    if ask_yn "Wire hooks into .claude/settings.local.json now?" "n"; then
      (cd "$target" && bash .aps/scripts/install-hooks.sh)
    else
      info "Run .aps/scripts/install-hooks.sh when you're ready"
    fi
  fi
}

setup_tools() {
  local target="$1"; shift
  local tools=("$@")
  [[ -d "$target/plans" ]] || warn "no plans/ here yet — run 'aps init' first"
  v2_install_tools "$target" "${tools[@]}"
}

setup_agent() {
  local target="$1"
  if [[ ! -d "$target/plans" ]]; then
    v2_install_plans "$target"
    v2_install_index "$target"
    info "plans/ (minimal planning content)"
  fi
  write_agent_next_steps "$target"
  info "plans/agent-next-steps.md"
}

setup_upgrade() {
  local target="$1"
  info "Refreshing templates and runtime via update"
  cmd_update "$target"
}

setup_all() {
  local target="$1" assume_yes="$2"
  if [[ "$assume_yes" != true ]]; then
    if ! ask_yn "Install the full APS footprint (CLI, hooks, Claude Code skill + agents)?" "n"; then
      info "Aborted — nothing was written."
      return 0
    fi
  fi
  if [[ ! -d "$target/plans" ]]; then
    v2_install_plans "$target"
    v2_install_index "$target"
  fi
  v2_install_cli "$target"
  v2_install_scripts "$target"
  v2_install_tools "$target" claude-code
  write_config "$target" "solo" "small" claude-code
  info "Full footprint installed."
}

cmd_setup() {
  local target="." key="" assume_yes=false
  local extra_tools=()

  while [[ $# -gt 0 ]]; do
    case $1 in
      --help|-h) cmd_setup_help; exit 0 ;;
      --yes|-y) assume_yes=true; shift ;;
      -*) error "Unknown option: $1"; echo "Run 'aps setup --help' for usage."; exit 1 ;;
      *) if [[ -z "$key" ]]; then key="$1"; else target="$1"; fi; shift ;;
    esac
  done

  # No shortcut → interactive picker; fail clearly when non-interactive.
  if [[ -z "$key" ]]; then
    if ! [[ -t 0 ]]; then
      error "aps setup needs a target when non-interactive"
      echo "  e.g. aps setup hooks | claude-code | agent | all" >&2
      exit 1
    fi
    local choice
    choice=$(prompt_select "What would you like to set up?" \
      "Vendor the APS CLI into this repo (.aps/)" \
      "Initialize minimal APS planning in this repo" \
      "Initialize this repo for an AI agent" \
      "Add a tool integration (skill + agents)" \
      "Configure hooks" \
      "Upgrade an existing APS project" \
      "Install the full APS footprint")
    case "$choice" in
      1) key="cli" ;;
      2) key="init" ;;
      3) key="agent" ;;
      4) key="tools" ;;
      5) key="hooks" ;;
      6) key="upgrade" ;;
      7) key="all" ;;
      *) error "invalid choice"; exit 1 ;;
    esac
    if [[ "$key" == "tools" ]]; then
      local choices idx
      choices=$(prompt_multi "Which AI tools? (comma-separated, e.g. 1,3)" "${TOOL_LABELS[@]}")
      local _idx
      IFS=',' read -ra _idx <<< "$choices"
      for idx in "${_idx[@]}"; do
        idx="${idx// /}"
        [[ -n "$idx" ]] && extra_tools+=("${TOOL_NAMES[$((idx - 1))]}")
      done
      [[ ${#extra_tools[@]} -gt 0 ]] || { error "no tools selected"; exit 1; }
    fi
  fi

  # Tool shortcut: `aps setup claude-code`
  if is_tool_key "$key"; then
    extra_tools=("$key")
    key="tools"
  fi

  case "$key" in
    cli)     setup_cli "$target" ;;
    init)    cmd_init "$target" --non-interactive ;;
    agent)   setup_agent "$target" ;;
    tools)   setup_tools "$target" "${extra_tools[@]}" ;;
    hooks)   setup_hooks "$target" ;;
    upgrade) setup_upgrade "$target" ;;
    all)     setup_all "$target" "$assume_yes" ;;
    *)
      error "unknown setup target '$key'"
      echo "  expected: cli, init, agent, hooks, upgrade, all, or a tool name" >&2
      echo "  tools: ${TOOL_NAMES[*]}" >&2
      exit 1
      ;;
  esac
}

cmd_setup_help() {
  cat <<EOF
aps setup - Add optional integrations to an APS project

Usage:
  aps setup [target] [options]   # interactive picker
  aps setup <component> [target] # install one component directly

Components:
  cli            Vendor the bash CLI into .aps/bin + .aps/lib
  init           Initialize minimal planning content (delegates to 'aps init')
  agent          Minimal plans + agent next-steps file
  hooks          Install hook scripts into .aps/scripts
  <tool>         Skill + agents for one tool:
                 ${TOOL_NAMES[*]}
  all            Full footprint (CLI + hooks + Claude Code) — asks first

Options:
  --yes, -y      Skip the confirmation prompt for 'all'
  --help         Show this help

Examples:
  aps setup                 # interactive picker
  aps setup hooks           # install hook scripts only
  aps setup claude-code     # add the Claude Code skill + agents
  aps setup all --yes       # full footprint without prompting
EOF
}

# --- aps upgrade (INSTALL-013) ---
#
# Safely removes generated v1/bulky-v2 bloat from an existing project. Always
# dry-runs first; on --apply it backs every removal up to .aps/backup/<ts>/
# before deleting. Never touches user planning content or instruction files.

# Known APS bash-lib filenames (relative to a lib/ dir). Used to decide whether
# a root lib/ is purely APS-owned (safe to remove) or mixed (ambiguous).
APS_LIB_FILES=(
  output.sh lint.sh orchestrate.sh audit.sh scaffold.sh
  rules/common.sh rules/module.sh rules/index.sh
  rules/workitem.sh rules/issues.sh rules/design.sh
)

# True when every file under "$1" is a known APS lib file.
_dir_only_aps_lib() {
  local d="$1" rel f known=false
  while IFS= read -r f; do
    rel="${f#"$d"/}"
    known=false
    local k
    for k in "${APS_LIB_FILES[@]}"; do
      [[ "$rel" == "$k" ]] && { known=true; break; }
    done
    $known || return 1
  done < <(find "$d" -type f)
  return 0
}

# Populate UPGRADE_REMOVE / UPGRADE_AMBIGUOUS (relative paths) for "$1".
_upgrade_scan() {
  local target="$1" f
  UPGRADE_REMOVE=()
  UPGRADE_AMBIGUOUS=()

  # Legacy Claude commands (single files)
  for f in ".claude/commands/plan.md" ".claude/commands/plan-status.md"; do
    [[ -e "$target/$f" ]] && UPGRADE_REMOVE+=("$f")
  done

  # v1 vendored CLI entry at the repo root
  [[ -f "$target/bin/aps" ]] && UPGRADE_REMOVE+=("bin/aps")

  # Root lib/ — only when it is the APS bash lib
  if [[ -d "$target/lib" && -f "$target/lib/lint.sh" ]]; then
    if _dir_only_aps_lib "$target/lib"; then
      UPGRADE_REMOVE+=("lib")
    else
      UPGRADE_AMBIGUOUS+=("lib/ (mixed APS + non-APS files)")
    fi
  fi

  # v1 skill directory (SKILL.md / hooks.md / scripts)
  if [[ -d "$target/aps-planning" ]]; then
    if [[ -f "$target/aps-planning/SKILL.md" || -f "$target/aps-planning/hooks.md" ]]; then
      UPGRADE_REMOVE+=("aps-planning")
    else
      UPGRADE_AMBIGUOUS+=("aps-planning/ (unrecognised contents)")
    fi
  fi

  # Superseded vendored v2 CLI runtime (global binary is the default now)
  [[ -d "$target/.aps/lib" ]] && UPGRADE_REMOVE+=(".aps/lib")
  [[ -d "$target/.aps/bin" ]] && UPGRADE_REMOVE+=(".aps/bin")

  # Never let a trailing short-circuit make this function return non-zero,
  # which would abort the caller under `set -e`.
  return 0
}

# Rewrite stale hook paths in settings.local.json (aps-planning/scripts -> .aps/scripts).
_upgrade_rewrite_hooks() {
  local target="$1" backup="$2"
  local settings="$target/.claude/settings.local.json"
  [[ -f "$settings" ]] || return 0
  grep -q 'aps-planning/scripts/' "$settings" || return 0

  mkdir -p "$backup/.claude"
  cp "$settings" "$backup/.claude/settings.local.json"
  local tmp
  tmp="$(mktemp)"
  sed 's#aps-planning/scripts/#.aps/scripts/#g' "$settings" > "$tmp" && mv "$tmp" "$settings"
  info "Rewrote hook paths (aps-planning/scripts/ -> .aps/scripts/) in settings.local.json"
}

cmd_upgrade() {
  local target="." apply=false assume_yes=false
  while [[ $# -gt 0 ]]; do
    case $1 in
      --help|-h) cmd_upgrade_help; exit 0 ;;
      --apply) apply=true; shift ;;
      --dry-run) apply=false; shift ;;
      --yes|-y) assume_yes=true; shift ;;
      -*) error "Unknown option: $1"; echo "Run 'aps upgrade --help' for usage."; exit 1 ;;
      *) target="$1"; shift ;;
    esac
  done

  if [[ ! -d "$target/plans" ]]; then
    error "no plans/ directory at $target — nothing to upgrade"
    exit 1
  fi

  _upgrade_scan "$target"

  if [[ ${#UPGRADE_REMOVE[@]} -eq 0 && ${#UPGRADE_AMBIGUOUS[@]} -eq 0 ]]; then
    info "No generated bloat found — this project is already clean."
    exit 0
  fi

  echo ""
  info "APS upgrade for $target"
  if [[ ${#UPGRADE_REMOVE[@]} -gt 0 ]]; then
    echo ""
    echo "Generated files to back up and remove:"
    local p
    for p in "${UPGRADE_REMOVE[@]}"; do echo "  - $p"; done
  fi
  if [[ ${#UPGRADE_AMBIGUOUS[@]} -gt 0 ]]; then
    echo ""
    warn "Ambiguous — left untouched, review manually:"
    local p
    for p in "${UPGRADE_AMBIGUOUS[@]}"; do echo "  - $p"; done
  fi
  echo ""
  info "Protected and never removed: plans/, AGENTS.md, CLAUDE.md, GEMINI.md, settings"

  if [[ "$apply" != true ]]; then
    echo ""
    info "Dry run — no files changed. Re-run with --apply to perform the cleanup."
    exit 0
  fi

  if [[ ${#UPGRADE_REMOVE[@]} -eq 0 ]]; then
    info "Nothing to remove (only ambiguous items). No changes made."
    exit 0
  fi

  if [[ "$assume_yes" != true ]]; then
    if ! ask_yn "Back up and remove the listed files?" "n"; then
      info "Aborted — nothing was changed."
      exit 0
    fi
  fi

  local ts backup
  ts="$(date +%Y%m%d-%H%M%S)"
  backup="$target/.aps/backup/$ts"
  mkdir -p "$backup"

  local p src
  for p in "${UPGRADE_REMOVE[@]}"; do
    src="$target/$p"
    [[ -e "$src" ]] || continue
    mkdir -p "$backup/$(dirname "$p")"
    cp -R "$src" "$backup/$p"
    rm -rf "$src"
    info "Backed up + removed $p"
  done

  # Prune now-empty generated parent directories
  rmdir "$target/bin" 2>/dev/null || true
  rmdir "$target/.claude/commands" 2>/dev/null || true

  # Keep hooks working if the project still uses them
  _upgrade_rewrite_hooks "$target" "$backup"

  echo ""
  info "Upgrade complete. Backup saved to .aps/backup/$ts/"
  info "Your plans/ and instruction files were not modified."
  if [[ -d "$target/.aps/lib" ]]; then :; else
    info "The vendored CLI was removed — ensure the global 'aps' binary is on PATH"
    echo "  (install it with: aps setup cli, or the --cli installer)"
  fi
}

cmd_upgrade_help() {
  cat <<EOF
aps upgrade - Safely remove generated v1/bulky-v2 bloat

Usage:
  aps upgrade [target-dir] [options]

Dry-runs by default: detects generated files (root bin/ + lib/, v1
aps-planning/, .claude/commands/, superseded .aps/bin + .aps/lib) and shows
what it would back up and remove. Nothing is changed until you pass --apply.

On --apply every removed path is copied to .aps/backup/<timestamp>/ first.
User content is never touched: plans/, AGENTS.md, CLAUDE.md, GEMINI.md, and
settings files are protected (hook paths in settings are rewritten in place,
with a backup, only when kept).

Options:
  --apply     Perform the cleanup (default is a dry run)
  --dry-run   Force a dry run (the default)
  --yes, -y   Skip the confirmation prompt under --apply
  --help      Show this help

Examples:
  aps upgrade                 # preview the cleanup for the current project
  aps upgrade --apply         # back up + remove, with a confirmation prompt
  aps upgrade --apply --yes   # non-interactive cleanup
EOF
}
