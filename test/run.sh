#!/usr/bin/env bash
#
# Simple test runner for APS CLI
#

set -eo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
APS="$PROJECT_ROOT/bin/aps"

# Pin hygiene thresholds so caller-environment values can't skew fixtures
export APS_STALE_DAYS=60
export APS_AUDIT_TIMEOUT=60

RED='\033[0;31m'
GREEN='\033[0;32m'
NC='\033[0m'

pass() { echo -e "${GREEN}PASS${NC} $1"; }
fail() { echo -e "${RED}FAIL${NC} $1"; exit 1; }

echo "Running APS CLI tests..."
echo ""

# Test 1: Help command works
echo -n "Test: --help returns 0... "
$APS --help > /dev/null 2>&1 && pass || fail "help command failed"

# Test 2: lint --help works
echo -n "Test: lint --help returns 0... "
$APS lint --help > /dev/null 2>&1 && pass || fail "lint help failed"

# Test 3: Valid fixtures pass
echo -n "Test: valid fixtures pass... "
$APS lint "$SCRIPT_DIR/fixtures/valid/" > /dev/null 2>&1 && pass || fail "valid fixtures failed"

# Test 4: Invalid fixtures fail with exit 1
echo -n "Test: invalid fixtures return exit 1... "
if $APS lint "$SCRIPT_DIR/fixtures/invalid/" > /dev/null 2>&1; then
  fail "expected exit 1 for invalid fixtures"
else
  pass
fi

# Test 5: E001 - Missing purpose detected
echo -n "Test: E001 missing purpose detected... "
output=$($APS lint "$SCRIPT_DIR/fixtures/invalid/missing-purpose.aps.md" 2>&1) || true
echo "$output" | grep -q "E001" && pass || fail "E001 not detected"

# Test 6: E002 - Missing work items detected
echo -n "Test: E002 missing work items detected... "
output=$($APS lint "$SCRIPT_DIR/fixtures/invalid/missing-work-items.aps.md" 2>&1) || true
echo "$output" | grep -q "E002" && pass || fail "E002 not detected"

# Test 7: E003 - Missing metadata detected
echo -n "Test: E003 missing metadata detected... "
output=$($APS lint "$SCRIPT_DIR/fixtures/invalid/missing-metadata.aps.md" 2>&1) || true
echo "$output" | grep -q "E003" && pass || fail "E003 not detected"

# Test 8: E005 - Missing required fields detected
echo -n "Test: E005 missing required fields detected... "
output=$($APS lint "$SCRIPT_DIR/fixtures/invalid/missing-required-fields.aps.md" 2>&1) || true
echo "$output" | grep -q "E005" && pass || fail "E005 not detected"

# Test 9: W001 - Bad task ID format detected
echo -n "Test: W001 bad task ID detected... "
output=$($APS lint "$SCRIPT_DIR/fixtures/invalid/bad-task-id.aps.md" 2>&1) || true
echo "$output" | grep -q "W001" && pass || fail "W001 not detected"

# Test 10: Valid issues.md passes lint
echo -n "Test: valid issues.md passes... "
$APS lint "$SCRIPT_DIR/fixtures/valid/issues.md" > /dev/null 2>&1 && pass || fail "valid issues.md failed"

# Test 11: E010 - Missing Issues section detected
echo -n "Test: E010 missing Issues section detected... "
output=$($APS lint "$SCRIPT_DIR/fixtures/invalid/issues-missing-section/issues.md" 2>&1) || true
echo "$output" | grep -q "E010" && pass || fail "E010 not detected"

# Test 12: W010 - Missing issue fields detected
echo -n "Test: W010 missing issue fields detected... "
output=$($APS lint "$SCRIPT_DIR/fixtures/invalid/issues-bad-fields/issues.md" 2>&1) || true
echo "$output" | grep -q "W010" && pass || fail "W010 not detected"

