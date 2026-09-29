#!/usr/bin/env bash
#
# Cross-CLI lint parity harness (CIP-002).
#
# The `aps` linter ships three implementations that must stay in lockstep
# (D-038/D-039): the canonical Rust binary, the maintained bash CLI, and the
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
# Prefer a binary whose --version matches cli/Cargo.toml so a stale
# cli/target/release/aps (CIB-007 / D-036) cannot win over a current debug build.
crate_ver=$(awk -F'"' '/^version = / {print $2; exit}' "$ROOT/cli/Cargo.toml")
RUST_APS="${APS_RUST_BIN:-}"
if [[ -z "$RUST_APS" ]]; then
  for cand in "$ROOT/cli/target/release/aps" "$ROOT/cli/target/debug/aps"; do
    [[ -x "$cand" ]] || continue
    if [[ -n "$crate_ver" && "$("$cand" --version 2>/dev/null | tr -d '\r')" == "aps $crate_ver" ]]; then
      RUST_APS="$cand"
      break
    fi
  done
fi
if [[ -z "$RUST_APS" ]]; then
  echo "No crate-matching Rust binary found — building (cli/)..."
  (cd "$ROOT/cli" && cargo build --quiet) || { echo "cargo build failed"; exit 1; }
  RUST_APS="$ROOT/cli/target/debug/aps"
fi
rust_ver=$("$RUST_APS" --version 2>/dev/null | tr -d '\r')
if [[ -n "$crate_ver" && "$rust_ver" != "aps $crate_ver" ]]; then
  echo "Rust binary at $RUST_APS reports ${rust_ver:-unknown}; rebuilding debug to match crate $crate_ver..."
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
#
# release/plans        one well-formed `releases/v<version>.md`, plus the
#                      README.md guide and a dotfile template that every CLI
#                      must exclude from discovery. Lints clean everywhere.
# release-invalid/plans  one release narrative per R00x failure mode plus a
#                      combined case, so R001 (naming), R002 (Target+Status
#                      header table), R003 (## Release Theme) and R004
#                      (## What Ships) each fire in isolation and together.
#                      This is the coverage whose absence let R001-R004 ship
#                      Rust-only (REL-003 / D-039).
FIXTURES=(
  "valid"
  "invalid"
  "conductor/plans"
  "conductor-clean/plans"
  "conductor-order/plans"
  "crossdep"
  "monorepo/plans"
  "pkgtags/plans"
  "pkgtags-clean/plans"
  "pkgtags-nomarker/plans"
  "release/plans"
  "release-invalid/plans"
  "release-hardening/plans"
  "lint-order/plans"
  "routing/plans"
)

# Compare full text, including file groups, valid files, and counts. Comparing
# finding lines alone misses discovery differences when extra files lint clean.
# PowerShell may use CRLF on Windows; line endings are presentation-only.
findings() { tr -d '\r'; }

# Run one CLI lint invocation, validate its exit status, and store its output
# in the named variable. `aps lint` exits 0 (clean/warnings) or 1 (errors); any
# other code means the CLI failed to run (bad path, missing binary, panic) —
# which could otherwise spuriously match another CLI. On an abnormal exit
# this prints the raw output
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
  printf -v "$__outvar" '%s\nexit: %s' "$(printf '%s\n' "$__out" | findings)" "$__rc"
  return 0
}

echo "Cross-CLI lint parity: bash vs Rust$([[ $HAVE_PWSH == true ]] && echo ' vs PowerShell')"
echo "  rust: $RUST_APS"
[[ $HAVE_PWSH == true ]] && echo "  pwsh: $PWSH"
echo ""

# CIB-008: exercise collision grouping when child paths sort before the parent,
# plus links to a directory, a file, and a cycle. Build links at runtime so Git
# symlink settings cannot turn the regression into ordinary fixture files.
LINT_TMP="$(mktemp -d)"
trap 'rm -rf "$LINT_TMP" "${MARKER_TMP:-}"' EXIT
cp -R "$SCRIPT_DIR/fixtures/monorepo/." "$LINT_TMP/"
cp "$LINT_TMP/packages/core/plans/modules/auth.aps.md" "$LINT_TMP/packages/api/plans/modules/auth.aps.md"
ln -s "$SCRIPT_DIR/fixtures/invalid" "$LINT_TMP/linked-plans" || exit 1
ln -s . "$LINT_TMP/cycle" || exit 1
ln -s "$LINT_TMP/packages/core/plans/modules/auth.aps.md" "$LINT_TMP/linked.aps.md" || exit 1
FIXTURES+=("$LINT_TMP")

