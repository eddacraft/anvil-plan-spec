---
name: aps-probe
description: >-
  PROBE+ — the self-validating capability manifest for the APS loop. Detect a
  repo's real test/lint/typecheck/build commands, isolation mechanism, CI,
  branch protections, secret tooling, and package manager once per repo, and
  cache them with evidence (config-file hash, discovered test count, output
  sample) so the rest of the loop binds to commands that provably exist and false
  greens are impossible. Use before executing an APS work item, when "detect the
  repo's capabilities", "probe the project", "build the capability manifest",
  "what commands does this repo use", or when a cached manifest may be stale.
---

# APS Probe (PROBE+)

The pre-loop phase of the canonical APS loop. PROBE+ **detects** what a repository
can do — it never assumes — and records the answer as a cached, self-validating
**capability manifest**. Every later phase (ISOLATE, BUILD, VERIFY, LAND) binds
to the manifest's commands rather than guessing, so the loop is portable across
languages and harnesses and cannot silently run the wrong (or a fake) check.

This is the **portability keystone** of the loop and, per the design's stress
test, its **highest-risk component**: if the manifest is wrong or stale, every
gate downstream inherits the error. The whole design of this phase is therefore
about making the manifest carry **evidence of its own correctness**.

Run PROBE+ **once per repo** and cache the result. Re-run only when an evidence
check below says to.

## What to detect

Detect each of the following; record absence as an explicit "none found", never as
an assumption:

- **Commands:** test, lint, typecheck, build — the actual invocation for this
  repo, derived from real config (manifest scripts, task runner, build file).
- **CI presence:** is there a CI definition, and which checks does it run.
- **Isolation mechanism:** worktree, devcontainer, or docker — how work can be
  done reversibly off the default branch.
- **Branch protections:** which branches are protected and what they require.
- **Secret-handling tooling:** how secrets are kept out of the repo (vault,
  env-file conventions, secret scanners).
- **Package manager:** the one actually in use (lockfile / config evidence).

Detection is **cross-language**: identify the toolchain from the files present,
not from a fixed assumption about ecosystem. If a capability genuinely does not
exist, the manifest records that fact so downstream phases adapt rather than
invent a command.

## The manifest carries its own evidence

For **each detected command** the manifest records, at minimum:

- **the command** — the exact invocation;
- **a config hash** — a hash of the config file the command was derived from;
- **the discovered test count** — how many tests the suite actually contains;
- **an output sample** — a short excerpt of the command's real output.

The count and sample together are the proof that the command does something. The
hash is the cache key for that command. A manifest entry without all four is
incomplete and must not be trusted by a later phase.

## Cache invalidation by config change

The manifest is **cached per repo**. Before _any_ use of the manifest, re-check
the recorded config-file hashes against the files on disk:

- **All hashes match** → the manifest is fresh; use it.
- **Any hash mismatches** → a config file changed; **re-probe** the affected
  capability (re-derive the command, re-run it, refresh count + sample + hash).

This makes config change the single, reliable trigger for re-probing. Nothing
else silently invalidates a cached command.

## Post-ISOLATE smoke test

After the loop isolates work (worktree/branch/container) and before BUILD
proceeds, run **one** detected command and require it to **exit 0**.

- **Exit 0** → the isolation is sound; BUILD may proceed.
- **Non-zero** → **invalidate the isolation section** of the manifest and force a
  **re-probe of isolation**. BUILD does not proceed on an unproven environment.

The smoke test catches an isolation mechanism that looked right at detection time
but does not actually produce a working environment.

## VERIFY guardrails

The manifest feeds two non-negotiable guards on the path to LAND:

- **Zero-tests is suspect.** A manifest that discovers **zero** tests must
  **escalate before any LAND** — an empty suite is far more often a detection or
  configuration failure than a genuine state, and must be confirmed by a human,
  not waved through.
- **Incomplete dossier blocks LAND.** If any required command is **missing,
  timed-out, or skipped**, set **`VERIFY_INCOMPLETE`**. This flag **blocks LAND
  even when the gates are set to auto** — an incomplete verification can never be
  treated as a pass.

## The failure mode this prevents

A **placeholder test script** — one that exits 0 trivially (an empty suite, a
stubbed `test` target, a `true` command) — looks green and masks a real suite
that is not running. PROBE+ defeats this because the manifest records the
**discovered test count** and an **output sample** alongside the command and its
config **hash**. A trivial green has no tests to count and no real output to
sample, so the evidence contradicts the exit code and the deception is caught
before it can produce a false green at VERIFY or LAND.

## Operating stance

- Detect, record, prove — never assume. Absence is a recorded fact, not a guess.
- Treat the manifest as cache: validate hashes before every use; re-probe on
  mismatch; refresh count + sample + hash whenever you re-derive a command.
- Surface suspicious results (zero tests, incomplete dossier, failed smoke test)
  rather than smoothing them over. These are escalation and re-probe triggers,
  not edge cases to suppress.
- Keep the manifest the single source of truth for "what this repo can do"; the
  rest of the loop reads it rather than re-deriving commands ad hoc.

## Cross-references

- `plans/designs/2026-06-18-canonical-aps-loop.design.md` — full design and
  rationale (PROBE+ is change #1 / gap G1; flagged the highest-risk, keystone
  component in the stress test).
- `aps-loop` — the canonical loop this phase precedes; it binds ISOLATE, BUILD,
  VERIFY, and LAND to this manifest.