# Test 13: W011 - Missing question fields detected
echo -n "Test: W011 missing question fields detected... "
output=$($APS lint "$SCRIPT_DIR/fixtures/invalid/issues-bad-fields/issues.md" 2>&1) || true
echo "$output" | grep -q "W011" && pass || fail "W011 not detected"

# Test 14: JSON output works
echo -n "Test: JSON output valid... "
output=$($APS lint "$SCRIPT_DIR/fixtures/valid/" --json 2>&1)
echo "$output" | grep -q '"summary"' && pass || fail "JSON output invalid"

# Test 15: Dogfood - lint our own plans/
echo -n "Test: plans/ directory passes lint... "
$APS lint "$PROJECT_ROOT/plans/" > /dev/null 2>&1 && pass || fail "our own plans failed lint"

# Test 16: Orchestration suite (next, start, complete)
echo -n "Test: orchestrate (next/start/complete)... "
bash "$SCRIPT_DIR/orchestrate.sh" > /dev/null 2>&1 && pass || fail "orchestrate tests failed"

# Test 17: Minimal init by default; --local-cli opts into vendored runtime (INSTALL-011)
echo -n "Test: init is minimal by default... "
INIT_DIR=$(mktemp -d)
trap 'rm -rf "$INIT_DIR"' EXIT
APS_LOCAL="$PROJECT_ROOT" $APS init "$INIT_DIR" --profile solo --scope small --tools generic > /dev/null 2>&1 || fail "init failed"
# Core planning content + project contract are always present
[[ -f "$INIT_DIR/plans/index.aps.md" ]] || fail "plans/index.aps.md missing"
[[ -f "$INIT_DIR/plans/aps-rules.md" ]] || fail "plans/aps-rules.md missing"
[[ -f "$INIT_DIR/plans/project-context.md" ]] || fail "plans/project-context.md missing"
[[ -f "$INIT_DIR/.aps/config.yml" ]] || fail ".aps/config.yml missing"
grep -qF 'context/' "$INIT_DIR/.aps/.gitignore" || fail "context ignore missing"
# Minimal footprint: no vendored CLI, no hooks, no v1 scatter
[[ ! -d "$INIT_DIR/.aps/lib" ]] || fail ".aps/lib/ vendored without --local-cli"
[[ ! -d "$INIT_DIR/.aps/bin" ]] || fail ".aps/bin/ vendored without --local-cli"
[[ ! -d "$INIT_DIR/.aps/scripts" ]] || fail "hooks installed without --hooks"
[[ ! -d "$INIT_DIR/bin" ]] || fail "root bin/ created"
[[ ! -d "$INIT_DIR/lib" ]] || fail "root lib/ created"
[[ ! -d "$INIT_DIR/aps-planning" ]] || fail "root aps-planning/ created"
[[ ! -d "$INIT_DIR/.claude/commands" ]] || fail ".claude/commands/ created"
pass

# Test 17b: --local-cli vendors the bash runtime; --hooks installs hook scripts
echo -n "Test: init --local-cli vendors runtime... "
LOCALCLI_DIR=$(mktemp -d)
APS_LOCAL="$PROJECT_ROOT" $APS init "$LOCALCLI_DIR" --profile solo --scope small --tools generic --local-cli --hooks > /dev/null 2>&1 || fail "init --local-cli failed"
[[ -f "$LOCALCLI_DIR/.aps/lib/orchestrate.sh" ]] || fail "orchestrate lib not installed under --local-cli"
[[ -x "$LOCALCLI_DIR/.aps/bin/aps" ]] || fail ".aps/bin/aps not installed under --local-cli"
[[ -d "$LOCALCLI_DIR/.aps/scripts" ]] || fail "hooks not installed under --hooks"
rm -rf "$LOCALCLI_DIR"
pass

