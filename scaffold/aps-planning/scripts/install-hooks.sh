#!/usr/bin/env bash
#
# APS Hooks Installer
# Merges APS hook configuration into .claude/settings.local.json
# Preserves existing settings and permissions.
#
# Usage:
#   ./.aps/scripts/install-hooks.sh           # Install all hooks
#   ./.aps/scripts/install-hooks.sh --minimal  # PreToolUse + Stop only
#   ./.aps/scripts/install-hooks.sh --remove   # Remove APS hooks
#
# Requires: python3 (for JSON manipulation)

set -euo pipefail

SETTINGS_DIR=".claude"
SETTINGS_FILE="$SETTINGS_DIR/settings.local.json"
MODE="full"

while [[ $# -gt 0 ]]; do
  case $1 in
    --minimal|-m) MODE="minimal"; shift ;;
    --remove|-r)  MODE="remove";  shift ;;
    --help|-h)
      echo "Usage: ./.aps/scripts/install-hooks.sh [OPTIONS]"
      echo ""
      echo "Options:"
      echo "  --minimal, -m  Install PreToolUse + Stop + SessionStart hooks"
      echo "  --remove, -r   Remove all APS hooks"
      echo "  --help, -h     Show this help"
      exit 0
      ;;
    *) echo "Unknown option: $1"; exit 1 ;;
  esac
done

# Colors
if [ -t 1 ]; then
  GREEN='\033[0;32m'
  YELLOW='\033[1;33m'
  RED='\033[0;31m'
  NC='\033[0m'
else
  GREEN='' YELLOW='' RED='' NC=''
fi

info()  { echo -e "${GREEN}[aps]${NC} $1"; }
warn()  { echo -e "${YELLOW}[aps]${NC} $1"; }
error() { echo -e "${RED}[aps]${NC} $1"; }

# Check for python3
if ! command -v python3 &>/dev/null; then
  error "python3 is required for JSON manipulation."
  echo "  Install it or manually copy hook config from aps-planning/hooks.md"
  exit 1
fi

# Ensure .claude directory exists
mkdir -p "$SETTINGS_DIR"

# Create settings file if it doesn't exist
if [ ! -f "$SETTINGS_FILE" ]; then
  echo '{}' > "$SETTINGS_FILE"
  info "Created $SETTINGS_FILE"
fi

# Use python3 for safe JSON merge
python3 - "$SETTINGS_FILE" "$MODE" << 'PYEOF'
import json
import sys

settings_path = sys.argv[1]
mode = sys.argv[2]

# Load existing settings
with open(settings_path) as f:
    settings = json.load(f)

def is_aps_hook(entry):
    """Check if a hook entry is APS-related (handles both old and new format)."""
    # Old format: {"hook": "...aps-planning/scripts..."}
    hook_str = entry.get("hook", "")
    if "[APS]" in hook_str or "aps-planning/scripts" in hook_str or ".aps/scripts" in hook_str:
        return True
    # New format: {"hooks": [{"command": "...aps-planning/scripts..."}]}
    for h in entry.get("hooks", []):
        cmd = h.get("command", "")
        if "aps-planning/scripts" in cmd or ".aps/scripts" in cmd:
            return True
    return False

if mode == "remove":
    # Remove APS hooks (handles both old and new format)
    if "hooks" in settings:
        for event in list(settings["hooks"].keys()):
            hooks = settings["hooks"][event]
            settings["hooks"][event] = [
                h for h in hooks if not is_aps_hook(h)
            ]
            # Clean up empty arrays
            if not settings["hooks"][event]:
                del settings["hooks"][event]
        if not settings["hooks"]:
            del settings["hooks"]
    print(json.dumps(settings, indent=2))
    with open(settings_path, "w") as f:
        json.dump(settings, f, indent=2)
        f.write("\n")
    sys.exit(0)

# Define APS hooks (new format: matcher + hooks array with type/command)
pretool = {
    "matcher": "Write|Edit|Bash",
    "hooks": [{
        "type": "command",
        "command": "./.aps/scripts/pre-tool-check.sh"
    }]
}

posttool = {
    "matcher": "Write|Edit",
    "hooks": [{
        "type": "command",
        "command": "./.aps/scripts/post-tool-nudge.sh"
    }]
}

stop = {
    "hooks": [{
        "type": "command",
        "command": "./.aps/scripts/check-complete.sh"
    }]
}

enforce_plan = {
    "hooks": [{
        "type": "command",
        "command": "./.aps/scripts/enforce-plan-update.sh"
    }]
}

session_start = {
    "hooks": [{
        "type": "command",
        "command": "./.aps/scripts/init-session.sh"
    }]
}

# Build hooks based on mode
if mode == "minimal":
    new_hooks = {
        "PreToolUse": [pretool],
        "Stop": [stop, enforce_plan],
        "SessionStart": [session_start],
    }
else:
    new_hooks = {
        "PreToolUse": [pretool],
        "PostToolUse": [posttool],
        "Stop": [stop, enforce_plan],
        "SessionStart": [session_start],
    }

# Merge: add APS hooks without clobbering existing non-APS hooks
if "hooks" not in settings:
    settings["hooks"] = {}

for event, aps_hooks in new_hooks.items():
    if event not in settings["hooks"]:
        settings["hooks"][event] = []

    # Remove any existing APS hooks first (idempotent, handles old+new format)
    existing = [
        h for h in settings["hooks"][event]
        if not is_aps_hook(h)
    ]

    # Append new APS hooks
    settings["hooks"][event] = existing + aps_hooks

with open(settings_path, "w") as f:
    json.dump(settings, f, indent=2)
    f.write("\n")
PYEOF

# Ensure session baseline is gitignored
BASELINE_ENTRY=".claude/.aps-session-baseline"
if [ "$MODE" != "remove" ]; then
  if [ -f .gitignore ]; then
    if ! grep -qF "$BASELINE_ENTRY" .gitignore 2>/dev/null; then
      echo "" >> .gitignore
      echo "# APS session baseline (ephemeral)" >> .gitignore
      echo "$BASELINE_ENTRY" >> .gitignore
      info "Added $BASELINE_ENTRY to .gitignore"
    fi
  else
    echo "# APS session baseline (ephemeral)" > .gitignore
    echo "$BASELINE_ENTRY" >> .gitignore
    info "Created .gitignore with $BASELINE_ENTRY"
  fi
fi

if [ "$MODE" = "remove" ]; then
  info "Removed APS hooks from $SETTINGS_FILE"
else
  info "Installed APS hooks ($MODE mode) into $SETTINGS_FILE"
  echo ""
  echo "  Hooks added:"
  if [ "$MODE" = "full" ]; then
    echo "    PreToolUse          — Reminds agent to check plan before code changes"
    echo "    PostToolUse         — Nudges agent to update specs after changes"
    echo "    Stop (Completion)   — Blocks session end if work items unresolved"
    echo "    Stop (Plan Update)  — Blocks session end if code changed but plans untouched"
    echo "    SessionStart        — Shows planning status at session start"
  else
    echo "    PreToolUse          — Reminds agent to check plan before code changes"
    echo "    Stop (Completion)   — Blocks session end if work items unresolved"
    echo "    Stop (Plan Update)  — Blocks session end if code changed but plans untouched"
    echo "    SessionStart        — Writes session baseline for change detection"
  fi
  echo ""
  info "See aps-planning/hooks.md for details on each hook."
fi
