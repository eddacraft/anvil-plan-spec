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
out=$(bash "$INSTALL" --setup foo </dev/null 2>&1 || true)
echo "$out" | grep -qi "Add integration: foo" || fail "--setup did not reach setup path"
echo "$out" | grep -qi "setup is not available" || fail "--setup should gate on a setup-capable CLI"
# --upgrade against a dir with no plans/ fails fast before any network use
out=$(cd "$(mktemp -d)" && bash "$INSTALL" --upgrade </dev/null 2>&1 || true)
echo "$out" | grep -qi "nothing to upgrade" || fail "--upgrade did not reach upgrade path"
pass

echo ""
echo -e "${GREEN}All tests passed!${NC}"
