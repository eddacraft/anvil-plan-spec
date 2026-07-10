#!/usr/bin/env bash
#
# Cross-CLI lint parity harness (CIP-002).
#
# The `aps` linter ships three implementations that must stay in lockstep
# (D-038/D-039): the canonical Rust binary, the feature-frozen bash CLI, and the
# PowerShell fallback. test/run.sh guards the ports by string-matching (a rule
# *exists*); test/ps-parity.ps1 checks PowerShell *behaviour* on curated
# scenarios. This harness closes the loop structurally: it runs all three CLIs
# over the fixture corpus and asserts they emit byte-identical findings — same
# codes, messages, line numbers, and emission order. A rule ported to only one
# CLI, a divergent message, an off-by-one line, or a reordered check all fail
# here. It is the automated form of the manual three-way diff sweep run in
# COND-007.
#
# Rust binary: taken from APS_RUST_BIN, else a prebuilt cli/target/{release,debug}
#   /aps, else built on the fly. A relative APS_RUST_BIN (as CI passes) is
#   resolved against the repo root — the script cd's there on startup.
# PowerShell: APS_PWSH, else `pwsh` on PATH. If neither is present the PowerShell
#   leg is skipped with a loud warning (CI runners always have pwsh); bash-vs-Rust
#   still runs.

set -u

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# Run from the repo root so a relative APS_RUST_BIN (e.g. CI's
# cli/target/debug/aps) resolves regardless of the invocation directory. Every
# other path below is already absolute ($ROOT/...), so this only affects that.
cd "$ROOT" || { echo "cannot cd to repo root: $ROOT"; exit 1; }

BASH_APS="$ROOT/bin/aps"
PS_APS="$ROOT/bin/aps.ps1"

# Pin hygiene thresholds so caller-environment values can't skew W017 across
# runs (test/run.sh does the same).
export APS_STALE_DAYS=60

# --- Locate the Rust binary (canonical CLI, D-031) ---------------------------
RUST_APS="${APS_RUST_BIN:-}"
if [[ -z "$RUST_APS" ]]; then
  for cand in "$ROOT/cli/target/release/aps" "$ROOT/cli/target/debug/aps"; do
    [[ -x "$cand" ]] && RUST_APS="$cand" && break
  done
fi
if [[ -z "$RUST_APS" ]]; then
  echo "No prebuilt Rust binary found — building (cli/)..."
  (cd "$ROOT/cli" && cargo build --quiet) || { echo "cargo build failed"; exit 1; }
  RUST_APS="$ROOT/cli/target/debug/aps"
fi

# --- Locate pwsh (required in CI, optional locally) --------------------------
PWSH="${APS_PWSH:-}"
[[ -z "$PWSH" ]] && command -v pwsh >/dev/null 2>&1 && PWSH="pwsh"
HAVE_PWSH=true
if [[ -z "$PWSH" ]]; then
  HAVE_PWSH=false
  echo "WARNING: pwsh not found — skipping the PowerShell leg (bash vs Rust only)."
  echo "         Install PowerShell or set APS_PWSH for full three-way parity."
  echo ""
fi

RED='\033[0;31m'; GREEN='\033[0;32m'; YELLOW='\033[0;33m'; NC='\033[0m'
fail=0

# Fixture directories whose lint output must be identical across every CLI.
# (Command-specific fixtures — audit/, orchestrate/, config/ — are excluded;
# this harness covers `aps lint` only.)
FIXTURES=(
  "valid"
  "invalid"
  "conductor/plans"
  "conductor-clean/plans"
  "conductor-order/plans"
  "crossdep"
  "monorepo/plans"
)

# Order-preserving finding lines: `CODE: message (line N)`. Strips the file-path
# group headers and summary lines, leaving the sequence of findings each CLI
# emits — which must match byte-for-byte, order included.
findings() { grep -oE '(E|W)[0-9]{3}:.*' || true; }

# Run one CLI lint invocation, validate its exit status, and store its findings
# in the named variable. `aps lint` exits 0 (clean/warnings) or 1 (errors); any
# other code means the CLI failed to run (bad path, missing binary, panic) —
# which the findings filter would otherwise reduce to an empty string that could
# spuriously "match" another CLI. On an abnormal exit this prints the raw output
# and returns non-zero so the caller fails the fixture rather than comparing junk.
run_lint() {
  local __outvar="$1" __label="$2"; shift 2
  local __out __rc
  __out=$("$@" 2>&1); __rc=$?
  if (( __rc != 0 && __rc != 1 )); then
    echo -e "${RED}ERROR${NC} $__label exited $__rc (not a normal lint result):"
    printf '%s\n' "$__out" | sed 's/^/    /'
    return 1
  fi
  printf -v "$__outvar" '%s' "$(printf '%s\n' "$__out" | findings)"
  return 0
}

echo "Cross-CLI lint parity: bash vs Rust$([[ $HAVE_PWSH == true ]] && echo ' vs PowerShell')"
echo "  rust: $RUST_APS"
[[ $HAVE_PWSH == true ]] && echo "  pwsh: $PWSH"
echo ""

for fx in "${FIXTURES[@]}"; do
  target="$SCRIPT_DIR/fixtures/$fx"
  if [[ ! -e "$target" ]]; then
    echo -e "${RED}MISSING${NC} fixture: $fx"; fail=1; continue
  fi

  ok=true
  b=""; r=""; p=""
  run_lint b "bash" "$BASH_APS" lint "$target" || { ok=false; fail=1; }
  run_lint r "Rust" "$RUST_APS" lint "$target" || { ok=false; fail=1; }

  if $ok && [[ "$b" != "$r" ]]; then
    echo -e "${RED}DIVERGE${NC} bash vs Rust on $fx:"
    diff <(printf '%s\n' "$b") <(printf '%s\n' "$r") | sed 's/^/    /'
    ok=false; fail=1
  fi

  if $ok && $HAVE_PWSH; then
    if run_lint p "PowerShell" "$PWSH" -NoProfile -File "$PS_APS" lint "$target"; then
      if [[ "$b" != "$p" ]]; then
        echo -e "${RED}DIVERGE${NC} bash vs PowerShell on $fx:"
        diff <(printf '%s\n' "$b") <(printf '%s\n' "$p") | sed 's/^/    /'
        ok=false; fail=1
      fi
    else
      ok=false; fail=1
    fi
  fi

  if $ok; then
    n=$(printf '%s\n' "$b" | grep -c . )
    echo -e "${GREEN}OK${NC} $fx ($n findings)"
  fi
done

echo ""
if [[ $fail -ne 0 ]]; then
  echo -e "${RED}Cross-CLI parity FAILED — the linters diverged (see above).${NC}"
  exit 1
fi
if $HAVE_PWSH; then
  echo -e "${GREEN}Cross-CLI parity OK: bash = Rust = PowerShell across ${#FIXTURES[@]} fixtures.${NC}"
else
  echo -e "${YELLOW}Cross-CLI parity OK: bash = Rust across ${#FIXTURES[@]} fixtures (PowerShell skipped).${NC}"
  echo -e "${YELLOW}NOTE: PowerShell leg did not run — full lockstep unverified in this environment.${NC}"
fi
exit 0
