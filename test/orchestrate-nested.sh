#!/usr/bin/env bash
#
# MONO-003: orchestration across federated nested plans.
# Exercises next/start/complete/graph/audit traversal + child scoping +
# cross-tree (<name>:<ID>) reference resolution against the monorepo fixture.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
APS="$ROOT/bin/aps"
FIXTURE="$ROOT/test/fixtures/monorepo"
PLANS="$FIXTURE/plans"

export APS_STALE_DAYS=100000   # keep the fixture's dated modules from going stale

assert_contains() {
  local output="$1" expected="$2"
  if [[ "$output" != *"$expected"* ]]; then
    printf 'Expected output to contain: %s\nActual output:\n%s\n' "$expected" "$output" >&2
    exit 1
  fi
}

assert_not_contains() {
  local output="$1" unexpected="$2"
  if [[ "$output" == *"$unexpected"* ]]; then
    printf 'Expected output NOT to contain: %s\nActual output:\n%s\n' "$unexpected" "$output" >&2
    exit 1
  fi
}

# --- next: federated traversal + child scope ---

# From the federation root, next spans both child trees. The only unblocked
# Ready item is core:AUTH-001 (HND-001 is blocked by cross-tree dep core:AUTH-001).
output=$("$APS" next --plans "$PLANS")
assert_contains "$output" "AUTH-001: Implement token issuance"
assert_contains "$output" "packages/core/plans/modules/auth.aps.md"

# --child scopes the queue to one child plan.
output=$("$APS" next --child core --plans "$PLANS")
assert_contains "$output" "AUTH-001: Implement token issuance"

# api's only item is blocked by the cross-tree dependency, so scoped next fails.
if output=$("$APS" next --child api --plans "$PLANS" 2>&1); then
  printf 'Expected next --child api to fail (HND-001 blocked)\n' >&2
  exit 1
fi
assert_contains "$output" "No ready work item found in child: api"

# --- mutation across trees writes only the owning child file ---

WORK_DIR=$(mktemp -d)
trap 'rm -rf "$WORK_DIR"' EXIT
cp -r "$FIXTURE/." "$WORK_DIR/"
WORK_PLANS="$WORK_DIR/plans"
CORE_MOD="$WORK_DIR/packages/core/plans/modules/auth.aps.md"
API_MOD="$WORK_DIR/packages/api/plans/modules/handlers.aps.md"
API_BEFORE=$(cat "$API_MOD")

output=$("$APS" start AUTH-001 --plans "$WORK_PLANS")
assert_contains "$output" "Marked AUTH-001 as In Progress"
assert_contains "$output" "packages/core/plans/modules/auth.aps.md"
grep -qF -- "- **Status:** In Progress" "$CORE_MOD" || {
  printf 'Expected core AUTH-001 marked In Progress\n' >&2; exit 1; }
# The sibling tree must be untouched by a start in another tree.
if [[ "$(cat "$API_MOD")" != "$API_BEFORE" ]]; then
  printf 'Expected api handlers.aps.md to be unchanged by core mutation\n' >&2
  exit 1
fi

# Completing the cross-tree dependency unblocks the dependent item.
"$APS" complete AUTH-001 --plans "$WORK_PLANS" --learning "Tokens issued" > /dev/null
grep -qF -- "- **Status:** Complete:" "$CORE_MOD" || {
  printf 'Expected core AUTH-001 Complete\n' >&2; exit 1; }

# Federated next now surfaces the api item; scoping proves isolation.
output=$("$APS" next --plans "$WORK_PLANS")
assert_contains "$output" "HND-001: Protect routes with core auth"

output=$("$APS" next --child api --plans "$WORK_PLANS")
assert_contains "$output" "HND-001"

if output=$("$APS" next --child core --plans "$WORK_PLANS" 2>&1); then
  printf 'Expected next --child core to fail after all core items complete\n' >&2
  exit 1
fi
assert_contains "$output" "No ready work item found in child: core"

# --- ambiguous bare IDs require disambiguation (D-002 / W020 semantics) ---

COL_DIR=$(mktemp -d)
cp -r "$FIXTURE/." "$COL_DIR/"
COL_PLANS="$COL_DIR/plans"
COL_API="$COL_DIR/packages/api/plans/modules/handlers.aps.md"
# Rename api's HND-001 to AUTH-001 so the ID now collides across trees.
awk '{ gsub(/HND-001/, "AUTH-001"); print }' "$COL_API" > "$COL_API.tmp" && mv "$COL_API.tmp" "$COL_API"

