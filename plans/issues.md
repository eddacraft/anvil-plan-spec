# Issues & Questions Tracker

> Development-time discoveries. See `plans/aps-rules.md` → "Issues & Questions
> Tracker" for conventions.

---

## Issues

### ISS-001: Markdown parser is not fence-aware outside build_id_index

| Field      | Value                 |
| ---------- | --------------------- |
| Status     | Closed (2026-07-16)   |
| Severity   | Medium                |
| Discovered | DOGFOOD-002           |
| Module     | VAL                   |

**Context:** Council review (session council-b459ae20) showed `### FAKE-999:`
inside a fenced code block is treated as a real work item by
`get_work_items`, triggering false E005 errors and polluting `aps next` /
`aps graph` item lists. `build_id_index` (cross-file W003) was made
fence-aware during DOGFOOD-002; the shared parser helpers were not.

**Impact:** Code blocks containing example work-item headers produce
false lint errors and phantom orchestration items.

**Resolution (VAL-002):** All shared parser helpers are now fence-aware in
all three CLIs (D-039): `get_work_items` / `Get-ApsWorkItems` /
`PlanFile::work_items`, the E005/W018 content extraction, the W003
in-file ID set and Dependencies scan, and the orchestrate item-content /
status-rewrite scanners. Fenced headers are invisible as items and inert
as terminators. Pinned by `test/fixtures/valid/fenced-examples.aps.md`
(cross-CLI parity corpus), test/run.sh Test 18c, and a Rust parser test;
three-way parity verified bash = Rust = PowerShell.

---

### ISS-002: Link checks follow paths outside the plan root

| Field      | Value       |
| ---------- | ----------- |
| Status     | Open        |
| Severity   | Low         |
| Discovered | DOGFOOD-002 |
| Module     | VAL         |

**Context:** W019 / A004 existence checks resolve `../`-style link targets
without confining them to the plan root, so an out-of-tree file that exists
passes as a valid module link, and lint output echoes attacker-chosen paths
(a filesystem existence oracle when logs are shared). Flagged by the council
security reviewer; deferred — requires `realpath` canonicalisation across
bash + PowerShell engines.

**Impact:** Semantically invalid index links pass; minor information
disclosure in shared CI logs.

---

### ISS-003: Plan-status behaviour is split across deprecated and current surfaces

| Field      | Value      |
| ---------- | ---------- |
| Status     | Open       |
| Severity   | Medium     |
| Discovered | 2026-07-16 |
| Module     | CIB        |
| Work Item  | CIB-001    |

**Context:** APS decisions D-015 and D-023 say `/plan-status` behaviour belongs
inside the APS planning skill and active command files are no longer shipped.
The planner agent contains the standard report, but the installed planning
skill does not explicitly own the natural-language query, while the Rust setup
path still writes deprecated Claude command files.

**Impact:** Different installation and agent surfaces can answer the same status
request differently, and duplicated instructions can drift.

**Tracking:** [CIB-001](./modules/continuous-improvement-backlog.aps.md)

---

### ISS-004: Public curl installation does not enter the native TUI

| Field      | Value      |
| ---------- | ---------- |
| Status     | Closed (2026-07-17) |
| Severity   | Medium     |
| Discovered | 2026-07-16 |
| Module     | CIB        |
| Work Item  | CIB-002    |

**Context:** In the observed first-run journey, the no-argument public
`curl | bash` entrypoint presents the shell mode picker. The richer native TUI
appears only after installation when the user separately runs `aps init`, even
though installer decision D-029 calls for handing off to the same choice model.

**Impact:** First-time users see two different setup experiences and must infer
that a second command is required to reach the intended initializer.

**Implementation:** The default interactive installer now performs native
onboarding in one run; explicit `--onboard` and `--menu` modes keep automation
and advanced choices deterministic. Close after native Windows CI evidence.

**Tracking:** [CIB-002](./modules/continuous-improvement-backlog.aps.md)

