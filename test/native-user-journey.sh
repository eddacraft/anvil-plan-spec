#!/usr/bin/env bash
# Native APS user journey for Unix CI. Mirrors windows-user-journey.ps1.

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
APS_BIN="${APS_BIN:-$ROOT/cli/target/debug/aps}"
if [[ "$APS_BIN" != /* ]]; then
  APS_BIN="$ROOT/$APS_BIN"
fi

if [[ ! -x "$APS_BIN" ]]; then
  echo "native user journey: APS_BIN is not executable: $APS_BIN" >&2
  exit 1
fi

WORK="$(mktemp -d)"
trap 'rm -rf "$WORK"' EXIT

"$APS_BIN" --version >/dev/null

# Single-project root.
SINGLE="$WORK/single"
mkdir -p "$SINGLE"
(
  cd "$SINGLE"
  "$APS_BIN" init --non-interactive --profile solo --tools generic
  grep -qF 'shape: single' .aps/config.yml
  grep -qF '  - index' .aps/config.yml
  ! grep -qF '## Modules by Package' plans/index.aps.md
)

# Monorepo root plus the complete native user command surface.
MONOREPO="$WORK/monorepo"
mkdir -p "$MONOREPO"
(
  cd "$MONOREPO"
  "$APS_BIN" init --non-interactive --profile team --shape monorepo --tools generic
  grep -qF '## Modules by Package' plans/index.aps.md
  grep -qF 'shape: monorepo' .aps/config.yml
  grep -qF '  - monorepo-index' .aps/config.yml
  "$APS_BIN" setup hooks --yes
  test -f .aps/scripts/install-hooks.sh
  test -f .aps/scripts/install-hooks.ps1
  "$APS_BIN" lint plans
  next_output=$("$APS_BIN" next 2>&1 || true)
  grep -qF 'No ready work item found' <<<"$next_output"
  "$APS_BIN" update . >/dev/null
  "$APS_BIN" migrate >/dev/null
  "$APS_BIN" doctor >/dev/null
)

# Federated/nested root and rollup.
NESTED="$WORK/nested"
mkdir -p "$NESTED"
(
  cd "$NESTED"
  "$APS_BIN" init --non-interactive --profile team --templates index-nested --tools generic
  grep -qF '## Child Plans' plans/index.aps.md
  test -f packages/core/plans/index.aps.md
  grep -qF '  - index-nested' .aps/config.yml
  "$APS_BIN" rollup >/dev/null
)

# State-changing/read-only orchestration commands on a disposable fixture.
LIFECYCLE="$WORK/lifecycle"
cp -R "$ROOT/test/fixtures/orchestrate" "$LIFECYCLE"
(
  cd "$LIFECYCLE"
  "$APS_BIN" next auth | grep -qF 'AUTH-003'
  "$APS_BIN" start AUTH-003 >/dev/null
  "$APS_BIN" graph auth >/dev/null
  "$APS_BIN" audit auth --no-run >/dev/null || test $? -eq 1
  "$APS_BIN" export --json | grep -qF '"work_items"'
  "$APS_BIN" complete AUTH-003 --learning "native Unix journey" >/dev/null
  "$APS_BIN" next auth | grep -qF 'AUTH-004'
)

# Real no-argument public installer -> native Ratatui handoff under a PTY.
if command -v script >/dev/null 2>&1; then
  ONBOARD="$WORK/onboard"
  HOME_DIR="$WORK/home"
  MOCK_BIN="$WORK/mock-bin"
  PAYLOAD="$WORK/payload"
  mkdir -p "$ONBOARD" "$HOME_DIR/.aps/bin" "$MOCK_BIN" "$PAYLOAD"
  cp "$APS_BIN" "$PAYLOAD/aps"
  tar -czf "$WORK/aps.tar.gz" -C "$PAYLOAD" aps
  cat > "$MOCK_BIN/curl" <<'EOF'
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
  chmod +x "$MOCK_BIN/curl"
  TRANSCRIPT="$WORK/tui.typescript"
  (
    cd "$ONBOARD"
    { sleep 1; printf 'q'; } | APS_HOME="$HOME_DIR/.aps" \
      APS_TEST_ARCHIVE="$WORK/aps.tar.gz" \
      PATH="$MOCK_BIN:$HOME_DIR/.aps/bin:$PATH" \
      script -q -e -c "stty rows 24 cols 80; bash '$ROOT/scaffold/install'" \
        "$TRANSCRIPT" >/dev/null
  )
  grep -aFq 'Profile' "$TRANSCRIPT"
fi

echo "native Unix user journey passed"