if output=$("$APS" start AUTH-001 --plans "$COL_PLANS" 2>&1); then
  printf 'Expected ambiguous start to fail\n' >&2
  exit 1
fi
assert_contains "$output" "Ambiguous work item 'AUTH-001'"

# A path-qualified ref resolves the collision and writes the right tree.
output=$("$APS" start core:AUTH-001 --plans "$COL_PLANS")
assert_contains "$output" "Marked AUTH-001 as In Progress"
assert_contains "$output" "packages/core/plans/modules/auth.aps.md"
"$APS" complete core:AUTH-001 --plans "$COL_PLANS" > /dev/null

# --child achieves the same disambiguation, targeting the api tree. (The api
# item's cross-tree dep core:AUTH-001 is now Complete, so it can start.)
output=$("$APS" start AUTH-001 --child api --plans "$COL_PLANS")
assert_contains "$output" "Marked AUTH-001 as In Progress"
assert_contains "$output" "packages/api/plans/modules/handlers.aps.md"
rm -rf "$COL_DIR"

# --- graph: cross-tree edges + child scope ---

output=$("$APS" graph --plans "$PLANS")
assert_contains "$output" "AUTH-001 [Ready] Implement token issuance"
assert_contains "$output" "HND-001 [Ready] Protect routes with core auth"
assert_contains "$output" "core:AUTH-001[Ready]"   # cross-tree edge keeps its prefix

output=$("$APS" graph --child api --plans "$PLANS")
assert_contains "$output" "HND-001"
assert_not_contains "$output" "AUTH-001 [Ready] Implement token issuance"

# --- audit: federated traversal + child scope ---

output=$("$APS" audit --no-run --plans "$PLANS")
assert_contains "$output" "2 items audited"   # both child trees are traversed

output=$("$APS" audit --no-run --child core --plans "$PLANS")
assert_contains "$output" "1 items audited"

# --- rollup (MONO-004): per-child aggregate matches the fixture root table ---

# Collapse runs of spaces so compact `aps rollup` output and the padded,
# hand-authored fixture table compare on content, not column alignment.
squeeze() { tr -s ' '; }

output=$("$APS" rollup --plans "$PLANS")
rollup_norm=$(printf '%s\n' "$output" | squeeze)
assert_contains "$rollup_norm" "| core | 0/1 | AUTH-001 | Ready |"
assert_contains "$rollup_norm" "| api | 0/1 | — | Ready |"

# The committed roll-up in the fixture root must match what rollup computes,
# so the hand-authored table cannot silently drift from child state.
fixture_norm=$(squeeze < "$PLANS/index.aps.md")
for row in \
  "| core | 0/1 | AUTH-001 | Ready |" \
  "| api | 0/1 | — | Ready |"; do
  printf '%s\n' "$fixture_norm" | grep -qF -- "$row" \
    || { printf 'Fixture root Roll-up out of sync with aps rollup:\n%s\n' "$row" >&2; exit 1; }
done

# rollup on a leaf child (no ## Child Plans) has nothing to summarise.
if "$APS" rollup --plans "$FIXTURE/packages/core/plans" > /dev/null 2>&1; then
  printf 'Expected rollup on a leaf child to fail\n' >&2
  exit 1
fi

# --- bare dependency IDs resolve within the depending item's own tree ---
# D-002 lets the same bare ID exist in sibling trees. A bare `Dependencies:`
# entry must resolve to the depending item's OWN tree, not an arbitrary
# declaration-order first match in another tree.
DEP_DIR=$(mktemp -d)
cp -r "$FIXTURE/." "$DEP_DIR/"
# core:AUTH-001 stays Ready. Give api its own AUTH-001 (Complete) plus an item
# that depends on the bare ID AUTH-001.
cat >> "$DEP_DIR/packages/api/plans/modules/handlers.aps.md" <<'DEPEOF'

### AUTH-001: Api-local complete item

- **Intent:** an api-tree item sharing a bare ID with core
- **Expected Outcome:** done
- **Validation:** `true`
- **Status:** Complete

### DEP-001: Depends on the bare in-tree AUTH-001

- **Intent:** verify bare deps resolve in-tree
- **Expected Outcome:** unblocked by api's own AUTH-001
- **Validation:** `true`
- **Dependencies:** AUTH-001
DEPEOF
# api's own AUTH-001 is Complete, so DEP-001 is ready even though core:AUTH-001
# (the declaration-order first match) is still Ready.
output=$("$APS" next --child api --plans "$DEP_DIR/plans")
assert_contains "$output" "DEP-001"
rm -rf "$DEP_DIR"

printf 'nested orchestration tests passed\n'