# Test 18: Generated agent artifacts and install include conductor
echo -n "Test: conductor agents install... "
[[ -f "$PROJECT_ROOT/scaffold/agents/claude-code/aps-conductor.md" ]] || fail "generated claude conductor missing"
[[ -f "$PROJECT_ROOT/scaffold/agents/copilot/aps-conductor.md" ]] || fail "generated copilot conductor missing"
[[ -f "$PROJECT_ROOT/scaffold/agents/codex/aps-conductor.toml" ]] || fail "generated codex conductor missing"
[[ -f "$PROJECT_ROOT/scaffold/agents/opencode/aps-conductor.md" ]] || fail "generated opencode conductor missing"
AGENT_DIR=$(mktemp -d)
APS_LOCAL="$PROJECT_ROOT" $APS init "$AGENT_DIR" --profile agent --scope small --tools claude-code,copilot,codex,opencode,gemini > /dev/null 2>&1 || fail "agent init failed"
[[ -f "$AGENT_DIR/.claude/agents/aps-conductor.md" ]] || fail "claude conductor missing"
[[ -f "$AGENT_DIR/.github/agents/aps-conductor.md" ]] || fail "copilot conductor missing"
[[ -f "$AGENT_DIR/.codex/agents/aps-conductor.toml" ]] || fail "codex conductor missing"
[[ -f "$AGENT_DIR/.opencode/agents/aps-conductor.md" ]] || fail "opencode conductor missing"
[[ -f "$AGENT_DIR/.gemini/skills/aps-conductor/SKILL.md" ]] || fail "gemini conductor missing"
grep -qF 'aps-conductor' "$AGENT_DIR/.codex/agents/codex-config-snippet.toml" || fail "codex conductor config missing"
rm -rf "$AGENT_DIR"
pass

# Test 19: Curl install/update scripts include CLI runtime libraries
echo -n "Test: curl installers include orchestration lib... "
grep -qF 'lib/orchestrate.sh' "$PROJECT_ROOT/scaffold/install" || fail "install omits lib/orchestrate.sh"
grep -qF 'lib/orchestrate.sh' "$PROJECT_ROOT/scaffold/update" || fail "update omits lib/orchestrate.sh"
pass

# Test 20: Legacy init script includes runtime files referenced by installed tools
echo -n "Test: legacy init includes referenced runtime files... "
grep -qF '"lib/rules/issues.sh"' "$PROJECT_ROOT/scaffold/init.sh" || fail "init omits lib/rules/issues.sh"
grep -qF 'enforce-plan-update.sh' "$PROJECT_ROOT/scaffold/init.sh" || fail "init omits enforce-plan-update.sh"
pass

# Test 21: MCP server (ORCH-006) — skipped when Node or its deps are unavailable
echo -n "Test: MCP server (ORCH-006)... "
if command -v node > /dev/null 2>&1 && [[ -d "$PROJECT_ROOT/mcp/node_modules" ]]; then
  (cd "$PROJECT_ROOT/mcp" && node --test > /dev/null 2>&1) && pass || fail "MCP server tests failed"
else
  echo "SKIP (node or mcp/node_modules unavailable)"
fi

# Test 22: W019 - Broken module link in index detected
echo -n "Test: W019 broken index link detected... "
output=$($APS lint "$SCRIPT_DIR/fixtures/invalid/index-broken-link/index.aps.md" 2>&1) || true
echo "$output" | grep -q "W019" && echo "$output" | grep -q "ghost.aps.md" || fail "W019 not detected"
echo "$output" | grep -q "example.com" && fail "W019 flagged external link" || pass
# Scaffold seed index must stay lint-clean (placeholder link is a warning, not error)
$APS lint "$PROJECT_ROOT/scaffold/plans" > /dev/null 2>&1 || fail "scaffold plans no longer lint clean"

# Test 23: W017 - Stale Last reviewed detected
echo -n "Test: W017 stale module detected... "
output=$($APS lint "$SCRIPT_DIR/fixtures/invalid/stale-module.aps.md" 2>&1) || true
echo "$output" | grep -q "W017" && pass || fail "W017 not detected"

