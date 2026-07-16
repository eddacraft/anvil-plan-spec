#!/usr/bin/env bash
# APS Agent Builder
# Generates tool-specific agent variants from shared core prompts.
#
# Usage: ./scaffold/agents/build.sh
#
# Generates: Claude Code, Copilot, OpenCode, Codex
# (Grok Build consumes the Codex-shared .agents/skills/ + AGENTS.md — D-040)

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
CORE_DIR="$SCRIPT_DIR/core"

info() { echo -e "\033[0;32minfo:\033[0m $1"; }

# Verify core files exist
for core in planner-core.md librarian-core.md conductor-core.md; do
  if [ ! -f "$CORE_DIR/$core" ]; then
    echo "error: missing $CORE_DIR/$core" >&2
    exit 1
  fi
done

PLANNER_DESC="Create, manage, execute, and review plans following the Anvil Plan Spec (APS) format, including initializing projects, modules, work items, action plans, validation, status tracking, and wave-based parallel execution"
LIBRARIAN_DESC="Repository organizing, cleanup, documentation filing, archiving stale specs, detecting orphaned files, cross-reference maintenance, and general repo hygiene"
CONDUCTOR_DESC="Coordinate APS execution through CLI-backed next-work selection, context packaging, dependency checks, validation, and learning capture"

# --- Claude Code (.claude/agents/) ---
CC_DIR="$SCRIPT_DIR/claude-code"
mkdir -p "$CC_DIR"

generate_claude_code() {
  local name="$1" description="$2" model="$3" tools="$4" core_file="$5"
  local output="$CC_DIR/$name.md"
  {
    echo "---"
    echo "name: $name"
    echo "description: $description"
    echo "model: $model"
    echo "tools:"
    for tool in $tools; do echo "  - $tool"; done
    echo "---"
    echo ""
    cat "$core_file"
  } > "$output"
  info "wrote $output"
}

generate_claude_code "aps-planner" "$PLANNER_DESC" "opus" \
  "Read Write Edit Glob Grep Bash Task" "$CORE_DIR/planner-core.md"
generate_claude_code "aps-librarian" "$LIBRARIAN_DESC" "sonnet" \
  "Read Write Edit Glob Grep Bash" "$CORE_DIR/librarian-core.md"
generate_claude_code "aps-conductor" "$CONDUCTOR_DESC" "opus" \
  "Read Write Edit Glob Grep Bash Task" "$CORE_DIR/conductor-core.md"

# --- Copilot (.github/agents/) ---
CP_DIR="$SCRIPT_DIR/copilot"
mkdir -p "$CP_DIR"

generate_copilot() {
  local name="$1" description="$2" core_file="$3"
  local output="$CP_DIR/$name.md"
  {
    echo "---"
    echo "name: $name"
    echo "description: $description"
    echo "---"
    echo ""
    cat "$core_file"
  } > "$output"
  info "wrote $output"
}

generate_copilot "aps-planner" "$PLANNER_DESC" "$CORE_DIR/planner-core.md"
generate_copilot "aps-librarian" "$LIBRARIAN_DESC" "$CORE_DIR/librarian-core.md"
generate_copilot "aps-conductor" "$CONDUCTOR_DESC" "$CORE_DIR/conductor-core.md"

# --- OpenCode (.opencode/agents/) ---
OC_DIR="$SCRIPT_DIR/opencode"
mkdir -p "$OC_DIR"

generate_opencode() {
  local name="$1" description="$2" model="$3" steps="$4" core_file="$5"
  local output="$OC_DIR/$name.md"
  {
    echo "---"
    echo "description: $description"
    echo "mode: subagent"
    echo "model: $model"
    echo "steps: $steps"
    echo "tools:"
    echo "  read: true"
    echo "  write: true"
    echo "  edit: true"
    echo "  glob: true"
    echo "  grep: true"
    echo "  bash: true"
    echo "permission:"
    echo "  edit: \"ask\""
    echo "  write: \"ask\""
    echo "  bash: \"ask\""
    echo "---"
    echo ""
    cat "$core_file"
  } > "$output"
  info "wrote $output"
}

generate_opencode "aps-planner" "$PLANNER_DESC" \
  "anthropic/claude-opus-4-6" 50 "$CORE_DIR/planner-core.md"
generate_opencode "aps-librarian" "$LIBRARIAN_DESC" \
  "anthropic/claude-sonnet-4-6" 30 "$CORE_DIR/librarian-core.md"
generate_opencode "aps-conductor" "$CONDUCTOR_DESC" \
  "anthropic/claude-opus-4-6" 50 "$CORE_DIR/conductor-core.md"

# --- Codex (.codex/agents/) ---
CX_DIR="$SCRIPT_DIR/codex"
mkdir -p "$CX_DIR"

generate_codex() {
  local name="$1" comment="$2" core_file="$3"
  local output="$CX_DIR/$name.toml"
  local core_content
  core_content=$(cat "$core_file")

  {
    echo "# APS ${comment} — Codex Agent Role"
    echo "#"
    echo "# Install: merge the [agents.$name] block from codex-config-snippet.toml"
    echo "# into your .codex/config.toml, then place this file at .codex/agents/$name.toml"
    echo "#"
    echo "# Usage: /agent spawn $name"
    echo ""
    echo 'sandbox_mode = "workspace-write"'
    echo ""
    echo 'developer_instructions = """'
    echo "$core_content"
    echo '"""'
  } > "$output"
  info "wrote $output"
}

generate_codex "aps-planner" "Planner" "$CORE_DIR/planner-core.md"
generate_codex "aps-librarian" "Librarian" "$CORE_DIR/librarian-core.md"
generate_codex "aps-conductor" "Conductor" "$CORE_DIR/conductor-core.md"

# Codex config snippet (static, not generated from core)
cat > "$CX_DIR/codex-config-snippet.toml" << 'SNIPPET'
# APS Agent Roles for Codex
#
# Merge these blocks into your .codex/config.toml to register the APS agents.
# Then place aps-planner.toml, aps-librarian.toml, and aps-conductor.toml in
# .codex/agents/.
#
# Usage:
#   /agent spawn aps-planner
#   /agent spawn aps-librarian
#   /agent spawn aps-conductor

[agents.aps-planner]
model = "o4-mini"  # OpenAI model — Codex runs on OpenAI infrastructure
config_file = ".codex/agents/aps-planner.toml"

[agents.aps-librarian]
model = "o4-mini"  # OpenAI model — Codex runs on OpenAI infrastructure
config_file = ".codex/agents/aps-librarian.toml"

[agents.aps-conductor]
model = "o4-mini"  # OpenAI model — Codex runs on OpenAI infrastructure
config_file = ".codex/agents/aps-conductor.toml"
SNIPPET
info "wrote $CX_DIR/codex-config-snippet.toml"

echo ""
info "all agent variants generated"
