# Kindling Integration Research: Factual Completion Capture for APS

## Context

This document analyzes how [Kindling](https://github.com/EddaCraft/kindling) — a local memory and continuity engine — could integrate with APS to provide **factual, machine-captured evidence** that work items and checkpoints were actually completed. It also incorporates relevant ideas from the VSDD (Verified Spec-Driven Development) methodology around traceability and convergence.

## The Gap in APS Today

APS currently defines a clear hierarchy: Index → Module → Work Item → Action Plan → Checkpoint. Each work item has a Validation field (a command or condition). Each action has a Checkpoint (observable proof of state).

**What's missing:** APS trusts the agent to *self-report* completion. The agent marks a work item "Complete" and notes a proof point, but there's no independent record of:

- What tool calls were made during execution
- What commands were run and their actual output
- What files were changed and the diffs
- What errors occurred and how they were resolved
- Whether the validation command actually passed (vs. the agent just claiming it did)
- The timeline of execution (how long, what order, what was retried)

This is the difference between a **spec** (what should happen) and an **observation log** (what actually happened). APS has specs. Kindling captures observations. Together they close the loop.

## What Kindling Provides

Kindling's core primitives map directly to APS concepts:

| Kindling Concept | APS Equivalent | Integration Role |
|------------------|---------------|------------------|
| **Capsule** | Work Item execution session | Groups all observations for one work item |
| **Observation** (tool_call) | Action execution | Records what the agent actually did |
| **Observation** (command) | Validation run | Captures command output + exit code |
| **Observation** (file_diff) | Implementation artifact | Records what changed |
| **Observation** (error) | Blocker / Issue discovery | Captures failures during execution |
| **Pin** | Critical finding | Marks important observations for review |
| **Capsule summary** | Work item completion note | Auto-generated summary of what happened |

## Integration Design

### Level 1: Session-Level Capture (Minimal)

Hook Kindling into APS's existing hook points. No spec format changes needed.

**SessionStart hook** — open a Kindling capsule:

```bash
kindling capsule open \
  --intent "APS session: $(cat plans/index.aps.md | head -1 | sed 's/^# //')" \
  --repo .
```

**Stop hook** — close the capsule with summary:

```bash
kindling capsule close $CAPSULE_ID \
  --summary "$(git diff --stat HEAD~1 HEAD)"
```

**Value:** Every APS session gets a searchable record. Future sessions can `kindling search "auth"` to find what happened last time. This alone solves the handoff problem (workflow.md lines 149-167) without relying on manually written notes.

### Level 2: Work Item-Level Capture (Recommended)

Map Kindling capsules 1:1 to work item execution. This is the sweet spot.

**When an agent starts executing a work item**, it opens a capsule:

```bash
kindling capsule open \
  --intent "Execute AUTH-001: Create registration endpoint" \
  --repo .
```

The capsule ID is stored in the action plan or work item metadata.

**During execution**, the Claude Code adapter auto-captures:

- Every tool call (Read, Edit, Write, Bash) → `tool_call` observations
- Every command run → `command` observations with exit codes
- Every file edit → `file_diff` observations

**When validation runs**, the output is explicitly captured:

```bash
kindling log --kind command "Validation: AUTH-001" \
  --exit-code $? --output "$(curl -X POST /api/register -d '...' 2>&1)"
```

**When the work item completes**, the capsule is closed:

```bash
kindling capsule close $CAPSULE_ID \
  --summary "AUTH-001 complete. Registration endpoint returns 201. Validation passed."
```

**Value:** You can now answer "Did AUTH-001 actually pass validation?" with machine-captured evidence, not agent self-report. The capsule contains the full execution trace.

### Level 3: Checkpoint-Level Capture (Full Traceability)

Each action's checkpoint gets a corresponding Kindling observation that records the verification result. This creates the full traceability chain:

```
Spec Requirement → Work Item → Action → Checkpoint → Kindling Observation (proof)
```

This mirrors VSDD's contract chain concept but stays lightweight — it's just observations being tagged with the checkpoint they verify.

## APS Format Changes

### Minimal: No format changes

Level 1 and 2 work purely through hooks and CLI. The agent opens/closes capsules as part of execution. No spec format changes needed.

### Optional: Capsule reference in work items

Add an optional field to work items:

```markdown
### AUTH-001: Create registration endpoint

- **Intent:** Allow new users to create accounts
- **Expected Outcome:** POST /api/register creates user, returns token
- **Validation:** `curl -X POST /api/register -d '...'` returns 201
- **Capsule:** `cap_a1b2c3d4` *(auto-populated on execution)*
```

This provides a direct link from the spec to its execution evidence.

### Optional: Evidence section in action plans

Add a completion evidence block to action plans:

```markdown
## Completion

- [x] All checkpoints validated
- [x] Work item marked complete in source module

**Evidence:** `kindling inspect capsule cap_a1b2c3d4`
**Completed by:** AI (Claude Code)
**Validation output:**
```

## Hook Integration

APS already has four hook points (SessionStart, PreToolUse, PostToolUse, Stop). Kindling has a Claude Code adapter with three hooks (SessionStart, PostToolUse, Stop). The overlap is almost perfect.

### Combined hooks approach

The APS install script could detect Kindling and wire both sets of hooks:

```json
{
  "hooks": {
    "SessionStart": [
      { "type": "command", "command": "./aps-planning/scripts/init-session.sh" },
      { "type": "command", "command": "kindling-hook session-start" }
    ],
    "PreToolUse": [
      {
        "matcher": "Write|Edit|Bash",
        "hooks": [
          { "type": "command", "command": "./aps-planning/scripts/pre-tool-check.sh" }
        ]
      }
    ],
    "PostToolUse": [
      {
        "matcher": "Write|Edit",
        "hooks": [
          { "type": "command", "command": "./aps-planning/scripts/post-tool-nudge.sh" }
        ]
      },
      {
        "hooks": [
          { "type": "command", "command": "kindling-hook post-tool-use" }
        ]
      }
    ],
    "Stop": [
      { "hooks": [{ "type": "command", "command": "./aps-planning/scripts/check-complete.sh" }] },
      { "hooks": [{ "type": "command", "command": "./aps-planning/scripts/enforce-plan-update.sh" }] },
      { "hooks": [{ "type": "command", "command": "kindling-hook stop" }] }
    ]
  }
}
```

### APS-specific Kindling hooks

New APS scripts could call Kindling directly:

- **init-session.sh** — open a Kindling capsule alongside the session baseline
- **check-complete.sh** — on work item completion, close the capsule and log the validation result
- **enforce-plan-update.sh** — include capsule ID in the enforcement check (did the agent record evidence?)

## VSDD Concepts Worth Incorporating

From the VSDD analysis, three ideas enhance this integration:

### 1. Adversarial Review Protocol

Kindling capsules provide the raw material for review. A reviewer (human or adversarial AI) can `kindling inspect capsule cap_xxx` to see exactly what happened, rather than trusting the agent's summary. This is VSDD's "Adversary with fresh context" but using factual evidence instead of re-reviewing code.

### 2. Convergence Signal

VSDD's termination criterion ("the adversary is forced to hallucinate flaws") maps to a concrete Kindling query: "Does every work item have a closed capsule with a passing validation observation?" If yes, the module is converged. If not, there's unfinished work.

### 3. Explicit Traceability Chain

APS's chain, made explicit with Kindling:

```
Index → Module → Work Item → Action Plan → Checkpoint → Kindling Observation
  (why)   (where)   (what)      (how)        (verify)      (proof it happened)
```

Every link is either a markdown file (spec-side) or a database record (evidence-side). Fully traceable from intent to proof.

## Implementation Priority

| Priority | What | Effort | Dependency |
|----------|------|--------|------------|
| **P0** | Document the integration pattern (this doc + guide) | Low | None |
| **P1** | Update `install-hooks.sh` to detect Kindling and wire combined hooks | Low | Kindling CLI installed |
| **P2** | Add optional `Capsule:` field to work item template | Trivial | None |
| **P3** | Add `--kindling` flag to scaffold to auto-configure | Medium | P1 |
| **P4** | Agent guide updates: instruct agents to open/close capsules per work item | Low | P0 |
| **P5** | Linter rule: warn if completed work items lack capsule reference (when Kindling detected) | Medium | P2 |

## What NOT to Do

- **Don't make Kindling required.** APS is tool-agnostic. Kindling is an optional enhancement.
- **Don't store Kindling data in plan files.** Capsule IDs are references, not evidence dumps. The evidence lives in Kindling's SQLite database.
- **Don't duplicate Kindling's capture in APS hooks.** Let Kindling's adapter handle observation capture. APS hooks handle spec-level concerns (status updates, plan enforcement).
- **Don't add Kindling-specific linter errors.** Warnings at most — many valid APS deployments won't have Kindling.

## Open Questions

1. **Capsule granularity:** One capsule per session, per work item, or per action? Per work item seems right for traceability, but per session is simpler. Could support both with nested scopes.
2. **Kindling in CI:** Should APS lint check for capsule references on completed work items? Only as a warning, and only if Kindling is detected.
3. **Cross-session continuity:** If a work item spans multiple sessions, should each session get its own capsule, or should there be one capsule per work item with multiple sessions appending to it?
4. **Agent instructions:** How explicitly should the agent guide instruct agents to use Kindling? Behavioral nudge in the skill, or explicit steps in the execution workflow?