# Test 24: W018 - Unaudited completion detected
echo -n "Test: W018 unaudited completion detected... "
output=$($APS lint "$SCRIPT_DIR/fixtures/invalid/unaudited-complete.aps.md" 2>&1) || true
echo "$output" | grep -q "UNA-001.*W018\|W018.*UNA-001" || fail "W018 not detected for UNA-001"
echo "$output" | grep -q "W018.*UNA-002\|UNA-002.*W018" && fail "W018 false positive on UNA-002"
# Future Last reviewed date (2099-01-01) must never read as stale
echo "$output" | grep -q "W017" && fail "W017 fired on future review date" || pass

# Test 25: W003 resolves dependencies across the plan tree
echo -n "Test: W003 cross-file dependency resolution... "
output=$($APS lint "$SCRIPT_DIR/fixtures/crossdep/" 2>&1) || true
echo "$output" | grep -q "W003.*GHOST-999\|GHOST-999.*W003" || fail "W003 missing for unknown ID"
echo "$output" | grep -qE "W003.*'(A-001|D-001)'" && fail "W003 false positive on cross-file ID" || pass

# Test 26: aps audit detects all four finding classes and exits non-zero
echo -n "Test: audit detects all finding classes... "
if output=$(cd "$PROJECT_ROOT" && $APS audit --plans test/fixtures/audit/plans 2>&1); then
  fail "expected exit 1 for broken audit fixture"
fi
for code in A001 A002 A003 A004; do
  echo "$output" | grep -q "$code" || fail "$code not reported"
done
echo "$output" | grep -q "DEMO-001.*PASS" || fail "PASS verification missing"
echo "$output" | grep -q "DEMO-003.*PARTIAL" || fail "PARTIAL verification missing"
echo "$output" | grep -q "DEMO-006.*PARTIAL.*command not found" || fail "unresolvable command not PARTIAL"
echo "$output" | grep -q "DEMO-007.*PARTIAL.*no Validation field" || fail "missing Validation not PARTIAL"
echo "$output" | grep -qE "A00[0-9].*DEMO-00[167]" && fail "false finding on DEMO-001/006/007"
# Exactly one A004: titled link and vscode:// scheme must not be flagged
[[ $(echo "$output" | grep -c "A004") -eq 1 ]] || fail "expected exactly one A004"
pass

# Test 27: aps audit --json emits valid JSON with finding codes
# (stderr carries the execution notice and must stay out of the JSON stream)
echo -n "Test: audit JSON output valid... "
output=$(cd "$PROJECT_ROOT" && $APS audit --plans test/fixtures/audit/plans --json 2>/dev/null) || true
echo "$output" | grep -q '"findings"' || fail "findings key missing"
if command -v python3 > /dev/null 2>&1; then
  echo "$output" | python3 -m json.tool > /dev/null 2>&1 || fail "invalid JSON"
fi
pass

# Test 28: aps audit --no-run skips validation execution
echo -n "Test: audit --no-run skips execution... "
output=$(cd "$PROJECT_ROOT" && $APS audit --plans test/fixtures/audit/plans --no-run 2>&1) || true
echo "$output" | grep -q "DEMO-002.*FAIL" && fail "--no-run still executed validation"
echo "$output" | grep -q "A001" && fail "A001 finding produced under --no-run"
echo "$output" | grep -q "DEMO-002.*PARTIAL" && pass || fail "PARTIAL not reported under --no-run"

# Test 29: module-scoped audit suppresses index link checks
echo -n "Test: audit module scope suppresses A004... "
output=$(cd "$PROJECT_ROOT" && $APS audit demo --plans test/fixtures/audit/plans --no-run 2>&1) || true
echo "$output" | grep -q "A004" && fail "A004 reported in module-scoped audit"
echo "$output" | grep -q "A003.*DEMO-005\|DEMO-005.*A003" && pass || fail "scoped audit missed A003"

# Test 30: Installer advertises all five modes (INSTALL-010)
echo -n "Test: installer advertises modes... "
INSTALL="$PROJECT_ROOT/scaffold/install"
for flag in --cli --init --agent --upgrade --setup; do
  grep -qF -- "$flag" "$INSTALL" || fail "install missing flag $flag"
