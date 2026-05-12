#!/usr/bin/env bash
#
# APS Scaffold Initializer
# Creates or updates APS structure with templates, agent guidance, and planning skill.
#
# Usage:
#   # Initialize new project
#   curl -fsSL https://raw.githubusercontent.com/EddaCraft/anvil-plan-spec/main/scaffold/init.sh | bash
#
#   # Update existing project (creates missing dirs, updates templates)
#   curl -fsSL https://raw.githubusercontent.com/EddaCraft/anvil-plan-spec/main/scaffold/init.sh | bash -s -- --update
#
#   # Local usage
#   bash init.sh [target-dir]           # Initialize new project
#   bash init.sh --update [target-dir]  # Update templates only (preserves your specs)
#

set -eo pipefail

UPDATE_MODE=false
TARGET="."

while [[ $# -gt 0 ]]; do
  case $1 in
    --update|-u)
      UPDATE_MODE=true
      shift
      ;;
    *)
      TARGET="$1"
      shift
      ;;
  esac
done

PLANS_DIR="$TARGET/plans"
APS_BASE_URL="https://raw.githubusercontent.com/EddaCraft/anvil-plan-spec/main"

if [[ -t 1 ]]; then
  GREEN='\033[0;32m'
  YELLOW='\033[1;33m'
  BLUE='\033[0;34m'
  BOLD='\033[1m'
  NC='\033[0m'
else
  GREEN='' YELLOW='' BLUE='' BOLD='' NC=''
fi

info() { echo -e "${GREEN}[aps]${NC} $1"; }
warn() { echo -e "${YELLOW}[aps]${NC} $1"; }
step() { echo -e "${BLUE}==>${NC} ${BOLD}$1${NC}"; }

# Detect if running via curl pipe (BASH_SOURCE is empty) or locally
if [[ -n "${BASH_SOURCE[0]:-}" && "${BASH_SOURCE[0]}" != "bash" ]]; then
  SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
else
  SCRIPT_DIR=""
fi

# Copy from local repo or download from GitHub
# $1: path relative to repo root (e.g., "scaffold/plans/aps-rules.md" or "bin/aps")
# $2: destination path
copy_or_download() {
  local src_name="$1"
  local dest_path="$2"

  mkdir -p "$(dirname "$dest_path")"

  # Try local file first: SCRIPT_DIR points to scaffold/, go up one level for repo root
  if [[ -n "$SCRIPT_DIR" ]]; then
    local repo_root
    repo_root="$(cd "$SCRIPT_DIR/.." && pwd)"
    local src_path="$repo_root/$src_name"
    if [[ -f "$src_path" ]]; then
      local src_real dest_real
      src_real="$(realpath "$src_path" 2>/dev/null || echo "$src_path")"
      dest_real="$(realpath "$dest_path" 2>/dev/null || echo "$dest_path")"
      if [[ "$src_real" != "$dest_real" ]]; then
        cp "$src_path" "$dest_path"
        return
      fi
    fi
  fi
  # Fall back to downloading
  curl -fsSL "$APS_BASE_URL/$src_name" -o "$dest_path"
}

# Prompt user with a yes/no question. Returns 0 for yes, 1 for no.
# Falls back to "no" if not running interactively (e.g., curl pipe without -s --).
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
    # Non-interactive: use default
    [[ "$default" == "y" ]]
  fi
}

# Install the APS CLI (bin/aps + lib/)
install_cli() {
  local target_dir="$1"

  step "Installing APS CLI"

  local cli_files=(
    "bin/aps"
    "lib/output.sh"
    "lib/lint.sh"
    "lib/scaffold.sh"
    "lib/rules/common.sh"
    "lib/rules/module.sh"
    "lib/rules/index.sh"
    "lib/rules/workitem.sh"
    "lib/rules/design.sh"
  )

  for f in "${cli_files[@]}"; do
    copy_or_download "$f" "$target_dir/$f"
  done
  chmod +x "$target_dir/bin/aps"

  info "bin/aps + lib/ (CLI)"
}

