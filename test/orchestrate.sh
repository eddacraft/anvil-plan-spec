#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
APS="$ROOT/bin/aps"
FIXTURE="$ROOT/test/fixtures/orchestrate"
PLANS="$FIXTURE/plans"

assert_contains() {
  local output="$1"
  local expected="$2"

  if [[ "$output" != *"$expected"* ]]; then
    printf 'Expected output to contain: %s\nActual output:\n%s\n' "$expected" "$output" >&2
    exit 1
  fi
}

assert_file_contains() {
  local file="$1"
  local expected="$2"

  if ! grep -qF -e "$expected" "$file"; then
    printf 'Expected file %s to contain: %s\n' "$file" "$expected" >&2
    printf -- '--- file contents ---\n' >&2
    cat "$file" >&2
    exit 1
  fi
}

# Use a temp copy for any tests that mutate state, so the committed fixture
# stays stable.
WORK_DIR=$(mktemp -d)
trap 'rm -rf "$WORK_DIR"' EXIT
cp -r "$FIXTURE" "$WORK_DIR/orchestrate"
WORK_PLANS="$WORK_DIR/orchestrate/plans"

output=$("$APS" next --plans "$PLANS")
assert_contains "$output" "AUTH-003: Add token refresh"
assert_contains "$output" "Dependencies: AUTH-001"
assert_contains "$output" "AUTH-002"
assert_contains "$output" "CORE-001"

output=$("$APS" next auth --plans "$PLANS")
assert_contains "$output" "AUTH-003: Add token refresh"
assert_contains "$output" "CORE-001"

if output=$("$APS" next billing --plans "$PLANS" 2>&1); then
  printf 'Expected billing module lookup to fail\n' >&2
  exit 1
fi

assert_contains "$output" "No ready work item found for module: billing"

# --- aps start ---

# Ready item with all deps Complete transitions to In Progress
output=$("$APS" start AUTH-003 --plans "$WORK_PLANS")
assert_contains "$output" "Marked AUTH-003 as In Progress"
assert_contains "$output" "Suggested branch: work/auth-003"
assert_contains "$output" "Context package:"
assert_file_contains "$WORK_PLANS/modules/auth.aps.md" "- **Status:** In Progress"
CONTEXT_FILE="$WORK_DIR/orchestrate/.aps/context/AUTH-003.md"
assert_file_contains "$CONTEXT_FILE" "# Context: AUTH-003 - Add token refresh"
assert_file_contains "$CONTEXT_FILE" "## Work Item"
assert_file_contains "$CONTEXT_FILE" "## Module Scope"
assert_file_contains "$CONTEXT_FILE" "## Dependency Learnings"
assert_file_contains "$CONTEXT_FILE" "CORE-001: \"Parser output is stable across modules\""
assert_file_contains "$CONTEXT_FILE" "- src/auth/refresh.sh"

# Regenerating context overwrites stale output.
printf '\nSTALE\n' >> "$CONTEXT_FILE"
output=$("$APS" start AUTH-003 --plans "$WORK_PLANS" 2>&1)
if grep -qF -e "STALE" "$CONTEXT_FILE"; then
  printf 'Expected context regeneration to remove stale content\n' >&2
  exit 1
fi
assert_contains "$output" "already In Progress"

# Already In Progress is a no-op warning
output=$("$APS" start AUTH-003 --plans "$WORK_PLANS" 2>&1)
assert_contains "$output" "already In Progress"

# Item with unmet deps is rejected
if output=$("$APS" start AUTH-004 --plans "$WORK_PLANS" 2>&1); then
  printf 'Expected start AUTH-004 to fail (unmet deps)\n' >&2
  exit 1
fi
assert_contains "$output" "unmet dependencies"

# Unknown ID is rejected
if output=$("$APS" start NOPE-999 --plans "$WORK_PLANS" 2>&1); then
  printf 'Expected start NOPE-999 to fail\n' >&2
  exit 1
fi
assert_contains "$output" "Work item not found"

