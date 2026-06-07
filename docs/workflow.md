# APS Workflow Guide

This guide shows how APS fits into your daily development workflow through
concrete scenarios.

## The Planning Lifecycle

APS follows the compound engineering philosophy: each unit of work should make
future work easier. The workflow has four phases that loop back to planning:

```
Plan → Execute → Validate → Learn → Plan again
  ↑                                      │
  └──────────────────────────────────────┘
```

| Phase        | What Happens                | APS Artefacts                       | How It Serves Planning                 |
| ------------ | --------------------------- | ----------------------------------- | -------------------------------------- |
| **Plan**     | Define scope and success    | Designs, Index, Modules, Work Items | Reference past patterns and solutions  |
| **Execute**  | Work against specs          | Action Plans, status updates        | Clean implementation from clear specs  |
| **Validate** | Check outcomes against spec | Review notes, checklist             | Verify plan was correct, update if not |
| **Learn**    | Document solutions          | Solution docs in `docs/solutions/`  | Future plans start with known answers  |

**The 80/20 principle:** Spend 80% of effort on planning and validation, 20% on
execution. Thorough preparation means fast, clean implementation.

**Why the cycle matters:** Planning without validation is guesswork. Validation
without learning repeats mistakes. The cycle exists to make each plan better
than the last.

## Driving the loop with the CLI

The `aps` CLI is built around this lifecycle. The day-to-day shape is:

```bash
aps next                              # Plan → discover next ready work item
aps start AUTH-003                    # Execute → claim it, get a context package
# ...implement, run validation...
aps complete AUTH-003 --learning "..."  # Validate + Learn → capture insights
aps next                              # Loop
```

You can ignore the CLI and hand-edit Status fields if you prefer — but the CLI
enforces the state machine (Ready → In Progress → Complete), checks
dependencies, and writes Status lines in a consistent format. See
[usage.md](usage.md) for the full command reference.

## Scenarios

### Starting a Feature

You've been asked to add user authentication to an existing app.

1. **Assess scope.** Is this a single module or multiple? Auth typically
   touches login/logout, session management, password reset, maybe OAuth.
   That's at least 2 modules — create an Index.

2. **Write a design (optional).** If the architecture is non-obvious or you're
   comparing approaches (JWT vs sessions, bcrypt vs argon2), write a design doc
   in `designs/`. This captures the "why this approach" thinking before you
   commit to modules and work items.

3. **Create the Index:**

   ```markdown
   # Add User Authentication

   ## Problem

   Users currently have no way to log in. All data is public.

   ## Success Criteria

   - [ ] Users can register and log in
   - [ ] Sessions persist across browser refresh
   - [ ] Password reset flow works

   ## Modules

   | Module                              | Purpose                     | Status |
   | ----------------------------------- | --------------------------- | ------ |
   | [auth](./modules/auth.aps.md)       | Login, logout, registration | Draft  |
   | [session](./modules/session.aps.md) | Token management            | Draft  |
   ```

4. **Draft the modules.** Create `auth.aps.md` with Purpose and Scope. Leave
   Work Items empty — you're still exploring.

5. **Get approval.** Share the Index with your team or reviewer. Discuss
   scope, dependencies, risks.

6. **Move to Ready.** Once approved, change module status to Ready and add Work Items:

   ```markdown
   ## Work Items

   ### AUTH-001: Create registration endpoint

   - **Intent:** Allow new users to create accounts
   - **Expected Outcome:** POST /api/register creates user, returns token
   - **Validation:** curl test returns 201
   ```

7. **Execute.** Work through work items. If a work item is complex, create an Action Plan file.

---

### Mid-Implementation

You're halfway through AUTH-001 and realize you need database migrations.

1. **Add a new work item.** If the missing piece deserves its own outcome,
   write the spec, then drive it through the lifecycle:

   ```bash
   # After editing auth.aps.md to add AUTH-000 as Ready
   aps start AUTH-000
   # implement, run validation...
   aps complete AUTH-000 --learning "Migrations must run before schema-aware tests"
   ```

   `aps complete` stamps `- **Status:** Complete: 2026-05-12` (UTC date) and
   inserts the `- **Learning:**` line after `- **Validation:**`. That learning
   surfaces in dependency learnings when downstream items run `aps start`.