# Install the APS planning skill and commands
install_skill() {
  local target_dir="$1"
  local skill_dir="$target_dir/aps-planning"
  local commands_dir="$target_dir/.claude/commands"

  step "Installing APS planning skill"

  # Skill files
  mkdir -p "$skill_dir/scripts"
  copy_or_download "scaffold/aps-planning/SKILL.md" "$skill_dir/SKILL.md"
  copy_or_download "scaffold/aps-planning/reference.md" "$skill_dir/reference.md"
  copy_or_download "scaffold/aps-planning/examples.md" "$skill_dir/examples.md"
  copy_or_download "scaffold/aps-planning/hooks.md" "$skill_dir/hooks.md"
  copy_or_download "scaffold/aps-planning/scripts/install-hooks.sh" "$skill_dir/scripts/install-hooks.sh"
  copy_or_download "scaffold/aps-planning/scripts/init-session.sh" "$skill_dir/scripts/init-session.sh"
  copy_or_download "scaffold/aps-planning/scripts/check-complete.sh" "$skill_dir/scripts/check-complete.sh"
  copy_or_download "scaffold/aps-planning/scripts/pre-tool-check.sh" "$skill_dir/scripts/pre-tool-check.sh"
  copy_or_download "scaffold/aps-planning/scripts/post-tool-nudge.sh" "$skill_dir/scripts/post-tool-nudge.sh"
  chmod +x "$skill_dir/scripts/"*.sh

  info "aps-planning/ (skill, reference, examples, hooks, scripts)"

  # Slash commands
  mkdir -p "$commands_dir"
  copy_or_download "scaffold/commands/plan.md" "$commands_dir/plan.md"
  copy_or_download "scaffold/commands/plan-status.md" "$commands_dir/plan-status.md"

  info ".claude/commands/ (plan, plan-status)"
}