for fx in "${FIXTURES[@]}"; do
  target="$SCRIPT_DIR/fixtures/$fx"
  [[ "$fx" == "$LINT_TMP" ]] && target="$LINT_TMP"
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
    n=$(printf '%s\n' "$b" | grep -cE '(E|W|R)[0-9]{3}:' )
    echo -e "${GREEN}OK${NC} $fx ($n findings)"
  fi
done

echo ""

# Export parity (INTEGRATIONS-002): `aps export` must be byte-identical
# between bash and Rust — exit code included. (No PowerShell surface,
# matching the next/rollup precedent.)
EXPORT_FIXTURES=("valid" "monorepo/plans" "pkgnext/plans" "orchestrate/plans")
for fx in "${EXPORT_FIXTURES[@]}"; do
  target="$SCRIPT_DIR/fixtures/$fx"
  b=$("$BASH_APS" export --plans "$target" 2>&1); brc=$?
  r=$("$RUST_APS" export --plans "$target" 2>&1); rrc=$?
  if [[ "$b" == "$r" && $brc -eq $rrc ]]; then
    echo -e "${GREEN}OK${NC} export $fx (rc=$brc)"
  else
    echo -e "${RED}DIVERGE${NC} export bash vs Rust on $fx (rc $brc vs $rrc):"
    diff <(printf '%s\n' "$b") <(printf '%s\n' "$r") | sed 's/^/    /' | head -10
    fail=1
  fi
done

echo ""

# Managed skill marker parity (INSTALL-020 / D-042): the `.aps-managed.json`
# sidecar must be byte-identical no matter which CLI wrote it — same JSON
# shape, same per-file hashes, same bundle digest, same cliVersion. Rust
# writes from its embeds via `aps setup`; bash installs via `aps init
# --tools`; PowerShell serialises the payload directly. A version bump or
# payload change that reaches only one CLI diverges here.
MARKER_TMP="$(mktemp -d)"
trap 'rm -rf "$LINT_TMP" "$MARKER_TMP"' EXIT

# The leg below cd's away from the repo root, so a relative APS_RUST_BIN
# must be pinned to an absolute path first.
RUST_APS_ABS="$(cd "$(dirname "$RUST_APS")" && pwd)/$(basename "$RUST_APS")"

mkdir -p "$MARKER_TMP/rust"
(cd "$MARKER_TMP/rust" && "$RUST_APS_ABS" init --non-interactive >/dev/null 2>&1 && "$RUST_APS_ABS" setup claude-code >/dev/null 2>&1)
RUST_MARKER="$MARKER_TMP/rust/.claude/skills/aps-planning/.aps-managed.json"

mkdir -p "$MARKER_TMP/bash"
APS_LOCAL="$ROOT" "$BASH_APS" init "$MARKER_TMP/bash" --profile solo --scope small --tools claude-code >/dev/null 2>&1
BASH_MARKER="$MARKER_TMP/bash/.claude/skills/aps-planning/.aps-managed.json"

marker_fail=0
if [[ ! -f "$RUST_MARKER" ]]; then
  echo -e "${RED}MISSING${NC} Rust marker (setup claude-code wrote nothing)"; marker_fail=1
fi
if [[ ! -f "$BASH_MARKER" ]]; then
  echo -e "${RED}MISSING${NC} bash marker (init --tools claude-code wrote nothing)"; marker_fail=1
fi
if (( marker_fail == 0 )) && ! cmp -s "$RUST_MARKER" "$BASH_MARKER"; then
  echo -e "${RED}DIVERGE${NC} managed marker bash vs Rust:"
  diff "$RUST_MARKER" "$BASH_MARKER" | sed 's/^/    /'
  marker_fail=1
fi

if (( marker_fail == 0 )) && $HAVE_PWSH; then
  APS_LOCAL="$ROOT" "$PWSH" -NoProfile -Command "
    Import-Module '$ROOT/lib/Output.psm1' -Force
    Import-Module '$ROOT/lib/Scaffold.psm1' -Force
    \$json = Get-ApsManagedManifestJson -PayloadDir (Get-ApsSkillPayload)
    [System.IO.File]::WriteAllText('$MARKER_TMP/marker-pwsh.json', \$json, [System.Text.UTF8Encoding]::new(\$false))
  "
  if ! cmp -s "$RUST_MARKER" "$MARKER_TMP/marker-pwsh.json"; then
    echo -e "${RED}DIVERGE${NC} managed marker Rust vs PowerShell:"
    diff "$RUST_MARKER" "$MARKER_TMP/marker-pwsh.json" | sed 's/^/    /'
    marker_fail=1
  fi