2. **Handle blockers.** APS doesn't have a `Blocked` transition in the CLI —
   blocking is a planning question, not a state-machine question. Capture the
   blocker in `plans/issues.md` and either:
   - Pause the item (leave it `In Progress`, add notes to the work item), or
   - Roll back to `Ready` by hand-editing Status and removing what was started.

   ```markdown
   ### ISS-001: OAuth credentials pending from infra

   | Field      | Value    |
   | ---------- | -------- |
   | Status     | Open     |
   | Severity   | medium   |
   | Discovered | AUTH-002 |
   | Module     | AUTH     |

   **Context:** Need Google API client ID/secret to finish AUTH-002.
   **Impact:** AUTH-002 paused until credentials arrive.
   ```

3. **Log discoveries.** Any issue or open question that emerges goes into
   `plans/issues.md` — keep them visible without cluttering work items.

4. **Status conventions.** The CLI uses four states for work items:
   - `Draft` — not yet ready to start
   - `Ready` — eligible for `aps start`
   - `In Progress` — `aps start` set this
   - `Complete: YYYY-MM-DD` — `aps complete` set this

   Items with no Status field default to `Ready` when read by the CLI.

---

### Handoff

You're going on vacation and someone else needs to continue your work.

1. **Update all statuses.** Make sure every work item reflects current state. The
   incoming dev should be able to look at the module and know exactly what's
   done and what's not.

2. **Document context.** Add notes for anything not obvious:

   ```markdown
   ## Notes

   - AUTH-002 is blocked on API credentials — ping @infra in Slack
   - The session token format follows RFC 7519 (JWT)
   - Tests are in `tests/auth/` — run with `npm test -- auth`
   ```

3. **Point to the spec.** Tell the incoming dev: "Start with
   `plans/modules/auth.aps.md` — it has everything you need."

---

### Completion and Archival

You've finished all work items in a module. Now what?

