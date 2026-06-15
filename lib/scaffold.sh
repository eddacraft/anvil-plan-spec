#!/usr/bin/env bash
#
# Scaffold logic for `aps init`, `aps update`, and `aps migrate`
#

APS_VERSION="${APS_VERSION:-main}"
APS_BASE_URL="https://raw.githubusercontent.com/EddaCraft/anvil-plan-spec/$APS_VERSION"

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
  "lib/scaffold.sh"
  "lib/rules/common.sh"
  "lib/rules/module.sh"
  "lib/rules/index.sh"
  "lib/rules/workitem.sh"
  "lib/rules/issues.sh"
  "lib/rules/design.sh"
)

# Canonical tool identifiers
TOOL_NAMES=("claude-code" "copilot" "codex" "opencode" "gemini" "generic")
TOOL_LABELS=("Claude Code" "GitHub Copilot" "Codex" "OpenCode" "Gemini" "None / manual only")

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

  local project_type="simple"
  local monorepo_tool="~"
  if [[ "$scope" == "monorepo" ]]; then
    project_type="monorepo"
    monorepo_tool="$(detect_monorepo_tool "$target")"
    [[ -z "$monorepo_tool" ]] && monorepo_tool="~"
  fi

  local today
  today="$(date +%Y-%m-%d)"

  {
    echo "# .aps/config.yml — written by installer, read by updater"
    echo "aps:"
    echo "  version: \"0.3.0\""
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
        gemini)
          echo "    skill: .agents/skills/aps-planning"
          echo "    instruction_file: GEMINI.md"
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
  echo "0.3.0" > "$plans_dir/.aps-version"

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

# Install v2 skill files to .claude/skills/aps-planning/
v2_install_skill() {
  local target="$1"
  local skill_dir="$target/.claude/skills/aps-planning"

  mkdir -p "$skill_dir"
  for f in "${V2_SKILL_FILES[@]}"; do
    local rel="${f#scaffold/aps-planning/}"
    download "$f" "$skill_dir/$rel"
  done
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
  download "scaffold/agents/codex/codex-config-snippet.toml" "$agents_dir/codex-config-snippet.toml"

  # Skill at .agents/skills/ (shared with Gemini)
  v2_install_agents_skill "$target"
}

# Install skill to .agents/skills/aps-planning/ (for Codex/Gemini)
v2_install_agents_skill() {
  local target="$1"
  local skill_dir="$target/.agents/skills/aps-planning"

  mkdir -p "$skill_dir"
  for f in "${V2_SKILL_FILES[@]}"; do
    local rel="${f#scaffold/aps-planning/}"
    download "$f" "$skill_dir/$rel"
  done
}

# Install Gemini skills to .gemini/skills/
v2_install_gemini() {
  local target="$1"

  mkdir -p "$target/.gemini/skills/aps-planner" "$target/.gemini/skills/aps-librarian" "$target/.gemini/skills/aps-conductor"
  download "scaffold/agents/gemini/aps-planner/SKILL.md" "$target/.gemini/skills/aps-planner/SKILL.md"
  download "scaffold/agents/gemini/aps-librarian/SKILL.md" "$target/.gemini/skills/aps-librarian/SKILL.md"
  download "scaffold/agents/gemini/aps-conductor/SKILL.md" "$target/.gemini/skills/aps-conductor/SKILL.md"

  # Also place skill at .agents/skills/ (shared path)
  v2_install_agents_skill "$target"
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
        post_install_msgs+=("Codex: merge .codex/agents/codex-config-snippet.toml into .codex/config.toml")
        post_install_msgs+=("  then run: codex skills install .agents/skills/aps-planning")
        ;;
      gemini)
        v2_install_gemini "$target"
        info ".gemini/skills/ (planner, librarian, conductor)"
        info ".agents/skills/aps-planning/ (skill)"
        post_install_msgs+=("Gemini: run: gemini skills link . --scope workspace")
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

  while [[ $# -gt 0 ]]; do
    case $1 in
      --help|-h) cmd_init_help; exit 0 ;;
      --profile) opt_profile="$2"; shift 2 ;;
      --scope) opt_scope="$2"; shift 2 ;;
      --tools) opt_tools="$2"; shift 2 ;;
      --non-interactive) non_interactive=true; shift ;;
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
      "Monorepo (multiple packages/apps)")
    case "$choice" in
      1) scope="small" ;;
      2) scope="module" ;;
      3) scope="multi" ;;
      4) scope="monorepo" ;;
    esac
  fi

  # --- Step 3: AI Tooling ---
  local selected_tools=()
  if [[ -n "$opt_tools" ]]; then
    IFS=',' read -ra selected_tools <<< "$opt_tools"
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

  # --- Step 4: Scaffold ---

  # CLI
  v2_install_cli "$target"
  info ".aps/bin/aps + .aps/lib/ (CLI)"

  # Plans
  v2_install_plans "$target"
  v2_install_index "$target"
  info "plans/ (templates, rules, project-context, designs)"

  # Hook scripts
  v2_install_scripts "$target"
  info ".aps/scripts/ (hook scripts)"

  # Tool-specific files
  v2_install_tools "$target" "${selected_tools[@]}"

  # Config
  write_config "$target" "$profile" "$scope" "${selected_tools[@]}"
  info ".aps/config.yml (install configuration)"

  # Print layout
  echo ""
  echo "  .aps/"
  echo "  ├── config.yml                       <- Install configuration"
  echo "  ├── bin/aps                           <- CLI (lint, init, update, migrate)"
  echo "  ├── lib/                              <- CLI internals"
  echo "  └── scripts/                          <- Hook scripts"
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

  # PATH setup
  v2_setup_path "$target"

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

Creates .aps/ tooling root, plans/, and tool-specific files via an
interactive wizard. Non-interactive mode available via flags.

Options:
  --profile PROFILE     solo | team | agent
  --scope SCOPE         small | module | multi | monorepo
  --tools TOOLS         Comma-separated: claude-code,copilot,codex,opencode,gemini,generic
  --non-interactive     Use defaults for any unspecified options
  --help                Show this help

Environment:
  APS_VERSION   Git ref to download from (default: main)

Examples:
  aps init                                        # Interactive wizard
  aps init --profile solo --scope small --tools claude-code  # Non-interactive
  aps init ./my-project                           # Init in a subdirectory
  aps init --non-interactive                      # All defaults (solo, small, generic)
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
        gemini) v2_install_gemini "$target" ;;
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
  echo "0.3.0" > "$target/plans/.aps-version"
  info "Updated plans/.aps-version to 0.3.0"

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
