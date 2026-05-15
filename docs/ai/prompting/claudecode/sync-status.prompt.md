# Sync Task Status to APS

Use this prompt at session end to update APS files with task completion status.

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

1. In the module file (plans/modules/{MODULE}.aps.md):
   - Mark completed work items with `Status: Complete (YYYY-MM-DD)`
   - Mark blocked items with `Status: Blocked: {reason}`
   - Add discovered work as new Draft work items

2. In the index (plans/index.aps.md):
   - Update "What's Next" section
   - Remove completed items
   - Add any new Ready items

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

- **Status:** Complete (2025-01-24)
  ...

### AUTH-002: Email verification

- **Status:** Complete (2025-01-24)
  ...

### AUTH-003: Password reset

- **Status:** Blocked: email provider API key not configured
  ...

### AUTH-004: Rate limiting for registration (Draft)

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
3. **Update "What's Next"** - Reflect current priority queue
4. **Session summary** - Brief note for next agent

Completed: {list tasks}
Blocked: {list with reasons}
Discovered: {list new work}

Then show the full git diff for my review before committing.
```