done
grep -qE 'MODE="cli"|MODE="init"|MODE="agent"|MODE="upgrade"|MODE="setup"' "$INSTALL" || fail "install missing mode dispatch"
grep -q 'pick_mode' "$INSTALL" || fail "install missing TTY picker"
# PowerShell parity
grep -qF -- '--upgrade' "$PROJECT_ROOT/scaffold/install.ps1" || fail "install.ps1 missing --upgrade"
grep -q 'Select-ApsMode' "$PROJECT_ROOT/scaffold/install.ps1" || fail "install.ps1 missing picker"
pass

# Test 31: Installer argument guards reject bad input without network
echo -n "Test: installer rejects bad args... "
if bash "$INSTALL" --bogus </dev/null >/dev/null 2>&1; then
  fail "unknown flag should exit non-zero"
fi
if bash "$INSTALL" --setup </dev/null >/dev/null 2>&1; then
  fail "--setup without a tool should exit non-zero"
fi
if bash "$INSTALL" --init "" </dev/null >/dev/null 2>&1; then
  fail "empty TARGET should exit non-zero"
fi
if bash "$INSTALL" --agent /abs </dev/null >/dev/null 2>&1; then
  fail "absolute TARGET should be rejected for project modes"
fi
# No mode + no terminal MUST exit non-zero (not silently scaffold). setsid
# detaches the controlling terminal so /dev/tty is genuinely absent.
if bash "$INSTALL" </dev/null >/dev/null 2>&1; then
  fail "no-mode no-tty path should exit non-zero"
fi
if command -v setsid >/dev/null 2>&1; then
  if setsid bash "$INSTALL" </dev/null >/dev/null 2>&1; then
    fail "no-mode detached path should exit non-zero"
  fi
fi
pass

# Test 32: Each mode flag dispatches to its own path (marker in output)
echo -n "Test: installer mode dispatch... "
# Run from a temp dir so no project-local ./bin/aps is discovered; the setup
# gate must report no setup-capable CLI when none is on PATH or in TARGET.
out=$(cd "$(mktemp -d)" && PATH=/usr/bin:/bin bash "$INSTALL" --setup foo </dev/null 2>&1 || true)
echo "$out" | grep -qi "Add integration: foo" || fail "--setup did not reach setup path"
echo "$out" | grep -qi "setup is not available" || fail "--setup should gate on a setup-capable CLI"
# --upgrade against a dir with no plans/ fails fast before any network use
out=$(cd "$(mktemp -d)" && bash "$INSTALL" --upgrade </dev/null 2>&1 || true)
echo "$out" | grep -qi "nothing to upgrade" || fail "--upgrade did not reach upgrade path"
pass

# Test 33: curl installers init is minimal by default (INSTALL-011)
echo -n "Test: curl installer init is minimal... "
INSTALL="$PROJECT_ROOT/scaffold/install"
INSTALL_PS1="$PROJECT_ROOT/scaffold/install.ps1"
# No v1 scatter shipped by default: skill + legacy commands must be gone
grep -qE 'commands/plan|aps-planning/SKILL' "$INSTALL" && fail "install still ships skill/commands by default"
grep -qE 'commands/plan|aps-planning/SKILL' "$INSTALL_PS1" && fail "install.ps1 still ships skill/commands by default"
# The project contract is written
grep -q 'write_min_config' "$INSTALL" || fail "install missing write_min_config"
grep -qF 'config.yml' "$INSTALL_PS1" || fail "install.ps1 missing config.yml"
# Opt-in footprint flags exist on both
for flag in --local-cli --hooks; do
  grep -qF -- "$flag" "$INSTALL" || fail "install missing $flag"
  grep -qF -- "$flag" "$INSTALL_PS1" || fail "install.ps1 missing $flag"
done
# The vendored CLI must only land under .aps/ now (not root bin/), gated by the flag
grep -q 'USE_LOCAL_CLI' "$INSTALL" || fail "install missing USE_LOCAL_CLI gate"
pass