---

### ISS-005: Monorepo init can produce the single-project root index

| Field      | Value      |
| ---------- | ---------- |
| Status     | Closed (2026-07-17) |
| Severity   | Medium     |
| Discovered | 2026-07-16 |
| Module     | CIB        |
| Work Item  | CIB-003    |

**Context:** In the observed native init journey, selecting Monorepo installs
the monorepo template asset but the generated `plans/index.aps.md` uses the old
single-project index. Source-level scaffold tests already assert the intended
monorepo content, so the mismatch may be in the released binary, wizard state,
or the end-to-end selection path rather than the pure scaffold planner.

**Impact:** The generated plan contradicts the reviewed setup choice and starts
a monorepo with the wrong planning structure.

**Implementation:** Project shape now owns root-index generation, wizard shape
changes repair the root-template selection, contradictory flags are rejected,
and native user-journey tests assert monorepo index content plus config. Close
after the native Windows shape journey passes.

**Tracking:** [CIB-003](./modules/continuous-improvement-backlog.aps.md)

---

### ISS-006: Windows user journeys lack native runtime validation

| Field      | Value      |
| ---------- | ---------- |
| Status     | Closed (2026-07-17) |
| Severity   | Medium     |
| Discovered | 2026-07-17 |
| Module     | CIB        |
| Work Item  | CIB-004    |

**Context:** CI cross-compiles the Rust CLI for Windows and runs PowerShell
parity under Ubuntu, but no native Windows job exercises `aps.exe` through a
representative user journey from PowerShell.

**Impact:** Windows-specific path, process, encoding, executable, or installer
regressions can ship while existing portability checks remain green.

**Tracking:** [CIB-004](./modules/continuous-improvement-backlog.aps.md)

---

### ISS-007: Rust lint misclassifies Windows paths

| Field      | Value      |
| ---------- | ---------- |
| Status     | Closed (2026-07-17) |
| Severity   | Medium     |
| Discovered | 2026-07-17 |
| Module     | CIB        |
| Work Item  | CIB-004    |

**Context:** The first native Windows user journey reached `aps lint` with a
valid monorepo root, but Rust path classification treated
`plans\index.aps.md` as a simple module because its rules only recognised `/`
separators.

**Impact:** Native Windows lint applies the wrong validation rules to indexes,
modules, actions, and releases even though the same project passes on Unix.

**Implementation:** Normalise separators at the parser classification boundary
and cover Windows-style paths with parser and lint regression tests. Close
after the native Windows journey passes.

**Tracking:** [CIB-004](./modules/continuous-improvement-backlog.aps.md)

---

### ISS-008: Windows PowerShell 5.1 misparses installer string

| Field      | Value      |
| ---------- | ---------- |
| Status     | Closed (2026-07-17) |
| Severity   | Medium     |
| Discovered | 2026-07-17 |
| Module     | CIB        |
| Work Item  | CIB-004    |

**Context:** After PowerShell 7 completed the full native journey, Windows
PowerShell 5.1 parsed the UTF-8 installer without a BOM. An em dash inside a
double-quoted error message was decoded with a smart-quote byte, prematurely
closing the string and producing a misleading missing-brace error.

**Impact:** The public installer cannot be parsed by Windows PowerShell 5.1
even though its PowerShell 7 journey passes.

**Implementation:** Keep executable installer strings ASCII-safe where
un-BOMed source decoding can create PowerShell quote characters. Close after
the Windows PowerShell 5.1 journey passes.

**Tracking:** [CIB-004](./modules/continuous-improvement-backlog.aps.md)

---

### ISS-009: PowerShell 5.1 promotes expected native stderr

| Field      | Value      |
| ---------- | ---------- |
| Status     | Closed (2026-07-17) |
| Severity   | Low        |
| Discovered | 2026-07-17 |
| Module     | CIB        |
| Work Item  | CIB-004    |