1. **Validate all work items.** Run each work item's validation command. Make sure
   everything actually works. `aps audit <module>` does this mechanically — it
   executes each Complete item's Validation command and reports
   PASS / FAIL / PARTIAL, plus understated Drafts and stale review dates (see
   [usage.md](usage.md#aps-audit-module--check-plan-state-against-reality)).

2. **Mark module complete.** Module status is hand-edited — the CLI only manages
   work item state. Bump the metadata table to `Complete` once every work item
   in the module is Complete:

   ```markdown
   | ID   | Owner | Status   |
   | ---- | ----- | -------- |
   | AUTH | @you  | Complete |
   ```

3. **Update the Index:**

   ```markdown
   | Module                        | Purpose                     | Status   |
   | ----------------------------- | --------------------------- | -------- |
   | [auth](./modules/auth.aps.md) | Login, logout, registration | Complete |
   ```

4. **Roll the task table into the completed archive.** Copy the module's work-item
   table into `plans/completed.aps.md`, grouped under the release that shipped
   it. This keeps the active index focused on in-flight work while preserving the
   task-by-task record for traceability. Use
   [`templates/completed-index.template.md`](../templates/completed-index.template.md)
   if the file doesn't exist yet:

   ```markdown
   ## v0.4.0 — Authentication

   ### Auth & Sessions

   | Task     | Module  | Description     | Status   |
   | -------- | ------- | --------------- | -------- |
   | AUTH-001 | auth    | Login flow      | Complete |
   | AUTH-002 | auth    | Session refresh | Complete |
   ```

   For long-form implementation notes — wave reports, post-mortems, deep dives —
   write a sibling file at `plans/completed/<release>-<module>.md` and link to
   it from the entry above. Skip the long-form file for ordinary modules; the
   task table alone is often enough.

5. **Decide on archival of the module spec itself:**

   | Approach                            | When to Use                                |
   | ----------------------------------- | ------------------------------------------ |
   | **Keep in `plans/modules/`**        | Ongoing reference, may need updates        |
   | **Move to `plans/archive/modules/`** | Historical record, unlikely to be revisited |
   | **Delete**                          | Ephemeral work, no long-term value         |

   Most teams keep specs indefinitely — they're lightweight and provide context
   for future work. Move them to `plans/archive/modules/` only when the active
   `modules/` directory starts to feel cluttered.

6. **For completed initiatives.** When all modules in an initiative are
   complete, update the Index:

   ```markdown
   | Field     | Value      |
   | --------- | ---------- |
   | Status    | Complete   |
   | Completed | 2025-01-15 |
   ```

   Consider writing a brief retrospective in Notes:

   ```markdown
   ## Notes

   Completed in 3 weeks. Key learnings:

   - OAuth took longer than expected due to credential delays
   - Session module was simpler than anticipated — could merge into auth next time
   ```

   For a release that bundles multiple modules, write a richer narrative in
   `plans/releases/<version>.md` rather than burying the story in Notes — see
   [Release Narrative](#release-narrative) below.

## Tips

### Keep specs in sync

Update specs as you work, not after. Stale specs lose trust.

### Don't over-specify

Work Items describe **what**, not **how**. Implementation details belong in code
and comments, not specs.

### Use Action Plans sparingly

Most work items don't need an action plan file. Only create one when:

- Work item has 5+ distinct actions
- Multiple agents or humans might work on it concurrently (waves)
- You want granular progress tracking

Action plans can be grouped into **waves** — actions in the same wave run in
parallel, waves run sequentially. Use waves when concurrent agents can pick up
independent slices of the same work item.

### Review specs in PRs

Include spec changes in your PRs. Reviewers should see what you planned
alongside what you built.

---

## Validate Phase

After completing work items, validate against the spec before shipping.

### Pre-Ship Checklist

Before merging or deploying, verify:

```markdown
## Review Checklist

### Functional

- [ ] All work item validations pass
- [ ] Edge cases handled
- [ ] Error states covered

### Quality

- [ ] Code follows existing patterns
- [ ] Tests added for new functionality
- [ ] No obvious security issues

### Documentation

- [ ] Spec reflects what was built
- [ ] README updated if needed
- [ ] Comments explain non-obvious code
```

### Multi-Perspective Review

For complex changes, consider multiple review angles:

| Perspective    | Questions to Ask                              |
| -------------- | --------------------------------------------- |
| **Developer**  | Is this easy to understand and modify?        |
| **Operations** | How do I deploy and troubleshoot this?        |
| **End User**   | Is the feature intuitive? Are errors helpful? |
| **Security**   | What's the attack surface? Is data protected? |

### Review in Practice

1. **Run validations.** Execute every work item's validation command:

   ```bash
   # From each work item
   curl -X POST /api/register -d '...'  # AUTH-001
   psql -c '\d users'                   # AUTH-000
   ```

2. **Check against spec.** Does the implementation match the Expected Outcome?

3. **Update spec if needed.** If implementation diverged from plan (for good
   reasons), update the spec to reflect reality.

4. **Note issues found.** If review catches problems, add them to `plans/issues.md`:

   ```markdown
   ### ISS-002: Rate limiting needed on AUTH-002

   | Field      | Value       |
   | ---------- | ----------- |
   | Status     | Open        |
   | Severity   | medium      |
   | Discovered | code review |
   | Module     | AUTH        |

   **Context:** Review identified missing rate limiting on login endpoint.

   **Impact:** Potential for brute force attacks without rate limiting.
   ```

   For questions that need team discussion, log them as questions:

   ```markdown
   ### Q-001: Should token expiry be configurable?

   | Field      | Value       |
   | ---------- | ----------- |
   | Status     | Open        |
   | Priority   | low         |
   | Discovered | code review |
   | Assigned   | @teamlead   |

   **Context:** Currently hardcoded to 1 hour. Some enterprise clients may need longer.
   ```

---

## Learn Phase

After solving problems, document solutions to inform future planning.

### Why Document Solutions?

| First Occurrence      | After Documenting       |
| --------------------- | ----------------------- |
| 30+ minutes debugging | 2 minute lookup         |
| Research from scratch | Reference past solution |
| Trial and error       | Known working approach  |

**Knowledge compounds.** Each documented solution makes future work faster.

### When to Document

Document immediately after fixing:

- **Non-trivial bugs** — Took multiple attempts to diagnose
- **Tricky configurations** — Easy to get wrong
- **Performance issues** — Required investigation
- **Integration problems** — External dependencies behaved unexpectedly

**Skip documentation for:**

- Simple typos or syntax errors
- Obvious issues with immediate fixes
- One-off problems unlikely to recur

### Solution Documentation Format

Create solution docs in `docs/solutions/` organized by category:

```text
docs/solutions/
├── performance/
│   └── n-plus-one-query-brief-system.md
├── configuration/
│   └── jwt-token-expiry-settings.md
├── integration/
│   └── oauth-redirect-uri-mismatch.md
└── database/
    └── migration-column-order-issue.md
```

Use [the solution template](../templates/solution.template.md) for the common
shape: Symptom, Investigation, Root Cause, Solution, Prevention, Related, and
Metadata.

Use a solution doc when the lesson is reusable across future work. Use an ADR
in `plans/decisions/` when the important artifact is a one-off project decision.
Cross-link both when a decision also produced a reusable implementation pattern.

### Learn Workflow in Practice

1. **Recognize the moment.** You just fixed something that took effort. Pause
   before moving on.

2. **Capture while fresh.** Context fades quickly. Document now, not later.

   ```bash
   mkdir -p docs/solutions/performance
   cp templates/solution.template.md docs/solutions/performance/n-plus-one-query.md
   ```

   Inline learnings from `aps complete --learning "..."` are a good seed for a
   solution doc. Expand the one-line learning into the root cause, durable fix,
   and prevention checklist while the context is still fresh.

3. **Cross-reference.** Link to related work items, PRs, and similar issues.

4. **Make it findable.** Use clear filenames and categories. Future you (or
   your teammates) will search for this.

5. **Update specs.** If the solution affects how work should be done, update
   relevant module specs or add to project conventions.

### Building a Knowledge Base

Over time, your `docs/solutions/` becomes a searchable knowledge base:

```bash
# Find past solutions
rg "timeout" docs/solutions/
rg "OAuth" docs/solutions/
```

**Patterns emerge.** After documenting 3+ similar issues, consider:

- Adding to project conventions
- Creating a checklist for common pitfalls
- Updating templates to prevent the issue

### Knowledge Compounds

| After 1 Month        | After 6 Months  | After 1 Year            |
| -------------------- | --------------- | ----------------------- |
| 5-10 solutions       | 30-50 solutions | 100+ solutions          |
| Occasional reference | Regular lookups | Comprehensive KB        |
| Individual knowledge | Team knowledge  | Institutional knowledge |

**Each solution documented makes future planning faster.** New team members
ramp up faster. Recurring problems get solved in minutes. Plans start with
known answers instead of research.

---

## Release Narrative

`CHANGELOG.md` tells users _what_ changed. A release narrative tells future
you _why_ this release exists — its theme, success criteria, risks, and how
the shipped modules tie together. Keep CHANGELOG entries terse and have them
link to the narrative for context.

### Where it lives

```text
plans/
├── releases/
│   ├── v0.3.0.md
│   ├── v0.4.0.md
│   └── ...
├── completed.aps.md           # task-table roll-up
└── completed/                 # optional long-form notes
    └── v0.3.0-orchestrate.md
```

`plans/releases/` is the standard location, parallel to `plans/index.aps.md`
and `plans/completed.aps.md`.

### When to write one

Write a release doc when:

- You're cutting a release that bundles work across multiple modules.
- The release marks a strategic shift you want findable six months later
  (framework swap, distribution change, breaking-change boundary).
- You want a single place to capture risks and success criteria the team
  agreed on before the cut.

Skip it for tiny patch releases — the CHANGELOG entry is enough on its own.

### How to write one

Use [`templates/release.template.md`](../templates/release.template.md) for
the canonical shape. The required spine is:

- **Release Theme** — one paragraph capturing the strategic narrative.
- **What Ships** — grouped tables of capabilities, with APS module IDs.
- **Success Criteria** — observable signals that the release worked.
- **Risks** — what's most likely to go wrong, with mitigations.
- **Related** — links to CHANGELOG, completed.aps.md, and module specs.

Add a **Retrospective** section after the release has been live long enough
to learn from. Bullet points, not essays.

### How it interacts with the other archives

| Artifact                  | Question it answers                              |
| ------------------------- | ------------------------------------------------ |
| `CHANGELOG.md`            | What changed in this version? (terse, for users) |
| `plans/releases/<v>.md`   | Why this release? Theme, criteria, risks         |
| `plans/completed.aps.md`  | Which work items shipped, grouped by release     |
| `plans/completed/<v>-<m>.md` | Wave-level implementation notes (optional)    |

Cross-link liberally. The CHANGELOG entry should point to the release doc;
the release doc should point back to the completed roll-up and the module
specs it covers.

---

## Complete Workflow Example

Putting it all together for a feature:

1. **Plan** — Create index and modules. Define work items with validation commands.
2. **Execute** — Work against specs. Create action plans for complex items.
3. **Validate** — Run validation commands. Check outcomes against spec. Update if diverged.
4. **Learn** — Document tricky problems solved. Add to solution library.
5. **Plan again** — Next feature references past solutions. Starts faster.