fi

if (( marker_fail == 0 )); then
  echo -e "${GREEN}OK${NC} managed marker byte parity (Rust = bash$($HAVE_PWSH && echo ' = PowerShell'))"
else
  fail=1
fi

echo ""

# CIB-007: top-level `aps --version` is identical across the three CLIs.
# Format is clap's `aps <semver>`. Unset APS_CLI_VERSION so the fallbacks use
# their baked default, which must match the Rust crate version (D-036).
version_fail=0
bver=$(env -u APS_CLI_VERSION "$BASH_APS" --version 2>&1); bvrc=$?
rver=$("$RUST_APS" --version 2>&1); rvrc=$?
bver=$(printf '%s' "$bver" | tr -d '\r')
rver=$(printf '%s' "$rver" | tr -d '\r')
if [[ "$bver" == "$rver" && $bvrc -eq 0 && $rvrc -eq 0 ]]; then
  echo -e "${GREEN}OK${NC} --version bash = Rust ($rver)"
else
  echo -e "${RED}DIVERGE${NC} --version bash vs Rust (rc $bvrc vs $rvrc):"
  diff <(printf '%s\n' "$bver") <(printf '%s\n' "$rver") | sed 's/^/    /'
  version_fail=1
fi

bV=$(env -u APS_CLI_VERSION "$BASH_APS" -V 2>&1); bVrc=$?
rV=$("$RUST_APS" -V 2>&1); rVrc=$?
bV=$(printf '%s' "$bV" | tr -d '\r')
rV=$(printf '%s' "$rV" | tr -d '\r')
if [[ "$bV" == "$rV" && "$bV" == "$rver" && $bVrc -eq 0 && $rVrc -eq 0 ]]; then
  echo -e "${GREEN}OK${NC} -V bash = Rust"
else
  echo -e "${RED}DIVERGE${NC} -V bash vs Rust (rc $bVrc vs $rVrc):"
  diff <(printf '%s\n' "$bV") <(printf '%s\n' "$rV") | sed 's/^/    /'
  version_fail=1
fi

if $HAVE_PWSH; then
  pver=$(env -u APS_CLI_VERSION "$PWSH" -NoProfile -Command "& '$PS_APS' --version" 2>&1); pvrc=$?
  pver=$(printf '%s' "$pver" | tr -d '\r')
  if [[ "$pver" == "$rver" && $pvrc -eq 0 ]]; then
    echo -e "${GREEN}OK${NC} --version PowerShell = Rust"
  else
    echo -e "${RED}DIVERGE${NC} --version PowerShell vs Rust (rc $pvrc vs $rvrc):"
    diff <(printf '%s\n' "$pver") <(printf '%s\n' "$rver") | sed 's/^/    /'
    version_fail=1
  fi
  # `pwsh -File` is how CI and non-interactive callers launch the fallback.
  pfile=$(env -u APS_CLI_VERSION "$PWSH" -NoProfile -File "$PS_APS" --version 2>&1); pfilerc=$?
  pfile=$(printf '%s' "$pfile" | tr -d '\r')
  if [[ "$pfile" == "$rver" && $pfilerc -eq 0 ]]; then
    echo -e "${GREEN}OK${NC} --version PowerShell -File = Rust"
  else
    echo -e "${RED}DIVERGE${NC} --version PowerShell -File vs Rust (rc $pfilerc vs $rvrc):"
    diff <(printf '%s\n' "$pfile") <(printf '%s\n' "$rver") | sed 's/^/    /'
    version_fail=1
  fi
  pV=$(env -u APS_CLI_VERSION "$PWSH" -NoProfile -Command "& '$PS_APS' -V" 2>&1); pVrc=$?
  pV=$(printf '%s' "$pV" | tr -d '\r')
  if [[ "$pV" == "$rver" && $pVrc -eq 0 ]]; then
    echo -e "${GREEN}OK${NC} -V PowerShell = Rust"
  else
    echo -e "${RED}DIVERGE${NC} -V PowerShell vs Rust (rc $pVrc vs $rvrc):"
    diff <(printf '%s\n' "$pV") <(printf '%s\n' "$rver") | sed 's/^/    /'
    version_fail=1
  fi
fi

if [[ ! "$rver" =~ ^aps\ [0-9]+\.[0-9]+\.[0-9]+ ]]; then
  echo -e "${RED}ERROR${NC} Rust --version is not 'aps <semver>': $rver"
  version_fail=1
fi
(( version_fail != 0 )) && fail=1

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
