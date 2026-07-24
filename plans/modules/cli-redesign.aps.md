<!-- APS: See docs/ai/prompting/ for AI guidance -->

# CLI Redesign & Harness Expansion Module

| ID  | Owner  | Priority | Status |
| --- | ------ | -------- | ------ |
| CLI | @aneki | high     | Draft  |

**Last reviewed:** 2026-07-23

## Purpose

Take a clean-sheet pass over the `aps` command surface so it reads coherently
for two audiences at once — an individual running solo, and a team where several
humans drive independent agent processes — and ship it as a binary upgrade. In
the same release, finish the harness set: remove the last Gemini residue,
confirm Grok, and expand init/setup beyond the current core (Claude Code,
Copilot, Codex, OpenCode, Grok) to any additional harness that earns its place.

## Background

The command surface grew feature-by-feature — `init`, `setup`, `upgrade`,
`update`, `migrate`, `lint`, `next`, `start`, `complete`, `graph`, `rollup`,
`audit`, `export`, `doctor` — without a deliberate pass over how the whole thing
reads. `.aps/config.yml` already carries `profile: solo`, and the TEAM module
(D-041) is designing an optional coordination plane, but the CLI itself does not
yet flex its vocabulary or surfaced commands by profile: a solo user sees
team-shaped affordances they never use, and a team gets no first-class claim /
handoff / who-has-what verbs.

On harnesses, D-040 already retired Gemini and added Grok at the code level —
`config.rs` rejects `gemini` with a migration hint and accepts `grok`, and the
bash/PowerShell tool lists match. What remains is (a) residual Gemini mentions
in `README.md`, `docs/installation.md`, and other prose, and (b) a genuine
question of how far to widen the harness set. Grok slotted in for zero bespoke
cost because it reads the `AGENTS.md` family and discovers `.agents/skills/`
natively; that native-discovery test is the cheap-add gate for any new harness.

All three CLI implementations (Rust `cli/src/`, bash `bin/aps` + `lib/*.sh`,
PowerShell `bin/aps.ps1` + `lib/*.psm1`) are equal shipping targets under D-039
lockstep — no command-surface change here is Complete until it lands in all
three and the parity suite (`test/fixtures/**`) confirms identical behaviour.

## In Scope

- Clean-sheet audit and redesign of the top-level command surface and help text
- Profile-aware command vocabulary and surfacing (solo vs team) over `profile:`
- Binary upgrade delivery: version surface, `aps upgrade` / self-update ergonomics
- Finishing the Gemini removal (docs/README residue) and confirming Grok end-to-end
- A research spike evaluating additional harnesses against the native-discovery gate
- Adding the approved new harnesses to init/setup/wizard across all three CLIs

## Out of Scope

- Building the TEAM coordination plane itself (claims, leases, adapters) — that
  is [team-coordination](./team-coordination.aps.md); CLI consumes it, it does
  not reimplement it here
- Changing the APS markdown spec, template fields, or status vocabulary
- Adding a hosted service, telemetry, or a required network dependency
- Renaming commands purely for taste without a migration/alias path

## Interfaces

**Depends on:**

- INSTALL — binary distribution channels, `.aps/config.yml` contract, `upgrade`
- ORCH — `next` / `start` / `complete` state flow the surface is built around
- AGENT — per-harness packaging formats and the native-discovery precedent (D-040)
- SCAFFOLD — `init` / `setup` / wizard harness selection and asset install
- TEAM — profile semantics and any actor/claim verbs the team surface exposes

**Exposes:**

- The redesigned `aps` command vocabulary and profile-aware help
- The confirmed harness set for init/setup and the criterion for extending it

## Constraints

- D-039 three-way lockstep: Rust, bash, PowerShell move together, parity-verified
- No breaking changes without an alias/migration path (index Risks; D-037 precedent)
- Markdown-first: the CLI stays optional and never becomes required to use APS

## Ready Checklist