# Test 34: aps setup shortcuts and confirmation gating (INSTALL-012)
echo -n "Test: aps setup shortcuts... "
SETUP_DIR=$(mktemp -d)
APS_LOCAL="$PROJECT_ROOT" $APS init "$SETUP_DIR" --non-interactive >/dev/null 2>&1 || fail "init for setup failed"
# setup help advertises components
$APS setup --help 2>&1 | grep -qi "aps setup" || fail "setup help missing"
# Bare 'aps setup' with no terminal must not silently write — it errors out
if APS_LOCAL="$PROJECT_ROOT" $APS setup "$SETUP_DIR" </dev/null >/dev/null 2>&1; then
  fail "bare 'aps setup' should fail without a terminal"
fi
# hooks shortcut installs only hook scripts (no tools, no CLI)
APS_LOCAL="$PROJECT_ROOT" $APS setup hooks "$SETUP_DIR" </dev/null >/dev/null 2>&1 || fail "setup hooks failed"
[[ -d "$SETUP_DIR/.aps/scripts" ]] || fail "setup hooks did not install scripts"
[[ ! -d "$SETUP_DIR/.aps/lib" ]] || fail "setup hooks pulled in the CLI"
[[ ! -d "$SETUP_DIR/.claude/skills" ]] || fail "setup hooks pulled in a tool skill"
# tool shortcut installs only that tool's skill + agents
APS_LOCAL="$PROJECT_ROOT" $APS setup claude-code "$SETUP_DIR" </dev/null >/dev/null 2>&1 || fail "setup claude-code failed"
[[ -f "$SETUP_DIR/.claude/skills/aps-planning/SKILL.md" ]] || fail "setup claude-code missing skill"
[[ -f "$SETUP_DIR/.claude/agents/aps-planner.md" ]] || fail "setup claude-code missing agents"
# unknown target exits non-zero
if APS_LOCAL="$PROJECT_ROOT" $APS setup bogus "$SETUP_DIR" </dev/null >/dev/null 2>&1; then
  fail "unknown setup target should exit non-zero"
fi
# 'all' without --yes and without a terminal aborts (default no) — writes no CLI
APS_LOCAL="$PROJECT_ROOT" $APS setup all "$SETUP_DIR" </dev/null >/dev/null 2>&1 || true
[[ ! -d "$SETUP_DIR/.aps/lib" ]] || fail "setup all wrote CLI without confirmation"
# 'all --yes' installs the full footprint
APS_LOCAL="$PROJECT_ROOT" $APS setup all --yes "$SETUP_DIR" </dev/null >/dev/null 2>&1 || fail "setup all --yes failed"
[[ -f "$SETUP_DIR/.aps/lib/orchestrate.sh" ]] || fail "setup all --yes missing CLI"
[[ -d "$SETUP_DIR/.aps/scripts" ]] || fail "setup all --yes missing hooks"
rm -rf "$SETUP_DIR"
pass

# Test 35: aps upgrade — safe dry-run, backup-before-remove, protections (INSTALL-013)
echo -n "Test: aps upgrade cleanup... "
build_bulky() {
  local d="$1"
  mkdir -p "$d/plans/modules" "$d/aps-planning/scripts" "$d/.claude/commands" \
           "$d/bin" "$d/lib/rules" "$d/.aps/lib" "$d/.aps/bin"
  echo x > "$d/plans/index.aps.md"; echo r > "$d/plans/aps-rules.md"
  echo s > "$d/aps-planning/SKILL.md"; echo h > "$d/aps-planning/hooks.md"
  echo c > "$d/.claude/commands/plan.md"; echo c > "$d/.claude/commands/plan-status.md"
  echo b > "$d/bin/aps"; echo l > "$d/lib/lint.sh"; echo o > "$d/lib/output.sh"
  echo or > "$d/.aps/lib/orchestrate.sh"; echo b > "$d/.aps/bin/aps"
  echo claude > "$d/CLAUDE.md"; echo agents > "$d/AGENTS.md"
  printf '{"hooks":{"x":"aps-planning/scripts/init-session.sh"}}\n' > "$d/.claude/settings.local.json"
}
UP=$(mktemp -d); build_bulky "$UP"
# Dry run changes nothing and creates no backup
$APS upgrade "$UP" > /dev/null 2>&1 || fail "upgrade dry-run failed"
[[ -d "$UP/aps-planning" && -f "$UP/bin/aps" && -d "$UP/.aps/lib" ]] || fail "dry-run removed files"
[[ ! -d "$UP/.aps/backup" ]] || fail "dry-run created a backup"
# Apply backs up + removes generated bloat
$APS upgrade "$UP" --apply --yes > /dev/null 2>&1 || fail "upgrade --apply failed"
for gone in aps-planning bin/aps lib .aps/lib .aps/bin .claude/commands/plan.md; do
  [[ -e "$UP/$gone" ]] && fail "upgrade did not remove $gone"
