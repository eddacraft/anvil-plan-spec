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

# Test 16b: Nested-plan orchestration (MONO-003) — federated traversal,
# child scoping, and cross-tree reference resolution.
echo -n "Test: orchestrate nested (federated next/start/graph/audit)... "
bash "$SCRIPT_DIR/orchestrate-nested.sh" > /dev/null 2>&1 && pass || fail "nested orchestration tests failed"

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

# Test 17a: nested (federated) scope scaffolds a root + wired child plans that
# lint clean as one federation (MONO-005).
echo -n "Test: init --scope nested scaffolds a lint-clean federation... "
NESTED_DIR=$(mktemp -d)/proj
APS_LOCAL="$PROJECT_ROOT" $APS init "$NESTED_DIR" --profile solo --scope nested --tools generic > /dev/null 2>&1 || fail "nested init failed"
[[ -f "$NESTED_DIR/plans/index.aps.md" ]] || fail "nested root index missing"
grep -q '^## Child Plans' "$NESTED_DIR/plans/index.aps.md" || fail "root missing ## Child Plans"
[[ -f "$NESTED_DIR/packages/core/plans/index.aps.md" ]] || fail "core child plan missing"
[[ -f "$NESTED_DIR/packages/api/plans/modules/module-name.aps.md" ]] || fail "api child module missing"
# Distinct per-package work-item prefixes (no W020 collision)
grep -q '### CORE-001' "$NESTED_DIR/packages/core/plans/modules/module-name.aps.md" || fail "core prefix not applied"
grep -q '### API-001' "$NESTED_DIR/packages/api/plans/modules/module-name.aps.md" || fail "api prefix not applied"
# The whole federation lints clean from the root
$APS lint "$NESTED_DIR/plans" > /dev/null 2>&1 || fail "scaffolded nested tree failed lint"
rm -rf "$(dirname "$NESTED_DIR")"
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
for codex_agent in "$PROJECT_ROOT"/scaffold/agents/codex/*.toml; do
  grep -Eq '^name = "[^"]+"$' "$codex_agent" || fail "$(basename "$codex_agent") missing Codex name"
  grep -Eq '^description = "[^"]+"$' "$codex_agent" || fail "$(basename "$codex_agent") missing Codex description"
  grep -q '^developer_instructions = """$' "$codex_agent" || fail "$(basename "$codex_agent") missing Codex instructions"
done
AGENT_DIR=$(mktemp -d)
mkdir -p "$AGENT_DIR/.codex/agents"
: > "$AGENT_DIR/.codex/agents/codex-config-snippet.toml"
APS_LOCAL="$PROJECT_ROOT" $APS init "$AGENT_DIR" --profile agent --scope small --tools claude-code,copilot,codex,opencode,grok,antigravity > /dev/null 2>&1 || fail "agent init failed"
[[ -f "$AGENT_DIR/.claude/agents/aps-conductor.md" ]] || fail "claude conductor missing"
[[ -f "$AGENT_DIR/.github/agents/aps-conductor.md" ]] || fail "copilot conductor missing"
[[ -f "$AGENT_DIR/.codex/agents/aps-conductor.toml" ]] || fail "codex conductor missing"
[[ -f "$AGENT_DIR/.opencode/agents/aps-conductor.md" ]] || fail "opencode conductor missing"
[[ -f "$AGENT_DIR/.agents/skills/aps-planning/SKILL.md" ]] || fail "grok/codex shared skill missing"
if find "$AGENT_DIR/.codex" -name 'codex-config-snippet.toml' -print -quit | grep -q .; then
  fail "obsolete Codex config snippet installed"
fi
for codex_agent in "$AGENT_DIR"/.codex/agents/*.toml; do
  grep -Eq '^name = "[^"]+"$' "$codex_agent" || fail "installed $(basename "$codex_agent") missing Codex name"
  grep -Eq '^description = "[^"]+"$' "$codex_agent" || fail "installed $(basename "$codex_agent") missing Codex description"
  grep -q '^developer_instructions = """$' "$codex_agent" || fail "installed $(basename "$codex_agent") missing Codex instructions"
done
rm -rf "$AGENT_DIR"
pass

# Test 18b: Retired gemini tool is rejected with a D-040 pointer
echo -n "Test: gemini tool rejected (D-040)... "
GEMINI_DIR=$(mktemp -d)
if APS_LOCAL="$PROJECT_ROOT" $APS init "$GEMINI_DIR" --profile agent --scope small --tools gemini > /dev/null 2>&1; then
  fail "gemini should be rejected (D-040)"
fi
rm -rf "$GEMINI_DIR"
pass

# Test 18c: ISS-001 — fenced example items are inert in lint and orchestration
echo -n "Test: fenced example items are inert (ISS-001)... "
lint_out=$($APS lint "$SCRIPT_DIR/fixtures/valid/fenced-examples.aps.md" 2>&1) || fail "lint errored on fenced fixture"
if echo "$lint_out" | grep -qE "E005|W003|FAKE-999|TILDE-777"; then
  fail "fenced example leaked into lint findings"
fi
FENCE_DIR=$(mktemp -d)
mkdir -p "$FENCE_DIR/modules"
cp "$SCRIPT_DIR/fixtures/valid/fenced-examples.aps.md" "$FENCE_DIR/modules/"
graph_out=$($APS graph --plans "$FENCE_DIR" 2>&1) || fail "graph errored on fenced fixture"
if echo "$graph_out" | grep -qE "FAKE-999|TILDE-777"; then
  fail "fenced example leaked into graph"
fi
echo "$graph_out" | grep -q "FENCE-001" || fail "real item missing from graph"
rm -rf "$FENCE_DIR"
pass

# Test 18d: aps export emits deterministic aps-export/v1 JSON
echo -n "Test: export emits deterministic JSON... "
exp1=$($APS export --plans "$SCRIPT_DIR/fixtures/valid" 2>&1) || fail "export failed"
[[ "$exp1" == '{"schema":"aps-export/v1"'* ]] || fail "unexpected schema header"
exp2=$($APS export --plans "$SCRIPT_DIR/fixtures/valid" 2>&1) || fail "second export failed"
[[ "$exp1" == "$exp2" ]] || fail "export not deterministic"
if command -v python3 >/dev/null 2>&1; then
  printf '%s' "$exp1" | python3 -m json.tool > /dev/null || fail "export is not valid JSON"
fi
if echo "$exp1" | grep -q "FAKE-999"; then
  fail "fenced example leaked into export"
fi
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
grep -q 'TARGET exists and is not a directory' "$INSTALL" || fail "bash onboarding does not reject file TARGETs"
grep -q 'TARGET exists and is not a directory' "$PROJECT_ROOT/scaffold/install.ps1" || fail "PowerShell onboarding does not reject file TARGETs"
grep -q 'New-Item -ItemType Directory -Path $Target' "$PROJECT_ROOT/scaffold/install.ps1" || fail "PowerShell onboarding does not create a missing TARGET"
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
ONBOARD_ROOT=$(mktemp -d)
: > "$ONBOARD_ROOT/not-a-directory"
onboard_error=$(cd "$ONBOARD_ROOT" && bash "$INSTALL" --onboard not-a-directory </dev/null 2>&1 || true)
echo "$onboard_error" | grep -q 'TARGET exists and is not a directory' \
  || fail "onboarding file TARGET should fail clearly before network access"
rm -rf "$ONBOARD_ROOT"
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

# Test 32a: Native installer onboarding hands off to aps init (CIB-002)
echo -n "Test: installer hands off to native init... "
grep -qF -- '--onboard' "$INSTALL" || fail "install missing explicit onboarding mode"
grep -q 'Invoke-ApsOnboarding' "$PROJECT_ROOT/scaffold/install.ps1" \
  || fail "install.ps1 missing native onboarding handoff"
if command -v script >/dev/null 2>&1; then
  HANDOFF_DIR=$(mktemp -d)
  mkdir -p "$HANDOFF_DIR/home/.aps/bin" "$HANDOFF_DIR/payload" "$HANDOFF_DIR/mock-bin"
  cat > "$HANDOFF_DIR/payload/aps" <<'EOF'
#!/usr/bin/env bash
printf '%s\n' "$*" >> "$APS_HANDOFF_LOG"
EOF
  chmod +x "$HANDOFF_DIR/payload/aps"
  tar -czf "$HANDOFF_DIR/aps.tar.gz" -C "$HANDOFF_DIR/payload" aps
  cat > "$HANDOFF_DIR/mock-bin/curl" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
dest=""
while [[ $# -gt 0 ]]; do
  if [[ "$1" == "-o" ]]; then
    dest="$2"
    shift 2
  else
    shift
  fi
done
cp "$APS_TEST_ARCHIVE" "$dest"
EOF
  chmod +x "$HANDOFF_DIR/mock-bin/curl"
  APS_HOME="$HANDOFF_DIR/home/.aps" \
    APS_TEST_ARCHIVE="$HANDOFF_DIR/aps.tar.gz" \
    APS_HANDOFF_LOG="$HANDOFF_DIR/handoff.log" \
    PATH="$HANDOFF_DIR/mock-bin:$HANDOFF_DIR/home/.aps/bin:$PATH" \
    script -q -e -c "cd '$HANDOFF_DIR' && bash '$INSTALL' --onboard project" /dev/null \
      </dev/null >/dev/null 2>&1 \
    || fail "native onboarding path failed"
  [[ -d "$HANDOFF_DIR/project" ]] \
    || fail "native onboarding did not create the missing TARGET"
  grep -qxF 'init' "$HANDOFF_DIR/handoff.log" \
    || fail "native onboarding did not invoke aps init"
  rm -rf "$HANDOFF_DIR"
fi
pass

# Test 32b: User documentation keeps PowerShell first-class (CIB-004)
echo -n "Test: PowerShell user documentation contract... "
for doc in "$PROJECT_ROOT/README.md" "$PROJECT_ROOT/docs/installation.md" "$PROJECT_ROOT/docs/usage.md"; do
  if grep -qE 'Use WSL/Git Bash|should be run from WSL|depend on the bash runtime' "$doc"; then
    fail "$(basename "$doc") still requires Bash for Windows users"
  fi
done
grep -qF 'scaffold/install.ps1' "$PROJECT_ROOT/README.md" \
  || fail "README missing PowerShell installer"
grep -qF 'scaffold/update.ps1' "$PROJECT_ROOT/docs/installation.md" \
  || fail "installation docs missing PowerShell updater"
grep -qF 'No user command requires WSL or Git Bash' "$PROJECT_ROOT/docs/usage.md" \
  || fail "usage docs missing native PowerShell contract"
grep -qF 'runs-on: windows-latest' "$PROJECT_ROOT/.github/workflows/ci.yml" \
  || fail "CI missing native Windows runner"
grep -qF 'test/windows-user-journey.ps1' "$PROJECT_ROOT/.github/workflows/ci.yml" \
  || fail "CI missing PowerShell user-journey harness"
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

# Test 37: .aps/config.yml project contract — cli_version + path keys (INSTALL-014)
echo -n "Test: config.yml project contract... "
CFG_DIR=$(mktemp -d)
APS_LOCAL="$PROJECT_ROOT" $APS init "$CFG_DIR" --non-interactive >/dev/null 2>&1 || fail "init for config test failed"
CFG="$CFG_DIR/.aps/config.yml"
grep -qE '^cli_version:' "$CFG" || fail "config missing cli_version"
grep -qE '^plans_dir:' "$CFG" || fail "config missing plans_dir"
grep -qE '^docs_dir:' "$CFG" || fail "config missing docs_dir"
grep -qE '^tooling_root:' "$CFG" || fail "config missing tooling_root"
rm -rf "$CFG_DIR"
# Alternate plans_dir fixture exists and pins a non-default tree
FIX="$PROJECT_ROOT/test/fixtures/config/alt-plans-dir.yml"
[[ -f "$FIX" ]] || fail "alt-plans-dir fixture missing"
grep -qE '^plans_dir: docs/plans/' "$FIX" || fail "fixture lacks alternate plans_dir"
grep -qE '^cli_version:' "$FIX" || fail "fixture lacks cli_version"
# This repo dogfoods the contract
grep -qE '^cli_version:' "$PROJECT_ROOT/.aps/config.yml" || fail "repo .aps/config.yml missing cli_version"
pass

# Test 38: runtime project config discovery — alternate plans_dir + --strict (INSTALL-016)
echo -n "Test: runtime config discovery... "
DISC=$(mktemp -d)
mkdir -p "$DISC/docs/plans" "$DISC/.aps"
cp "$PROJECT_ROOT/scaffold/plans/index.aps.md" "$DISC/docs/plans/index.aps.md"
printf 'cli_version: %s\nplans_dir: docs/plans/\n' "${APS_CLI_VERSION:-0.3.0}" > "$DISC/.aps/config.yml"
# `aps lint` with no args discovers plans_dir and lints the alternate tree
( cd "$DISC" && "$APS" lint >/dev/null 2>&1 ) || fail "lint did not discover alternate plans_dir"
# Mismatched cli_version: warns but succeeds without --strict, fails with it
printf 'cli_version: 9.9.9\nplans_dir: docs/plans/\n' > "$DISC/.aps/config.yml"
disc_warn=$( cd "$DISC" && "$APS" lint 2>&1 || true )
echo "$disc_warn" | grep -qi "pins cli_version 9.9.9" || fail "no cli_version mismatch warning"
( cd "$DISC" && "$APS" lint >/dev/null 2>&1 ) || fail "non-strict mismatch should still pass"
if ( cd "$DISC" && "$APS" lint --strict >/dev/null 2>&1 ); then
  fail "--strict should fail on cli_version mismatch"
fi
# APS_PLANS overrides discovery (MCP/manual escape hatch)
( cd "$DISC" && APS_PLANS=docs/plans "$APS" lint >/dev/null 2>&1 ) || fail "APS_PLANS override failed"
# A bare repo with no config falls back to plans/
NOCFG=$(mktemp -d); mkdir -p "$NOCFG/plans"
cp "$PROJECT_ROOT/scaffold/plans/index.aps.md" "$NOCFG/plans/index.aps.md"
( cd "$NOCFG" && "$APS" lint >/dev/null 2>&1 ) || fail "fallback to plans/ failed"
rm -rf "$DISC" "$NOCFG"
pass

# Test 39: global binary-first install channels (INSTALL-015)
echo -n "Test: global binary-first install channels... "
INSTALL="$PROJECT_ROOT/scaffold/install"
INSTALL_PS1="$PROJECT_ROOT/scaffold/install.ps1"
# Bash cli install is binary-first: prefers the release binary unless --bash
grep -q 'USE_LOCAL_CLI.*!=.*true.*&&.*install_binary' "$INSTALL" \
  || fail "install_global is not binary-first"
grep -qF -- '--bash' "$INSTALL" || fail "install missing --bash force"
# --binary requires the binary (no silent bash fallback)
grep -q 'binary requested but no release binary' "$INSTALL" \
  || fail "install --binary should hard-fail without fallback"
grep -q 'binary requested but no release binary' "$INSTALL_PS1" \
  || fail "install.ps1 --binary should hard-fail without fallback"
# Bash lib fallback manifest still ships the full set incl. audit.sh
grep -q 'lib/audit.sh' "$INSTALL" || fail "global lib manifest missing audit.sh"
# PowerShell mirror: binary install path + User PATH + aps.exe
grep -q 'Install-ApsBinary' "$INSTALL_PS1" || fail "install.ps1 missing binary install"
grep -q 'aps.exe' "$INSTALL_PS1" || fail "install.ps1 missing aps.exe"
grep -q 'Get-ApsReleaseTarget' "$INSTALL_PS1" || fail "install.ps1 missing target detection"
# Cargo.toml: binstall metadata + crates.io readiness, publish blocker documented
CARGO="$PROJECT_ROOT/cli/Cargo.toml"
grep -q 'package.metadata.binstall' "$CARGO" || fail "Cargo.toml missing binstall metadata"
grep -q '^keywords' "$CARGO" || fail "Cargo.toml missing crates.io keywords"
grep -q '^publish = true' "$CARGO" || fail "Cargo.toml publish flag not enabled"
grep -qi 'crates.io' "$CARGO" || fail "Cargo.toml missing crates.io publish note"
# Scoop manifest is valid JSON with the Windows asset + autoupdate
SCOOP="$PROJECT_ROOT/packaging/scoop/aps.json"
[[ -f "$SCOOP" ]] || fail "scoop manifest missing"
if command -v python3 >/dev/null 2>&1; then
  python3 -c "import json,sys; json.load(open('$SCOOP'))" || fail "scoop manifest is not valid JSON"
fi
grep -q 'aps-x86_64-pc-windows-gnu.zip' "$SCOOP" || fail "scoop manifest missing windows asset"
grep -q 'autoupdate' "$SCOOP" || fail "scoop manifest missing autoupdate"
# Release workflow documents the bump checklist (tag -> assets -> crates.io -> Scoop)
REL="$PROJECT_ROOT/.github/workflows/release.yml"
grep -qi 'bump checklist' "$REL" || fail "release.yml missing bump checklist"
for kw in 'crates.io' 'Scoop' 'binstall'; do
  grep -qi "$kw" "$REL" || fail "release.yml checklist missing $kw"
done
pass

# Test 40: binary-first project init — docs + picker reflect no default vendoring (INSTALL-018)
echo -n "Test: binary-first project init... "
DOCS="$PROJECT_ROOT/docs/installation.md"
INSTALL="$PROJECT_ROOT/scaffold/install"
# "What Gets Installed" leads with binary-first minimal, marks .aps/config.yml required
grep -qi 'binary-first and minimal' "$DOCS" || fail "docs missing binary-first minimal framing"
grep -q 'config.yml.*required\|required.*config.yml' "$DOCS" || fail "docs do not mark config.yml required"
# Vendored CLI is documented as opt-in (--local-cli), not default
grep -q 'only with --local-cli' "$DOCS" || fail "docs do not mark vendored CLI opt-in"
# Manual setup no longer tells users to copy bin/ + lib/ as the default step
grep -q 'Copy .bin/aps. and .lib/. into your project' "$DOCS" \
  && fail "manual setup still defaults to vendoring bin/ + lib/"
# TTY picker + non-interactive next-steps offer the global CLI before repo init
cli_line=$(grep -n 'Install the APS CLI on this machine' "$INSTALL" | head -1 | cut -d: -f1)
init_line=$(grep -n 'Initialize APS planning in this repository' "$INSTALL" | head -1 | cut -d: -f1)
[[ -n "$cli_line" && -n "$init_line" && "$cli_line" -lt "$init_line" ]] \
  || fail "picker should offer CLI install before repo init"
pass

# Test 41: aps doctor migration diagnostics (INSTALL-017)
echo -n "Test: aps doctor migration diagnostics... "
# doctor is a native-binary command; the migration docs + command wiring must
# exist, and the built binary (when present) must flag a bloated v1 project.
grep -qi 'Migrating to the Global Binary' "$PROJECT_ROOT/docs/installation.md" \
  || fail "installation.md missing migration section"
grep -qF 'aps doctor' "$PROJECT_ROOT/docs/usage.md" || fail "usage.md missing aps doctor"
grep -q 'Command::Doctor' "$PROJECT_ROOT/cli/src/main.rs" || fail "main.rs missing Doctor command"
DOCTOR_BIN="$PROJECT_ROOT/cli/target/release/aps"
if [[ -x "$DOCTOR_BIN" ]]; then
  DOC=$(mktemp -d)
  mkdir -p "$DOC/bin" "$DOC/lib" "$DOC/.aps"
  printf '#!/usr/bin/env bash\n' > "$DOC/bin/aps"
  printf '# bash lint\n' > "$DOC/lib/lint.sh"
  printf 'cli_version: 0.0.1\n' > "$DOC/.aps/config.yml"
  printf 'PATH_add bin\n' > "$DOC/.envrc"
  out=$( cd "$DOC" && "$DOCTOR_BIN" doctor 2>&1 || true )
  echo "$out" | grep -qi 'vendored CLI' || fail "doctor did not flag vendored CLI"
  echo "$out" | grep -qi 'direnv' || fail "doctor did not flag stale direnv"
  echo "$out" | grep -qi 'cli_version' || fail "doctor did not report cli_version"
  rm -rf "$DOC"
  pass
else
  echo "SKIP (cli/target/release/aps not built)"
fi

# Test 42: MONO-002 — federated parent root follows ## Child Plans links
echo -n "Test: nested-plans traversal lints children from parent root... "
output=$($APS lint "$SCRIPT_DIR/fixtures/monorepo/plans" 2>&1) || true
echo "$output" | grep -q "5 files checked" || fail "parent root did not traverse into child plans"
echo "$output" | grep -q "core/plans/index.aps.md" || fail "core child not linted from parent root"
echo "$output" | grep -q "api/plans/modules/handlers.aps.md" || fail "api child module not linted from parent root"
echo "$output" | grep -q "W003" && fail "cross-tree dep core:AUTH-001 should resolve in federated lint" || pass

# Test 43: MONO-002 — bad cross-tree ref warns when the named child is in scope
echo -n "Test: nested-plans bad cross-tree ref flagged in federated lint... "
BADREF=$(mktemp -d)
cp -r "$SCRIPT_DIR/fixtures/monorepo/." "$BADREF/"
HFILE="$BADREF/packages/api/plans/modules/handlers.aps.md"
awk '{ gsub(/core:AUTH-001/, "core:AUTH-999"); print }' "$HFILE" > "$HFILE.tmp" && mv "$HFILE.tmp" "$HFILE"
output=$($APS lint "$BADREF/plans" 2>&1) || true
echo "$output" | grep -q "W003" && echo "$output" | grep -q "core:AUTH-999" || { rm -rf "$BADREF"; fail "bad cross-tree ref not flagged"; }
rm -rf "$BADREF"
pass

# Test 44: MONO-002 — isolated child with a cross-tree ref still lints clean
echo -n "Test: nested-plans isolated child lints clean... "
output=$($APS lint "$SCRIPT_DIR/fixtures/monorepo/packages/api/plans" 2>&1) || true
echo "$output" | grep -q "W003" && fail "isolated child should not flag external cross-tree ref" || pass

# Test 45: MONO-002 — W020 work-item ID collision across child trees
echo -n "Test: nested-plans cross-tree ID collision detected (W020)... "
COL=$(mktemp -d)
cp -r "$SCRIPT_DIR/fixtures/monorepo/." "$COL/"
CF="$COL/packages/api/plans/modules/handlers.aps.md"
awk '{ gsub(/HND-001/, "AUTH-001"); print }' "$CF" > "$CF.tmp" && mv "$CF.tmp" "$CF"
output=$($APS lint "$COL/plans" 2>&1) || true
echo "$output" | grep -q "W020" && echo "$output" | grep -q "AUTH-001" || { rm -rf "$COL"; fail "W020 collision not detected"; }
rm -rf "$COL"
# the clean fixture must NOT trip W020
output=$($APS lint "$SCRIPT_DIR/fixtures/monorepo/plans" 2>&1) || true
echo "$output" | grep -q "W020" && fail "W020 false positive on clean fixture" || pass

# Test 46: MONO-002 — PowerShell parity for nested-plans traversal/W003/W020
# The bash linter is canonical; lib/*.psm1 mirrors it. CI carries no pwsh, so
# guard parity by string-checking the ported surface (same approach as the
# install.ps1 parity checks above).
echo -n "Test: nested-plans PowerShell parity surface present... "
PS_LINT="$PROJECT_ROOT/lib/Lint.psm1"
PS_WI="$PROJECT_ROOT/lib/rules/WorkItem.psm1"
grep -q 'function Expand-ApsChildPlans' "$PS_LINT" || fail "Lint.psm1 missing Expand-ApsChildPlans (child-plan traversal)"
grep -q 'function Build-ApsChildRegistry' "$PS_LINT" || fail "Lint.psm1 missing Build-ApsChildRegistry"
grep -q 'function Test-ApsCrossTreeCollisions' "$PS_LINT" || fail "Lint.psm1 missing Test-ApsCrossTreeCollisions (W020)"
grep -q 'W020' "$PS_LINT" || fail "Lint.psm1 missing W020 code"
grep -q 'function Build-ApsChildModuleRegistry' "$PS_LINT" || fail "Lint.psm1 missing Build-ApsChildModuleRegistry"
grep -q 'function Test-ApsCrossTreeModuleCollisions' "$PS_LINT" || fail "Lint.psm1 missing Test-ApsCrossTreeModuleCollisions (W021)"
grep -q 'W021' "$PS_LINT" || fail "Lint.psm1 missing W021 code"
grep -qF 'Cross-tree dependency' "$PS_WI" || fail "WorkItem.psm1 missing prefix-aware W003"
grep -qF '[a-z0-9][a-z0-9-]*:' "$PS_WI" || fail "WorkItem.psm1 missing <name>:<ID> token grammar"
pass

# Test 47: MONO-007 — Rust parity for nested-plans traversal/W003/W020.
# The Rust binary is the canonical `aps` (D-031); cli/src/lint.rs must carry the
# same federated-lint surface as bash (lib/lint.sh) and PowerShell. This CI job
# has no cargo, so — like the pwsh check above — guard by string-matching the
# ported surface; byte-for-byte behaviour is asserted by the cargo tests over
# test/fixtures/monorepo/ (see cli/src/lint.rs, run by the `cargo test` job).
echo -n "Test: nested-plans Rust parity surface present... "
RS_LINT="$PROJECT_ROOT/cli/src/lint.rs"
grep -q 'fn expand_child_plans' "$RS_LINT" || fail "lint.rs missing expand_child_plans (child-plan traversal)"
grep -q 'fn build_child_registry' "$RS_LINT" || fail "lint.rs missing build_child_registry"
grep -q 'fn check_cross_tree_collisions' "$RS_LINT" || fail "lint.rs missing check_cross_tree_collisions (W020)"
grep -q '"W020"' "$RS_LINT" || fail "lint.rs missing W020 code"
grep -q 'fn build_child_module_registry' "$RS_LINT" || fail "lint.rs missing build_child_module_registry"
grep -q 'fn check_cross_tree_module_collisions' "$RS_LINT" || fail "lint.rs missing check_cross_tree_module_collisions (W021)"
grep -q '"W021"' "$RS_LINT" || fail "lint.rs missing W021 code"
grep -qF 'Cross-tree dependency' "$RS_LINT" || fail "lint.rs missing prefix-aware W003"
grep -qF '[a-z0-9][a-z0-9-]*:' "$RS_LINT" || fail "lint.rs missing <name>:<ID> token grammar"
pass

# Test 48: COND-007 — conductor lint rules W002/W006 (bash, canonical behaviour)
echo -n "Test: conductor typo (W002) and mislisted index entry (W006) detected... "
output=$($APS lint "$SCRIPT_DIR/fixtures/conductor/plans" 2>&1) || true
echo "$output" | grep -q "W002" && echo "$output" | grep -q "AUTH-999" \
  || fail "W002 conductor cross-ref typo not detected"
echo "$output" | grep -q "W006" && echo "$output" | grep -q "auth.aps.md" \
  || fail "W006 mislisted conductor index entry not detected"
# the clean conductor fixture must NOT trip W002 or W006
output=$($APS lint "$SCRIPT_DIR/fixtures/conductor-clean/plans" 2>&1) || true
echo "$output" | grep -qE "W002|W006" && fail "W002/W006 false positive on clean conductor fixture" || pass

# Test 49: COND-007 — PowerShell parity for conductor rules W002/W006.
# The bash linter is canonical; lib/*.psm1 mirrors it. CI carries no pwsh, so
# guard parity by string-checking the ported surface (same approach as the
# nested-plans checks above). Behaviour was verified against a fetched pwsh
# 7.4 producing byte-identical W002/W006 lines to bash and Rust.
echo -n "Test: conductor rules PowerShell parity surface present... "
PS_COMMON="$PROJECT_ROOT/lib/rules/Common.psm1"
PS_MODULE="$PROJECT_ROOT/lib/rules/Module.psm1"
PS_INDEX="$PROJECT_ROOT/lib/rules/Index.psm1"
grep -q 'function Get-ApsModuleType' "$PS_COMMON" || fail "Common.psm1 missing Get-ApsModuleType"
grep -q 'function Test-ApsConductor' "$PS_COMMON" || fail "Common.psm1 missing Test-ApsConductor"
grep -q 'function Test-W002ConductorRefs' "$PS_MODULE" || fail "Module.psm1 missing Test-W002ConductorRefs"
grep -q 'W002' "$PS_MODULE" || fail "Module.psm1 missing W002 code"
grep -q 'function Test-W006ConductorIndex' "$PS_INDEX" || fail "Index.psm1 missing Test-W006ConductorIndex"
grep -q 'W006' "$PS_INDEX" || fail "Index.psm1 missing W006 code"
pass

# Test 51: COND-007 — W017-before-W002 emission order for an active conductor.
# Rust's lint_module emits W017 before the conductor W002 check; the bash and
# PowerShell ports must match so a byte-level diff of lint output stays clean
# across CLIs. The conductor/ fixture is Status: Complete (W017 short-circuits),
# so this fixture is Ready with a missing Last reviewed field AND a bad cross-ref
# to actually exercise both rules together.
echo -n "Test: active conductor emits W017 before W002 (Rust order parity)... "
COF="$SCRIPT_DIR/fixtures/conductor-order/plans/modules/release-planning.aps.md"
output=$($APS lint "$COF" 2>&1) || true
w017_ln=$(printf '%s\n' "$output" | grep -n "W017" | head -1 | cut -d: -f1)
w002_ln=$(printf '%s\n' "$output" | grep -n "W002" | head -1 | cut -d: -f1)
[[ -n "$w017_ln" && -n "$w002_ln" ]] || fail "expected both W017 and W002 on active conductor (got: $output)"
[[ "$w017_ln" -lt "$w002_ln" ]] || fail "W017 must be emitted before W002 (Rust lint_module order)"
pass

# Test 52: COND-007 — Get-ApsStatus separator-row parity guard (PowerShell).
# A spaced `| --- |` separator must not be read as the status data row, else
# W005/W017/W018 status gating silently never matches in PowerShell.
echo -n "Test: PowerShell Get-ApsStatus skips spaced separator rows... "
grep -qF 'returned "------" as the status' "$PS_COMMON" || fail "Common.psm1 Get-ApsStatus missing separator-row skip"
grep -qF '[math]::Floor' "$PS_MODULE" || fail "Module.psm1 W017 must floor age days to match bash/Rust"
pass

# Test 50: COND-007 — Rust parity for conductor rules W002/W006.
# The Rust binary is the canonical `aps` (D-031); cli/src/lint.rs must carry the
# same conductor-lint surface as bash and PowerShell. This CI job has no cargo,
# so guard by string-matching the surface; byte-for-byte behaviour is asserted
# by the cargo tests in cli/src/lint.rs (run by the `cargo test` job).
echo -n "Test: conductor rules Rust parity surface present... "
RS_LINT="$PROJECT_ROOT/cli/src/lint.rs"
grep -q 'fn check_w002_conductor_refs' "$RS_LINT" || fail "lint.rs missing check_w002_conductor_refs"
grep -q 'fn check_w006_conductor_index' "$RS_LINT" || fail "lint.rs missing check_w006_conductor_index"
grep -q '"W002"' "$RS_LINT" || fail "lint.rs missing W002 code"
grep -q '"W006"' "$RS_LINT" || fail "lint.rs missing W006 code"
pass

# Test 53: CIP-002 — cross-CLI parity harness present and wired into CI.
# The harness itself (test/cli-parity.sh) needs the Rust binary + pwsh and runs
# in its own `cli-parity` CI job; this guard just keeps it from silently
# disappearing (same rationale as the string guards above).
echo -n "Test: cross-CLI parity harness present and wired into CI... "
[[ -x "$PROJECT_ROOT/test/cli-parity.sh" ]] || fail "test/cli-parity.sh missing or not executable"
grep -q 'cli-parity.sh' "$PROJECT_ROOT/.github/workflows/ci.yml" || fail "ci.yml does not run test/cli-parity.sh"
pass

# Test 54: PKG-002 — W022 Packages: tag validation (bash, canonical behaviour).
# A tag that resolves to no workspace directory warns; resolvable tags and
# single-package projects (no packages/ or apps/ dirs) stay silent. PowerShell
# and Rust behaviour is asserted byte-for-byte by test/cli-parity.sh, which
# carries the pkgtags fixtures.
echo -n "Test: unresolvable Packages: tag detected (W022)... "
output=$($APS lint "$SCRIPT_DIR/fixtures/pkgtags/plans" 2>&1) || true
echo "$output" | grep -q "W022" && echo "$output" | grep -q "storefront" \
  || fail "W022 not raised for typo'd Packages entry"
echo "$output" | grep -q "packages/wrong" || fail "W022 missed path-as-given typo"
output=$($APS lint "$SCRIPT_DIR/fixtures/pkgtags-clean/plans" 2>&1) || true
echo "$output" | grep -q "no issues" || fail "clean pkgtags fixture did not lint cleanly (got: $output)"
output=$($APS lint "$SCRIPT_DIR/fixtures/pkgtags-nomarker/plans" 2>&1) || true
echo "$output" | grep -q "W022" && fail "W022 fired without monorepo markers" 
echo "$output" | grep -q "no issues" || fail "nomarker fixture did not lint cleanly (got: $output)"
pass

# Test 55: PKG-001 — aps next --package / --by-package (bash, canonical).
# Rust behaviour is asserted by cargo tests over the same shared fixture;
# byte parity was verified by diffing both CLIs across all modes.
echo -n "Test: next --package filters and --by-package groups... "
output=$($APS next --plans "$SCRIPT_DIR/fixtures/pkgnext/plans" --package api 2>&1) || fail "next --package api failed"
echo "$output" | grep -q "HND-001" || fail "--package api did not resolve HND-001 (item-level apps/api tag)"
output=$($APS next --plans "$SCRIPT_DIR/fixtures/pkgnext/plans" --package core 2>&1) || fail "next --package core failed"
echo "$output" | grep -q "AUTH-001" || fail "--package core did not resolve AUTH-001 (inherited module tag)"
if $APS next --plans "$SCRIPT_DIR/fixtures/pkgnext/plans" --package ghost >/dev/null 2>&1; then
  fail "--package ghost should exit non-zero"
fi
output=$($APS next --plans "$SCRIPT_DIR/fixtures/pkgnext/plans" --by-package 2>&1) || fail "next --by-package failed"
echo "$output" | grep -q "^core:$" || fail "--by-package missing core group"
echo "$output" | grep -q "^(untagged):$" || fail "--by-package missing (untagged) bucket"
echo "$output" | tail -1 | grep -q "MISC-001" || fail "(untagged) bucket must come last"
pass

# Test 56: PKG-003 — rollup --by-package renders the grouped module view.
echo -n "Test: rollup --by-package groups modules with (untagged) last... "
output=$($APS rollup --plans "$SCRIPT_DIR/fixtures/pkgnext/plans" --by-package 2>&1) || fail "rollup --by-package failed"
echo "$output" | grep -q "^### core$" || fail "missing core heading"
echo "$output" | grep -q "AUTH | In Progress" || fail "missing tagged module row"
echo "$output" | grep -q "^### (untagged)$" || fail "missing (untagged) heading"
echo "$output" | awk '/### \(untagged\)/{f=1} f && /MISC/{found=1} END{exit !found}' || fail "MISC not under (untagged)"
pass

# Test 57: INSTALL-020 / D-042 — managed skill markers: bash writes the
# canonical `.aps-managed.json` on skill install and `aps update` reconciles
# each tree by marker state instead of blind-overwriting.
echo -n "Test: managed skill markers (bash write + reconcile)... "
MNG_DIR=$(mktemp -d)
MNG_SKILL="$MNG_DIR/.claude/skills/aps-planning"
MNG_MARKER="$MNG_SKILL/.aps-managed.json"
APS_LOCAL="$PROJECT_ROOT" $APS init "$MNG_DIR" --profile solo --scope small --tools claude-code > /dev/null 2>&1 || fail "init with claude-code failed"
[[ -f "$MNG_MARKER" ]] || fail "marker not written on install"
grep -q '"schemaVersion": 1' "$MNG_MARKER" || fail "marker missing schemaVersion"
grep -q '"kind": "skill"' "$MNG_MARKER" || fail "marker missing kind"
grep -q '"bundleDigest": "[0-9a-f]\{64\}"' "$MNG_MARKER" || fail "marker missing bundle digest"
grep -q '"SKILL.md": "[0-9a-f]\{64\}"' "$MNG_MARKER" || fail "marker missing SKILL.md hash"
# Fresh: update leaves marker and files untouched byte-for-byte
cp "$MNG_MARKER" "$MNG_DIR/.marker-before"
APS_LOCAL="$PROJECT_ROOT" $APS update "$MNG_DIR" > /dev/null 2>&1 || fail "update on fresh tree failed"
cmp -s "$MNG_MARKER" "$MNG_DIR/.marker-before" || fail "fresh update rewrote the marker"
# Dirty: user edit is refused and preserved
echo "user edit" >> "$MNG_SKILL/SKILL.md"
output=$(APS_LOCAL="$PROJECT_ROOT" $APS update "$MNG_DIR" 2>&1) || fail "update on dirty tree errored"
echo "$output" | grep -q "local edits" || fail "dirty tree not reported"
grep -q "user edit" "$MNG_SKILL/SKILL.md" || fail "dirty tree was overwritten"
# Stale: valid old marker refreshes without touching matching content
sed -i '$ d' "$MNG_SKILL/SKILL.md"
sed -i 's/"cliVersion": "[^"]*"/"cliVersion": "0.0.1"/' "$MNG_MARKER"
APS_LOCAL="$PROJECT_ROOT" $APS update "$MNG_DIR" > /dev/null 2>&1 || fail "update on stale tree failed"
grep -q '"cliVersion": "0.0.1"' "$MNG_MARKER" && fail "stale marker not refreshed"
cmp -s "$MNG_MARKER" "$MNG_DIR/.marker-before" || fail "stale refresh is not canonical"
# Adopt: matching files without a marker gain one, files untouched
rm "$MNG_MARKER"
APS_LOCAL="$PROJECT_ROOT" $APS update "$MNG_DIR" > /dev/null 2>&1 || fail "update on adoptable tree failed"
[[ -f "$MNG_MARKER" ]] || fail "matching unmanaged tree not adopted"
# Unmanaged differing: refused, no marker written
rm "$MNG_MARKER"
echo "custom skill" > "$MNG_SKILL/SKILL.md"
output=$(APS_LOCAL="$PROJECT_ROOT" $APS update "$MNG_DIR" 2>&1) || fail "update on unmanaged tree errored"
echo "$output" | grep -q "not installed by APS" || fail "unmanaged tree not reported"
[[ ! -f "$MNG_MARKER" ]] || fail "unmanaged differing tree gained a marker"
grep -q "custom skill" "$MNG_SKILL/SKILL.md" || fail "unmanaged tree was overwritten"
# Broken: unreadable marker refused
echo "{not json" > "$MNG_MARKER"
output=$(APS_LOCAL="$PROJECT_ROOT" $APS update "$MNG_DIR" 2>&1) || fail "update on broken marker errored"
echo "$output" | grep -q "unreadable" || fail "broken marker not reported"
grep -q "custom skill" "$MNG_SKILL/SKILL.md" || fail "broken-marker tree was overwritten"
rm -rf "$MNG_DIR"
pass

# Test 58: INSTALL-021 / D-043 — the curl updater brings a v1-layout project
# to the current v2 layout (config, managed skill trees with markers, no
# legacy paths) and refreshes a v2 project in place on a second run. The
# repo's bash CLI is put on PATH so the updater's delegation path is the one
# under test.
echo -n "Test: curl updater delivers the v2 layout (v1 migrate + v2 refresh)... "
UPD_DIR=$(mktemp -d)
mkdir -p "$UPD_DIR/plans/modules" "$UPD_DIR/aps-planning/scripts" "$UPD_DIR/.claude/commands"
cp "$PROJECT_ROOT/scaffold/aps-planning/SKILL.md" \
   "$PROJECT_ROOT/scaffold/aps-planning/reference.md" \
   "$PROJECT_ROOT/scaffold/aps-planning/examples.md" \
   "$PROJECT_ROOT/scaffold/aps-planning/hooks.md" \
   "$UPD_DIR/aps-planning/"
echo "legacy plan command" > "$UPD_DIR/.claude/commands/plan.md"
echo "old v1 rules" > "$UPD_DIR/plans/aps-rules.md"
printf '# My Plan\n\nUPD-CUSTOM-INDEX-SENTINEL\n' > "$UPD_DIR/plans/index.aps.md"
(cd "$UPD_DIR" && APS_LOCAL="$PROJECT_ROOT" PATH="$PROJECT_ROOT/bin:$PATH" \
  bash "$PROJECT_ROOT/scaffold/update" > /dev/null 2>&1) || fail "updater failed on v1 fixture"
[[ -f "$UPD_DIR/.aps/config.yml" ]] || fail ".aps/config.yml not created"
grep -q 'name: claude-code' "$UPD_DIR/.aps/config.yml" || fail "migrated config missing claude-code tool"
[[ ! -d "$UPD_DIR/aps-planning" ]] || fail "root aps-planning/ still present"
[[ ! -e "$UPD_DIR/.claude/commands" ]] || fail ".claude/commands still present"
[[ -f "$UPD_DIR/.claude/skills/aps-planning/SKILL.md" ]] || fail "managed skill tree not installed"
[[ -f "$UPD_DIR/.claude/skills/aps-planning/.aps-managed.json" ]] || fail "managed marker not written"
grep -q "UPD-CUSTOM-INDEX-SENTINEL" "$UPD_DIR/plans/index.aps.md" || fail "user index.aps.md was modified"
grep -q "APS Rules" "$UPD_DIR/plans/aps-rules.md" || fail "aps-rules.md not refreshed to v2"
ls "$UPD_DIR/.aps/backup" | grep -q . || fail "no migration backup written"
# Second run: project is now v2 — refresh in place, no duplication, no
# legacy resurrection, marker stays canonical byte-for-byte.
cp "$UPD_DIR/.claude/skills/aps-planning/.aps-managed.json" "$UPD_DIR/.marker-before"
(cd "$UPD_DIR" && APS_LOCAL="$PROJECT_ROOT" PATH="$PROJECT_ROOT/bin:$PATH" \
  bash "$PROJECT_ROOT/scaffold/update" > /dev/null 2>&1) || fail "updater failed on v2 refresh"
cmp -s "$UPD_DIR/.claude/skills/aps-planning/.aps-managed.json" "$UPD_DIR/.marker-before" \
  || fail "v2 refresh rewrote the marker"
[[ ! -d "$UPD_DIR/aps-planning" ]] || fail "v2 refresh recreated root aps-planning/"
[[ ! -e "$UPD_DIR/.claude/commands" ]] || fail "v2 refresh recreated .claude/commands"
[[ "$(grep -c 'name: claude-code' "$UPD_DIR/.aps/config.yml")" == "1" ]] \
  || fail "v2 refresh duplicated config tools"
rm -rf "$UPD_DIR"
pass

# Test 59: INSTALL-021 / D-043 — static guard: the curl updaters never fetch
# the legacy v1 payload (unprefixed aps-planning/* or commands/*) and never
# create .claude/commands. They delegate the refresh to `aps update`, whose
# skill payload both CLI libraries pin to the packaged scaffold/aps-planning/*.
echo -n "Test: curl updater static guard (scaffold payload, no legacy paths)... "
UPD_SH="$PROJECT_ROOT/scaffold/update"
UPD_PS="$PROJECT_ROOT/scaffold/update.ps1"
! grep -Eq 'download(_root)? "(aps-planning|commands)/' "$UPD_SH" \
  || fail "bash updater fetches a legacy payload path"
! grep -Eq '\-(Path|Source) "(aps-planning|commands)/' "$UPD_PS" \
  || fail "PowerShell updater fetches a legacy payload path"
! grep -Eq 'mkdir[^#]*\.claude/commands' "$UPD_SH" \
  || fail "bash updater creates .claude/commands"
! grep -Eq 'New-Item[^#]*\.claude[^#]*commands|New-Item[^#]*commands[^#]*\.claude' "$UPD_PS" \
  || fail "PowerShell updater creates .claude/commands"
grep -Fq '"$APS_BIN" update' "$UPD_SH" \
  || fail "bash updater does not delegate to aps update"
grep -Fq 'DelegateArgs @("update", $Target)' "$UPD_PS" \
  || fail "PowerShell updater does not delegate to aps update"
grep -Fq '"scaffold/aps-planning/SKILL.md"' "$PROJECT_ROOT/lib/scaffold.sh" \
  || fail "bash CLI skill payload is not the packaged scaffold copy"
grep -Fq '"scaffold/aps-planning/SKILL.md"' "$PROJECT_ROOT/lib/Scaffold.psm1" \
  || fail "PowerShell CLI skill payload is not the packaged scaffold copy"
pass

# Test 60: INSTALL-022 / D-044 — plans/.aps-version is retired. Init never
# writes it, update removes a legacy one, and the skill's staleness check is
# bound to .aps/config.yml instead of a hardcoded version constant.
echo -n "Test: .aps-version retired (init clean, update removes legacy)... "
AV_DIR=$(mktemp -d)
APS_LOCAL="$PROJECT_ROOT" $APS init "$AV_DIR" --profile solo --scope small --tools generic > /dev/null 2>&1 || fail "init failed"
[[ ! -f "$AV_DIR/plans/.aps-version" ]] || fail "init wrote plans/.aps-version"
echo "0.5.0" > "$AV_DIR/plans/.aps-version"
APS_LOCAL="$PROJECT_ROOT" $APS update "$AV_DIR" > /dev/null 2>&1 || fail "update failed"
[[ ! -f "$AV_DIR/plans/.aps-version" ]] || fail "update did not remove legacy .aps-version"
# Static guards: no bash write path remains; both SKILL.md copies bind the
# staleness check to the config contract, not a version stamp.
grep -E '(echo|printf)[^#]*\.aps-version' "$PROJECT_ROOT/lib/scaffold.sh" > /dev/null && fail "lib/scaffold.sh still writes .aps-version"
grep -q 'aps-version' "$PROJECT_ROOT/scaffold/aps-planning/SKILL.md" && fail "scaffold SKILL.md still references .aps-version"
grep -q 'aps-version' "$PROJECT_ROOT/aps-planning/SKILL.md" && fail "root SKILL.md still references .aps-version"
grep -q '\.aps/config\.yml' "$PROJECT_ROOT/scaffold/aps-planning/SKILL.md" || fail "scaffold SKILL.md staleness check not bound to config.yml"
rm -rf "$AV_DIR"
pass

echo ""
echo -e "${GREEN}All tests passed!${NC}"