Change status to **Ready** when:

- [x] The command-redesign map (CLI-001) is drafted and reviewed
      ([design](../designs/2026-07-22-cli-redesign.design.md), approved 2026-07-23)
- [ ] Profile semantics (solo vs team) are pinned with the TEAM module
- [ ] Harness-expansion candidates have a go/no-go from the CLI-005 spike

## Work Items

### CLI-001: Command-surface audit and redesign map

- **Intent:** Produce a deliberate map of the whole `aps` command surface — every
  verb, its noun, its help text, and its grouping — that reads coherently for
  solo and team users, before changing any behaviour.
- **Expected Outcome:** A design doc under `plans/designs/` that inventories the
  current commands, flags overlaps/inconsistencies (e.g. `update` vs `upgrade` vs
  `migrate`), and proposes the redesigned surface with aliases/migration for any
  rename. No renames land without this map approved.
- **Validation:** Design reviewed and linked from this module; `aps lint` clean.
- **Confidence:** medium
- **Design:** [CLI command-surface redesign map](../designs/2026-07-22-cli-redesign.design.md) — approved 2026-07-23 (D-CLI-a…d accepted; OQ-1 routed to CLI-003/CLI-006)
- **Status:** Complete — merged 2026-07-23 (PR #124, ancestor of `main` verified).
- **Files:** `plans/designs/2026-07-22-cli-redesign.design.md`
- **Dependencies:** _(none — this is the anchor)_

### CLI-002: Profile-aware command vocabulary (solo vs team)

- **Intent:** Make the CLI flex by `profile:` so a solo user sees a lean surface
  and a team sees first-class actor/claim/handoff-shaped verbs and help.
- **Expected Outcome:** With `profile: solo`, team-only affordances are hidden or
  de-emphasised; with `profile: team`, the team verbs and help surface. Behaviour
  is identical across Rust, bash, and PowerShell and covered by parity fixtures.
- **Validation:** `cargo test` + parity suite over `test/fixtures/**` for both profiles.
- **Confidence:** medium
- **Non-scope:** Implementing the underlying claim/lease mechanics (TEAM owns those).
- **Dependencies:** CLI-001, TEAM

### CLI-003: Binary upgrade and version-surface ergonomics

- **Intent:** Ship the redesign as a clean binary upgrade with a single, honest
  version surface and reliable self-update.
- **Expected Outcome:** `aps upgrade` and the version output behave consistently
  across channels; `cli_version` reconciliation (D-036/D-044) and the staleness
  check remain the single on-disk version stamp; release checklist updated.
- **Validation:** `cargo test`; a fetched/installed binary self-reports the new
  version and upgrades cleanly on Mac/Linux/Windows.
- **Confidence:** medium
- **Dependencies:** CLI-001, INSTALL

### CLI-004: Finish Gemini removal and confirm Grok end-to-end

- **Intent:** Close out D-040 in prose and verify Grok works through the full
  init/setup path, so the harness core is Claude Code, Copilot, Codex, OpenCode,
  and Grok with no stale Gemini references.
- **Expected Outcome:** No user-facing Gemini scaffolding references remain in
  `README.md`, `docs/installation.md`, or other prose (the `GEMINI.md`
  protected-migrate entry stays by design); `aps init` with `--tools grok`
  installs the Codex-shared assets and Grok discovers them.
- **Validation:** `grep -ri gemini` over live user-facing docs (`README.md`,
  `docs/**` excluding the historical `docs/plans/**` + `docs/release-notes/**`
  archives that record the removal) shows only the intentional `GEMINI.md`
  protected-list mention; init smoke test with `grok` selected passes in all three CLIs.
- **Confidence:** high
- **Status:** Complete (verification-only) — verified 2026-07-23; **no code or prose
  changes needed**. Both halves were already delivered by D-040 (v0.7) and the
  README reposition. Evidence:
  - _Gemini prose:_ `grep -ri gemini` over `README.md` + `docs/**` (excluding the
    historical `docs/plans/**` + `docs/release-notes/**` archives that record the
    removal) yields **zero** live scaffolding references. Remaining Gemini mentions
    are the intentional `GEMINI.md` protected-list entries and the D-040 retirement
    **error messages** that reject `gemini` (`cli/src/config.rs:71`,
    `lib/scaffold.sh:1010`, `lib/Scaffold.psm1:784`) — both stay by design.
  - _Grok end-to-end:_ `aps init --tools grok` installs the Codex-shared
    `.agents/skills/aps-planning/` payload — confirmed live for **Rust** and
    **bash**; **PowerShell** confirmed statically (`lib/Scaffold.psm1:608,725,846`)
    and via CI (`PowerShell parity` + `Native Windows PowerShell user journey`).
    Grok is documented as a supported harness (`README.md:190,264,286`,
    `docs/installation.md:208`); `cargo test config::` passes 16/16 (grok accepted,
    gemini rejected).
- **Files:** _(no changes — already satisfied on `main`)_ Verified against
  `README.md`, `docs/installation.md`, `lib/scaffold.sh`, `lib/Scaffold.psm1`, `cli/src/config.rs`
- **Dependencies:** _(none — completes prior D-040 work)_

### CLI-005: Harness-expansion research spike

- **Intent:** Decide which additional harnesses to support by testing candidates
  against the D-040 native-discovery gate (does it read the `AGENTS.md` family and
  discover `.agents/skills/` natively → cheap zero-asset add like Grok; or does it
  need bespoke assets → cost/benefit call).
- **Expected Outcome:** A short evaluation covering **Antigravity** (Google's
  agentic platform — verifiable), **Hermes**, **OpenClaw**, and any other
  credible harness, each marked verified/unverified with a go/no-go and the asset
  cost if go. Candidates I could not confirm exist get a discovery step before commitment.
- **Validation:** Evaluation linked from this module; a decision (D-045 below) records the approved set.
- **Confidence:** low
- **Evaluation:** [Harness-expansion spike](../research/2026-07-23-harness-expansion-spike.md) — 2026-07-23
- **Status:** Complete — evaluation delivered with primary-source citations; the
  D-045 set was **approved 2026-07-23** (below). Six zero-asset GOs (Antigravity,
  Amp, Gemini CLI, Windsurf, Roo Code, OpenClaw), Cursor split to CLI-007, Hermes
  deprioritised, the `AGENTS.md`-only crowd deferred.
- **Dependencies:** CLI-004, AGENT

### CLI-006: Add approved harnesses to init/setup/wizard

- **Intent:** Implement the harnesses approved by CLI-005 across the init/setup
  surface and scaffold asset install.
- **Expected Outcome:** The D-045-approved harnesses — **Antigravity, Amp,
  Gemini CLI, Windsurf, Roo Code, OpenClaw** — appear in the init/setup tool list,
  the TUI wizard, and the installers. Each is a **zero-asset add**: it reuses the
  Codex-shared `.agents/skills/aps-planning/` payload (Grok precedent), so no new
  per-harness generated assets. Landed in Rust, bash, and PowerShell with matching
  behaviour. **OpenClaw** is used as an orchestrator, so its support must expose
  plan management — the `aps` CLI reachable + the planning skill discoverable — not
  just passive skill install; its smoke test verifies `aps next/start/complete`.
- **Validation:** `cargo test` + parity suite; init smoke test selecting each new
  harness passes in all three CLIs (Grok precedent, CLI-004). OpenClaw additionally
  verified for a working plan-management path.
- **Confidence:** medium
- **Status:** Complete — merged 2026-07-24 (PRs #128 Antigravity, #129 the other
  five). All six install the shared skill in Rust/bash/pwsh with three-way parity;
  independently verified; regression tests iterate the full tool list.
- **Files:** `cli/src/config.rs`, `cli/src/scaffold.rs`, `cli/src/wizard.rs`,
  `cli/src/{setup,update,doctor,main}.rs`, `lib/scaffold.sh`, `lib/Scaffold.psm1`,
  `scaffold/install`, `scaffold/install.ps1`, docs, and `test/**` parity fixtures
- **Dependencies:** CLI-005

### CLI-007: Add Cursor

- **Intent:** Add Cursor to init/setup as a supported harness. Cursor reads
  `AGENTS.md` natively but does **not** scan the shared `.agents/skills/` path
  (it discovers skills from `.cursor/`, `.claude/`, `.codex/`). Because it scans
  `.claude/skills/`, APS reuses the existing claude-root payload there — no
  bespoke `.cursor/` asset needed (see the Build decision below).
- **Expected Outcome:** `aps init --tools cursor` installs the planning skill where
  Cursor discovers it. **Build decision:** rely on Cursor's opportunistic
  `.claude/skills/` scan (no `.cursor/` mirror needed) — Cursor reuses the
  existing `.claude/skills/aps-planning` payload, making it a **zero-asset** add
  after all. Landed in Rust, bash, and PowerShell with matching behaviour.
- **Validation:** `cargo test` + parity suite; init smoke test with `cursor`
  selected installs a Cursor-discoverable skill in all three CLIs.
- **Confidence:** medium
- **Status:** Complete — verified 2026-07-24. Cursor joins the `.claude/skills/`
  claude-root group; skill installs in Rust/bash/pwsh; `doctor`/`setup`/`skill_step`
  regression tests extended to cursor.
- **Files:** `cli/src/config.rs`, `cli/src/scaffold.rs`, `cli/src/wizard.rs`,
  `cli/src/{setup,doctor}.rs`, `lib/scaffold.sh`, `lib/Scaffold.psm1`, docs, `test/**`
- **Dependencies:** CLI-005

## Decisions

- **D-045 (accepted 2026-07-23):** Widen the harness set beyond the D-040 core.
  The native-discovery gate (reads `AGENTS.md` **and** auto-discovers
  `.agents/skills/<name>/SKILL.md` → zero-asset add) is the criterion. Per the
  CLI-005 [spike](../research/2026-07-23-harness-expansion-spike.md), the
  **approved set** is:
  - _Add as zero-asset (CLI-006):_ **Antigravity, Amp, Gemini CLI, Windsurf,
    Roo Code** (Roo Code's `AGENTS.md` auto-load is a toggle — document it).
  - _Add (CLI-006), as an orchestrator:_ **OpenClaw** — regularly used to drive
    plan management, so its support must expose the `aps` plan-management path
    (`next`/`start`/`complete`) + the discoverable planning skill, not just passive
    skill install. Confirm the plan-management path in its smoke test.
  - _Separate item **CLI-007** — **Cursor**:_ reads `AGENTS.md` and scans
    `.cursor/.claude/.codex` skills dirs. Build resolved to the `.claude/skills/`
    path (Cursor scans it), so it reuses the existing claude-root payload — a
    **zero-asset** add, no `.cursor/` mirror needed. (Delivered CLI-007.)
  - _Deprioritise (not approved):_ **Hermes** (global `~/.hermes/skills/` path +
    severe name collision).
  - _Not now:_ the `AGENTS.md`-only crowd (Aider, Zed, Devin, Jules, Warp,
    Continue, Factory, Junie, goose) — no shared `.agents/skills/` discovery yet.

  Note: "zero-asset" bounds bespoke **scaffold** cost only; each add still carries
  per-tool CLI plumbing + D-039 three-way parity + a real init smoke test.

## Notes

- Gemini→Grok is already shipped at the code level (D-040, v0.7); CLI-004 only
  closes the prose residue and verifies the end-to-end path.
- Every command-surface change here is bound by D-039 three-way lockstep — Rust,
  bash, and PowerShell are equal shipping targets, parity-verified.