done
for keep in plans/index.aps.md plans/aps-rules.md CLAUDE.md AGENTS.md; do
  [[ -e "$UP/$keep" ]] || fail "upgrade removed protected $keep"
done
[[ -n "$(find "$UP/.aps/backup" -name 'hooks.md' 2>/dev/null)" ]] || fail "backup missing removed files"
grep -q '\.aps/scripts/init-session.sh' "$UP/.claude/settings.local.json" || fail "hook path not rewritten"
grep -q 'aps-planning/scripts' "$UP/.claude/settings.local.json" && fail "stale hook path remains"
rm -rf "$UP"
# Guard: no plans/ -> nothing to upgrade
NOPLANS=$(mktemp -d)
$APS upgrade "$NOPLANS" > /dev/null 2>&1 && fail "upgrade should fail with no plans/"
noplans_out=$($APS upgrade "$NOPLANS" 2>&1 || true)
echo "$noplans_out" | grep -qi "nothing to upgrade" || fail "missing nothing-to-upgrade message"
rm -rf "$NOPLANS"
# Ambiguous lib/ (mixed files) is reported, never removed
AMB=$(mktemp -d); mkdir -p "$AMB/plans" "$AMB/lib"
echo x > "$AMB/plans/index.aps.md"; echo l > "$AMB/lib/lint.sh"; echo USER > "$AMB/lib/custom.sh"
$APS upgrade "$AMB" --apply --yes > /dev/null 2>&1 || true
[[ -f "$AMB/lib/custom.sh" ]] || fail "upgrade removed an ambiguous lib/"
rm -rf "$AMB"
pass

# Test 36: scaffold/upgrade curl entrypoint is dry-run-first and agent-safe
echo -n "Test: scaffold/upgrade entrypoint... "
UPSH="$PROJECT_ROOT/scaffold/upgrade"
[[ -f "$UPSH" ]] || fail "scaffold/upgrade missing"
bash -n "$UPSH" || fail "scaffold/upgrade has a syntax error"
grep -q 'APPLY=false' "$UPSH" || fail "scaffold/upgrade is not dry-run by default"
# Refuses to modify non-interactively without --yes
SAFE=$(mktemp -d); mkdir -p "$SAFE/plans" "$SAFE/bin"; echo x > "$SAFE/plans/index.aps.md"; echo b > "$SAFE/bin/aps"
if bash "$UPSH" "$SAFE" --apply </dev/null > /dev/null 2>&1; then
  fail "scaffold/upgrade applied without --yes non-interactively"
fi
[[ -f "$SAFE/bin/aps" ]] || fail "scaffold/upgrade removed files without --yes"
# Dry run reports the candidate and exits clean
bash "$UPSH" "$SAFE" </dev/null 2>&1 | grep -qi "dry run" || fail "scaffold/upgrade dry-run missing"
[[ -f "$SAFE/bin/aps" ]] || fail "scaffold/upgrade dry-run removed files"
rm -rf "$SAFE"
pass

echo ""
echo -e "${GREEN}All tests passed!${NC}"
