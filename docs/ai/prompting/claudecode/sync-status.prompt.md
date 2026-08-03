# Sync Task Status to APS

This variant defers to the [generic module](../module.prompt.md) and [index](../index.prompt.md) prompts; only Claude Code-specific differences follow.

Use this prompt at session end to reconcile Claude Code Task outcomes with APS.

---

## Prompt

```
Session complete. Here's what was accomplished:

Completed tasks:
- {TASK_ID}: {description}
- {TASK_ID}: {description}

Blocked tasks:
- {TASK_ID}: blocked because {reason}

New work discovered:
- {description of new work item}

Please update the APS files:

Follow the repository's `AGENTS.md` and the current APS file structure for
lifecycle and index rules.

1. In the module file (`plans/modules/{MODULE}.aps.md`), map Claude Code Task
   outcomes to canonical APS status fields:
   - Completed: `- **Status:** Complete: YYYY-MM-DD`
   - Blocked: `- **Status:** Blocked: {reason}`
   - Discovered: add a work item with `- **Status:** Draft`

2. Reconcile the index only where repository guidance and its existing structure
   require it. Do not add or remove a "What's Next" section merely for this sync.

3. Show me the git diff of changes for review.
```

---

## Example

Input:

```
Completed tasks:
- AUTH-001: User registration flow
- AUTH-002: Email verification

Blocked tasks:
- AUTH-003: blocked because email provider API key not configured

New work discovered:
- Need rate limiting on registration endpoint
```

Claude updates:

```markdown
# In plans/modules/02-auth.aps.md

### AUTH-001: User registration flow

- **Status:** Complete: 2025-01-24
  ...

### AUTH-002: Email verification

- **Status:** Complete: 2025-01-24
  ...

### AUTH-003: Password reset

- **Status:** Blocked: email provider API key not configured
  ...

### AUTH-004: Rate limiting for registration

- **Status:** Draft
- **Intent:** Prevent abuse of registration endpoint
- **Expected Outcome:** Registration endpoint returns 429 after N requests/minute
- **Validation:** `npm test -- rate-limit`
- **Confidence:** medium
- **Dependencies:** AUTH-001
```

---

## Automated Session End Ritual

Combine with the full session end ritual:

```
Session ending. Please perform the APS session end ritual:

1. **Update status** - Mark work items based on task completion
2. **Capture discovered work** - Add new Draft items I mentioned
3. **Reconcile the index** - Follow repository guidance and existing structure
4. **Session summary** - Brief note for next agent

Completed: {list tasks}
Blocked: {list with reasons}
Discovered: {list new work}

Then show the full git diff for my review before committing.
```
