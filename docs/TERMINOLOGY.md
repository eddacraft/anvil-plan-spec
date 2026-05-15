# APS Terminology Update: Work Items & Action Plans

This document defines updated terminology and structure for APS planning and execution artefacts.

The goal is to make intent, execution, and verification clearly distinct — without introducing implementation detail into specs.

---

## Terminology Changes

### Renamed Concepts

| Old Term        | New Term    | Meaning                                                            |
| --------------- | ----------- | ------------------------------------------------------------------ |
| Task            | Work Item   | A bounded unit of work with intent, outcome, scope, and validation |
| Step (file)     | Action Plan | Execution breakdown for a work item                                |
| Step (numbered) | Action      | A coherent unit of execution within a plan                         |
| Checkpoint      | Checkpoint  | Observable proof that an action is complete                        |

---

## Core Principle (Updated)

- **Specs describe intent.**
- **Work Items define outcomes.**
- **Action Plans decompose execution.**
- **Checkpoints verify state.**

Implementation emerges from patterns and agent judgement.

---

## Hierarchy

| Layer       | Purpose             | You Write                          | You DON'T Write       |
| ----------- | ------------------- | ---------------------------------- | --------------------- |
| Index       | Plan overview       | Modules, milestones, risks         | Implementation detail |
| Module      | Bounded work area   | Interfaces, boundaries, work items | Code snippets         |
| Work Item   | Outcome unit        | Intent, outcome, validation        | How to implement      |
| Action Plan | Execution breakdown | Actions, checkpoints               | Tutorials             |
| Action      | Unit of work        | Purpose, produces, checkpoint      | Code structure        |
| Checkpoint  | Verification        | Observable state                   | Implementation steps  |

---

## Work Items

Work Items replace Tasks.

A Work Item is the contract: what will be achieved and how success is verified.

### Required Fields

- **Intent** — One sentence describing the outcome
- **Expected Outcome** — Testable or observable result
- **Validation** — Command or condition to verify completion

### Optional Fields

- Scope / Non-scope
- Dependencies
- Confidence
- Files (best-effort)
- Risks
- Deliverables

### Work Item Rules

- Work Items describe **what must be true**, not how it is achieved
- Work Items authorise change but do not prescribe implementation
- Validation must be deterministic where possible

---

## Action Plans

Action Plans replace Step files.

An Action Plan decomposes a Work Item into executable Actions.

### When to Create an Action Plan

Create an Action Plan when:

- The Work Item is non-trivial
- Multiple artefacts are produced
- Ordering or dependencies matter

**Simple Work Items may be executed without an Action Plan.**

---

## Action Plan Format

### File Naming

```
plans/execution/[WORK-ITEM-ID].actions.md
```

**Example:**

```
ADAPTER-OC-001.actions.md
```

### Header

```markdown
# Action Plan: ADAPTER-OC-001

Source: ../modules/[module].aps.md
Work Item: ADAPTER-OC-001 — [title]
Status: Draft | Ready | In Progress | Complete
Created by: [author]
```

---

## Actions

Actions replace numbered Steps.

Each Action represents a coherent unit of execution, not a tutorial.

### Action Format

```markdown
## Action N — [Action verb] [target]

**Purpose**
Why this action exists.

**Produces**
Concrete artefacts or state.

**Checkpoint**
Observable state (max ~12 words).

**Validate**
Command or condition (optional).
```

### Action Rules

- Actions may be procedural
- Actions may list artefacts
- Actions must include at least one Checkpoint
- Actions must not describe code structure or implementation tactics

---

## Checkpoints

Checkpoints are observable proofs of state, not instructions.

### Checkpoint Rules

- Must be verifiable by inspection or command
- Must avoid implementation detail
- Must remain valid even if implementation changes

### Examples

✅ **Good:**

- "All OpenCode events mapped to observation kinds"
- "Session start opens capsule; session end closes it"

❌ **Bad:**

- "Create mapping.ts with switch statement"
- "Use jsonwebtoken to validate tokens"

---

## Anti-Patterns

### ❌ Bad: Action as tutorial

```markdown
Checkpoint:

- Extract JWT from header
- Decode with jsonwebtoken
- Attach user to request
```

### ✅ Good: Observable state

```markdown
Checkpoint:
Authenticated requests attach user context
```

---

## Execution Rules for Agents

When asked to execute:

1. Locate the relevant Work Item
2. Ensure its status is Ready
3. Open or create the Action Plan if required
4. Execute one Action at a time
5. Validate the Checkpoint before proceeding
6. Mark the Work Item complete only when validation passes
7. Note the proof point (file location or date/time or successful test etc)

---

## Migration Guidance

- Existing **Task** sections become **Work Items**
- Existing **.steps.md** files become **.actions.md**
- Numbered **steps** become **Actions**
- Existing **Checkpoint:** lines remain unchanged

**No content needs to be rewritten unless it violates checkpoint rules.**