# Set up PATH so `aps` works without ./bin/ prefix
setup_path() {
  local target_dir="$1"

  echo ""
  if command -v direnv &>/dev/null; then
    local envrc="$target_dir/.envrc"
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

# Check if APS hooks are already configured in settings
has_aps_hooks() {
  local target_dir="$1"
  local settings="$target_dir/.claude/settings.local.json"
  [[ -f "$settings" ]] && grep -q 'aps-planning/scripts\|\[APS\]' "$settings" 2>/dev/null
}

# Prompt for hook installation with two-step fallback
prompt_hooks() {
  local target_dir="$1"
  local hook_script="$target_dir/aps-planning/scripts/install-hooks.sh"

  echo ""
  if ask_yn "Install APS hooks into .claude/settings.local.json?" "y"; then
    (cd "$target_dir" && bash "$hook_script")
  else
    if ask_yn "Copy hook scripts for you to install/review later?" "y"; then
      info "Hook scripts are at: aps-planning/scripts/"
      echo "  Run ./aps-planning/scripts/install-hooks.sh when ready"
      echo "  See aps-planning/hooks.md for what each hook does"
    else
      info "Skipping hooks. You can install them later:"
      echo "  ./aps-planning/scripts/install-hooks.sh"
    fi
  fi
}

# --- Main flow ---

if [[ "$UPDATE_MODE" == true ]]; then
  if [[ ! -d "$PLANS_DIR" ]]; then
    warn "No plans/ directory found. Run without --update to initialize."
    exit 1
  fi

  info "Updating APS templates, rules, and skill (your specs are preserved)..."

  # Create any missing directories
  mkdir -p "$PLANS_DIR/modules"
  mkdir -p "$PLANS_DIR/execution"
  mkdir -p "$PLANS_DIR/decisions"
  mkdir -p "$TARGET/designs"

  # CLI
  install_cli "$TARGET"

  # Templates and rules
  step "Updating templates and rules"
  copy_or_download "scaffold/plans/aps-rules.md" "$PLANS_DIR/aps-rules.md"
  copy_or_download "scaffold/plans/modules/.module.template.md" "$PLANS_DIR/modules/.module.template.md"
  copy_or_download "scaffold/plans/modules/.simple.template.md" "$PLANS_DIR/modules/.simple.template.md"
  copy_or_download "scaffold/plans/modules/.index-monorepo.template.md" "$PLANS_DIR/modules/.index-monorepo.template.md"
  copy_or_download "scaffold/plans/execution/.actions.template.md" "$PLANS_DIR/execution/.actions.template.md"
  copy_or_download "scaffold/designs/.design.template.md" "$TARGET/designs/.design.template.md"

  info "plans/ + designs/ (templates, rules)"

  info "Skill not updated. Run: aps setup claude-code  (or codex/opencode) to install/update the planning skill."

  echo ""
  info "Your specs (index.aps.md, modules/*.aps.md) were NOT modified."

else
  if [[ -d "$PLANS_DIR" ]]; then
    warn "plans/ directory already exists."
    warn "To update templates only: bash init.sh --update"
    warn "To reinitialize: rm -rf $PLANS_DIR && bash init.sh"
    exit 1
  fi

  # CLI
  install_cli "$TARGET"

  # Plans
  step "Creating APS structure"

  mkdir -p "$PLANS_DIR/modules"
  mkdir -p "$PLANS_DIR/execution"
  mkdir -p "$PLANS_DIR/decisions"
  mkdir -p "$TARGET/designs"

  copy_or_download "scaffold/plans/aps-rules.md" "$PLANS_DIR/aps-rules.md"
  copy_or_download "scaffold/plans/index.aps.md" "$PLANS_DIR/index.aps.md"
  copy_or_download "scaffold/plans/modules/.module.template.md" "$PLANS_DIR/modules/.module.template.md"
  copy_or_download "scaffold/plans/modules/.simple.template.md" "$PLANS_DIR/modules/.simple.template.md"
  copy_or_download "scaffold/plans/modules/.index-monorepo.template.md" "$PLANS_DIR/modules/.index-monorepo.template.md"
  copy_or_download "scaffold/plans/execution/.actions.template.md" "$PLANS_DIR/execution/.actions.template.md"
  copy_or_download "scaffold/designs/.design.template.md" "$TARGET/designs/.design.template.md"

  touch "$PLANS_DIR/decisions/.gitkeep"

  info "plans/ + designs/ (templates, rules, index)"

  echo ""
  echo "  bin/"
  echo "  └── aps                              <- CLI (lint, init, update)"
  echo ""
  echo "  designs/"
  echo "  └── .design.template.md              <- Template for technical designs"
  echo ""
  echo "  plans/"
  echo "  ├── aps-rules.md                     <- Agent guidance (READ THIS)"
  echo "  ├── index.aps.md                     <- Your main plan (edit this)"
  echo "  ├── modules/"
  echo "  │   ├── .module.template.md          <- Template for modules"
  echo "  │   ├── .simple.template.md          <- Template for small features"
  echo "  │   └── .index-monorepo.template.md  <- Index for monorepos"
  echo "  ├── execution/"
  echo "  │   └── .actions.template.md         <- Template for action plans"
  echo "  └── decisions/"
  echo ""
  echo "  aps-planning/"
  echo "  ├── SKILL.md                         <- Planning skill (core rules)"
  echo "  ├── reference.md                     <- APS format reference"
  echo "  ├── examples.md                      <- Real-world examples"
  echo "  ├── hooks.md                         <- Hook configuration guide"
  echo "  └── scripts/                         <- Hook install + session scripts"
  echo ""
  echo "  Harness skill not installed."
  echo "  Run: aps setup claude-code   (Claude Code)"
  echo "       aps setup codex         (Codex)"
  echo "       aps setup opencode      (OpenCode)"

  echo ""
  step "Next steps"
  echo "  1. Edit plans/index.aps.md to define your plan"
  echo "  2. Copy templates to create modules (remove leading dot)"
  echo "  3. Run: aps setup <harness> to install the planning skill for your tool"
  echo ""
fi