# Final work item extraction stops before following module sections.
output=$("$APS" start AUTH-006 --plans "$WORK_PLANS")
BILL_CONTEXT="$WORK_DIR/orchestrate/.aps/context/AUTH-006.md"
assert_file_contains "$BILL_CONTEXT" "# Context: AUTH-006 - Final item before decisions"
work_item_section=$(awk '/^## Work Item/{flag=1; next} flag && /^## Module Scope/{exit} flag' \
  "$BILL_CONTEXT")
if [[ "$work_item_section" == *"## Decisions"* || "$work_item_section" == *"## Module Scope"* ]]; then
  printf 'Expected final work item context not to include following sections\n%s\n' "$work_item_section" >&2
  exit 1
fi

# --- aps complete ---

# Cannot complete an item that is still Ready
if output=$("$APS" complete AUTH-004 --plans "$WORK_PLANS" 2>&1); then
  printf 'Expected complete AUTH-004 to fail (not In Progress)\n' >&2
  exit 1
fi
assert_contains "$output" "must be In Progress"

# Completing an In Progress item with a learning stamps the date and inserts
# the learning after Validation (per ORCH D-002).
output=$("$APS" complete AUTH-003 --plans "$WORK_PLANS" --learning "Rotate refresh tokens")
assert_contains "$output" "Marked AUTH-003 as Complete:"
assert_contains "$output" "Learning recorded"
assert_file_contains "$WORK_PLANS/modules/auth.aps.md" "- **Status:** Complete:"
assert_file_contains "$WORK_PLANS/modules/auth.aps.md" '- **Learning:** "Rotate refresh tokens"'

# Verify Learning immediately follows the Validation block within AUTH-003
# (per ORCH D-002). Extract just the AUTH-003 work item block and inspect.
auth_block=$(awk '/^### AUTH-003:/{flag=1; next} flag && /^### /{exit} flag' \
  "$WORK_PLANS/modules/auth.aps.md")
validation_idx=$(printf '%s\n' "$auth_block" | grep -n 'Validation:' | head -1 | cut -d: -f1)
learning_idx=$(printf '%s\n' "$auth_block" | grep -n 'Learning:' | head -1 | cut -d: -f1)
if [[ -z "$validation_idx" || -z "$learning_idx" ]] || \
   ! [[ "$learning_idx" -gt "$validation_idx" ]]; then
  printf 'Expected Learning to follow Validation in AUTH-003 block.\nValidation idx: %s\nLearning idx: %s\nBlock:\n%s\n' \
    "$validation_idx" "$learning_idx" "$auth_block" >&2
  exit 1
fi

# After AUTH-003 completes, aps next should resolve AUTH-004
output=$("$APS" next --plans "$WORK_PLANS")
assert_contains "$output" "AUTH-004"

# --- aps graph ---

output=$("$APS" graph auth --plans "$WORK_PLANS")
assert_contains "$output" "AUTH-003 [Complete] Add token refresh"
assert_contains "$output" "<- AUTH-001[Complete] AUTH-002[Complete] CORE-001[Complete]"
assert_contains "$output" "AUTH-004 [Ready] Add session audit log"
assert_contains "$output" "<- AUTH-003[Complete]"

if output=$("$APS" graph nope --plans "$WORK_PLANS" 2>&1); then
  printf 'Expected unknown graph lookup to fail\n' >&2
  exit 1
fi
assert_contains "$output" "No work items found for module: nope"

# --- status vocabulary aliases (SPEC-001 / D-026 Approach A) ---

# Proposed module status maps to Draft — not actionable even with Ready items
if output=$("$APS" next proposed --plans "$PLANS" 2>&1); then
  printf 'Expected proposed module lookup to fail\n' >&2
  exit 1
fi
assert_contains "$output" "No ready work item found for module: proposed"

# Done dependency maps to Complete — unblocks the next Ready item
output=$("$APS" next zstatus --plans "$PLANS")
assert_contains "$output" "ZSTAT-002: Ready item behind Done dependency"
assert_contains "$output" "Dependencies: ZSTAT-001"

printf 'orchestrate tests passed\n'
