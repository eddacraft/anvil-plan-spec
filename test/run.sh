#!/usr/bin/env bash
#
# Simple test runner for APS CLI
#

set -eo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
APS="$PROJECT_ROOT/bin/aps"

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

# Test 17: Init installs CLI support files and ignores generated context
echo -n "Test: init installs orchestration support... "
INIT_DIR=$(mktemp -d)
trap 'rm -rf "$INIT_DIR"' EXIT
APS_LOCAL="$PROJECT_ROOT" $APS init "$INIT_DIR" --profile solo --scope small --tools generic > /dev/null 2>&1 || fail "init failed"
[[ -f "$INIT_DIR/.aps/lib/orchestrate.sh" ]] || fail "orchestrate lib not installed"
grep -qF 'context/' "$INIT_DIR/.aps/.gitignore" || fail "context ignore missing"
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

echo ""
echo -e "${GREEN}All tests passed!${NC}"