**Context:** Windows PowerShell 5.1 completed installation, hooks, and lint in
the native journey, then promoted the expected stderr from an empty `aps next`
queue into a terminating `NativeCommandError` because the harness redirects
native stderr while using `ErrorActionPreference = Stop`.

**Impact:** The compatibility gate fails before it can assert the documented
exit code and status message even though the CLI behaves correctly.

**Implementation:** Capture commands with expected non-zero results under a
temporary non-terminating preference, preserve their output and exit code, and
restore strict error handling immediately afterwards. Close after the Windows
PowerShell 5.1 journey passes.

**Tracking:** [CIB-004](./modules/continuous-improvement-backlog.aps.md)

---

### ISS-010: plan-doctor rules disagree with the documented status vocabulary

| Field      | Value               |
| ---------- | ------------------- |
| Status     | Closed (2026-09-12) |
| Severity   | High       |
| Discovered | 2026-07-26 |
| Module     | CIB        |
| Work Item  | CIB-005    |

**Context:** The bundled `plan-doctor` skill hard-codes a status set and a
filename convention that `plans/aps-rules.md` does not require. W03 omits the
documented `Proposed` → `Draft` and `Done` → `Complete` aliases, W04 inherits
the same gap by treating only `Complete`/`Merged`/`Released`/`Shipped` as
terminal, and W02 flags every module whose filename lacks a numeric prefix —
a convention no consumer plan follows.

**Impact:** Roughly 385 warnings fire on a consumer plan with no real defects
(162 `Done` and 132 `Proposed` statuses, plus all 91 module filenames), which
makes the skill unusable and trains readers to ignore its output. The skill
bytes are managed and embedded in the binary, so a consumer cannot patch it
locally without desyncing `.aps-managed.json`.

