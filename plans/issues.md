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