**Tracking:** [CIB-005](./modules/continuous-improvement-backlog.aps.md),
[issue #132](https://github.com/eddacraft/anvil-plan-spec/issues/132)

---

### ISS-011: Release-plan lint rules exist only in the Rust CLI

| Field      | Value               |
| ---------- | ------------------- |
| Status     | Closed (2026-09-12) |
| Severity   | Medium     |
| Discovered | 2026-09-12 |
| Module     | CIB        |
| Work Item  | CIB-006    |

**Context:** R001–R004 were implemented in `cli/src/lint.rs` only. The bash and
PowerShell linters never discover `plans/releases/v*.md`, so on this repo's own
plans bash reports 42 files checked where the Rust binary reports 49. The
cross-CLI parity suite carries no release fixtures, so CI cannot see the gap.

**Impact:** A malformed release plan passes both fallback CLIs silently,
breaking the D-039 lockstep guarantee that one command surface behaves
identically across the three implementations.

**Tracking:** [CIB-006](./modules/continuous-improvement-backlog.aps.md)

---

### ISS-012: Completed-work archive was never rolled for v0.4.0–v0.8.1

| Field      | Value      |
| ---------- | ---------- |
| Status     | Open       |
| Severity   | Low        |
| Discovered | 2026-09-12 |
| Module     | REL        |
| Work Item  | REL-005    |

**Context:** `plans/completed.aps.md` jumps from theme-compacted pre-v0.7.0
tables to v0.7.0 and then to the v0.9.0 roll. The task tables for v0.4.0,
v0.5.0, v0.6.0, v0.8.0, and v0.8.1 were never archived — roughly sixty items
across `MONO`, `PKG`, `CIP`, `COND`, `INSTALL`, `REL`, `SPEC`, `TASKS`, and
`CLI` are Complete in their modules but absent from the archive. Until the
2026-09-12 release review, everything up to and including v0.7.0 also sat
under a `## Unreleased` heading, which is how the omission stayed invisible.

**Impact:** The archive cannot be used to answer "what shipped when" for five
releases, and the roll-up convention in `AGENTS.md` ("roll the task table into
`plans/completed.aps.md`") is not being met. Low severity because the module
files remain authoritative and the release narratives record the content.

**Implementation:** Backfill mechanically rather than by hand — this is the
closeout sweep REL-005's `aps release close` is specified to perform from the
prose release records, and the v0.5.0 record is already its acceptance target.
Reconstructing attribution by hand risks fabricating it, so the false
`Unreleased` label was corrected on review and the backfill left to the tool.

**Tracking:** [REL-005](./modules/release-planning.aps.md)

---

### ISS-013: Fallback CLIs cannot report their own version

| Field      | Value      |
| ---------- | ---------- |
| Status     | Open       |
| Severity   | Low        |
| Discovered | 2026-09-12 |
| Module     | CIB        |
| Work Item  | CIB-007    |

**Context:** The Rust binary answers `aps --version` with `aps 0.9.0`. The bash
CLI rejects both `--version` and `version` as unknown commands and lists no
version entry in `aps --help`; the PowerShell CLI has no version surface
either. Both fallbacks nonetheless know their version — each defaults
`APS_CLI_VERSION` and writes it as `cli_version` into `.aps/config.yml`.

**Impact:** A user on a fallback CLI who hits the `cli_version` pin-mismatch
warning has no way to ask the CLI which version it is, which is the one
question the warning raises. It also breaks the D-039 expectation that one
command surface behaves identically across the three implementations.

**Tracking:** [CIB-007](./modules/continuous-improvement-backlog.aps.md)

---

### ISS-014: Three residual lint divergences across the CLI implementations

| Field      | Value      |
| ---------- | ---------- |
| Status     | Open       |
| Severity   | Low        |
| Discovered | 2026-09-12 |
| Module     | CIB        |
| Work Item  | CIB-008    |

**Context:** Surfaced while porting R001–R004 for CIB-006, on targets outside
that item's scope. (1) `cli/scaffold` is a symlink to `../scaffold`; Rust's
`is_dir()` follows it, so `aps lint .` lints two extra files and raises a W019
that bash `find` and PowerShell `Get-ChildItem -Recurse` never see. (2)
PowerShell's `Find-ApsFiles` uses `Sort-Object FullName`, which is
culture-aware, while Rust and bash sort byte-order — with mixed-case sibling
filenames in one directory this reorders findings. (3) bash and PowerShell run
the cross-tree W020/W021 collision checks before the per-file loop, so a
federation parent's group is emitted first, whereas Rust emits in path order.

**Impact:** `aps lint .` and mixed-case plan trees can produce different output
from different CLIs, which is the condition D-039 exists to prevent. Low
severity: none of the three affects `aps lint plans` on a conventional tree,
which is the documented invocation, and the parity suite is green.

**Implementation:** Fix in all three CLIs together or record as accepted
differences with a note in the parity suite. The PowerShell ordinal-sort fix
changes ordering for every file type, so it needs its own parity pass.

**Tracking:** [CIB-008](./modules/continuous-improvement-backlog.aps.md)

---

### ISS-015: Release-plan rules are weaker than they read

| Field      | Value      |
| ---------- | ---------- |
| Status     | Open       |
| Severity   | Low        |
| Discovered | 2026-09-12 |
| Module     | CIB        |
| Work Item  | CIB-008    |

**Context:** R001 only requires a literal `v`, one ASCII digit, and a `.md`
extension, so `v0garbage.md` and `v9.aps.md` pass as well-formed release
filenames — it does not validate a version. R002 is satisfied by any two lines
in the first twenty that start `| Target |` and `| Status |`; they need not sit
in the same table, or in a table at all. Separately, Rust classifies
`plans/releases/foo.aps.md` as a release file, so a module placed under
`releases/` silently loses all module linting.

**Impact:** A malformed release record can pass the structural gate, and a
misfiled module is validated by the wrong rule set with no warning. Not a
regression — this is the behaviour CIB-006 deliberately mirrored into bash and
PowerShell rather than diverging from.

**Tracking:** [CIB-008](./modules/continuous-improvement-backlog.aps.md)

---

### ISS-016: Local clippy silently disagrees with CI's

| Field      | Value      |
| ---------- | ---------- |
| Status     | Open       |
| Severity   | Medium     |
| Discovered | 2026-09-12 |
| Module     | CIB        |
| Work Item  | CIB-009    |

**Context:** CI installs `dtolnay/rust-toolchain@stable` and the repo pins no
toolchain, so CI ran clippy 0.1.98 while a contributor environment had 0.1.94.
`clippy::unnecessary_sort_by` does not exist in the older version, so
`cargo clippy --locked --all-targets -- -D warnings` reported zero errors
locally and failed the `Rust CLI` job on the identical commit.

**Impact:** Every pre-push gate can pass and still turn CI red, which is the
one outcome the gates exist to prevent. It costs a CI cycle and teaches
contributors to distrust local validation. The failure mode is silent: there is
no warning that the local lint set is a subset of CI's.

**Implementation:** Proposed fix is `rust-toolchain.toml` with
`channel = "stable"`, so rustup resolves the same current stable CI uses,
plus a `CONTRIBUTING.md` note. Pinning an exact version is the alternative and
needs CI pinned to match. Decision recorded in CIB-009.

**Tracking:** [CIB-009](./modules/continuous-improvement-backlog.aps.md)

---

## Questions

### Q-001: Which shared claim transport should team mode use?

| Field      | Value      |
| ---------- | ---------- |
| Status     | Open       |
| Priority   | High       |
| Discovered | TEAM-000   |
| Assigned   | unassigned |

**Context:** Local exclusive files can coordinate processes sharing one
filesystem, but team projects also need an atomic single winner across
independent clones. The transport must preserve APS portability and auditability
without requiring a hosted APS database.

**Options considered:**

1. Git refs/objects — compare-and-set and outside the worktree, but remote
   namespace support, permissions, discovery, and cleanup need proof.
2. Tracked claim files — visible and portable, but create default-branch churn
   and reproduce the merge conflicts claims are meant to prevent.
3. Provider state — authenticated and visible, but provider-specific and not
   available offline.
4. Required hosted service — strong shared transactions, rejected by current
   APS constraints.

**Related:** TEAM-000, TEAM-001, TEAM-002

---

### Q-002: How do claims affect the effective and declared work-item status?

| Field      | Value      |
| ---------- | ---------- |
| Status     | Open       |
| Priority   | High       |
| Discovered | TEAM-000   |
| Assigned   | unassigned |

**Context:** Claims should immediately hide contested work from team-aware
`next`, but changing shared APS markdown during claim acquisition causes the
same cross-branch conflicts the coordination plane is intended to avoid. The
design separates declared status from effective coordination views, but must
decide when a successful claim becomes durably `In Progress`.

**Options considered:**

1. Claim only changes the effective view; the actor's implementation PR carries
   the durable `In Progress`/completion edits.
2. A coordinator writes `In Progress` directly to the integration branch.
3. Claim changes the actor's branch copy only, while shared claim state prevents
   another actor selecting the item.

**Related:** TEAM-000, ORCH-002

---

### Q-003: How are actor identity, lease expiry, and takeover trusted?

| Field      | Value      |
| ---------- | ---------- |
| Status     | Open       |
| Priority   | High       |
| Discovered | TEAM-000   |
| Assigned   | unassigned |

**Context:** A local actor name is self-asserted, while a provider can bind an
operation to an authenticated user. Expiry must recover crashed agents without
allowing clock skew, a short pause, or a malicious claimant to steal live work.
APS must define useful coordination semantics without becoming an identity or
authorisation provider.

**Options considered:**

1. Advisory identity with holder-only renew/release and explicit audited
   takeover after expiry.
2. Provider-authenticated identity when available, local advisory fallback.
3. Human approval for every takeover, safer but unsuitable for unattended
   recovery.

**Related:** TEAM-000, TEAM-001, TEAM-002, TEAM-003

---
