# Solution Template

Use this template to document solved problems in `docs/solutions/`. Each
documented solution compounds team knowledge—making future occurrences faster
to resolve.

## When to Use

Document immediately after fixing:

- Non-trivial bugs that took investigation
- Tricky configurations easy to get wrong
- Performance issues requiring debugging
- Integration problems with external dependencies

**Skip** for: simple typos, obvious fixes, one-off issues.

## File Organization

Place solutions in category directories:

```text
docs/solutions/
├── performance/          # N+1 queries, slow endpoints, memory issues
├── configuration/        # Settings, environment variables, secrets
├── integration/          # APIs, OAuth, external services
├── database/             # Migrations, queries, schema issues
├── build/                # CI/CD, compilation, dependencies
├── testing/              # Test failures, flaky tests, mocking
└── runtime/              # Crashes, exceptions, deployment issues
```

## Naming Convention

`[short-description]-[component].md`

Examples:

- `n-plus-one-query-user-list.md`
- `jwt-expiry-refresh-token.md`
- `oauth-redirect-mismatch.md`

---

## Template

````markdown
# [Problem Title]

Brief one-line summary of the problem.

## Symptom

What you observed. Be specific and include exact text where possible.

- **Error message:** `[exact error text]`
- **Behavior:** [what happened vs. what was expected]
- **Context:** [when/where this occurred]

## Investigation

What you tried that didn't work (helps others avoid dead ends):

1. [First thing tried] — [why it didn't help]
2. [Second thing tried] — [what you learned]

## Root Cause

Technical explanation of what was actually wrong.

[Explain the underlying issue, not just the symptom. Include code references
where relevant, e.g., `app/services/user_service.rb:42`]

## Solution

What fixed it:

### Code Changes

```[language]
# Before (if helpful)
[old code]

# After
[new code]
```
````

### Configuration Changes

```
[config changes if applicable]
```

### Commands

```bash
[commands to run if applicable]
```

## Prevention

How to avoid this in future:

- [ ] [Pattern to follow]
- [ ] [Check to add to review process]
- [ ] [Test to write]

## Related

- **Work item:** [WORK-ITEM-ID if applicable]
- **PR:** [#number or link]
- **Similar issues:** [links to related solutions]
- **Documentation:** [relevant external docs]

## Metadata

| Field       | Value                     |
| ----------- | ------------------------- |
| Date        | YYYY-MM-DD                |
| Component   | [affected module/area]    |
| Severity    | [critical/moderate/minor] |
| Time to fix | [rough estimate]          |

```

---

## Tips for Good Solutions

**Do include:**

- Exact error messages (copy-paste)
- Specific file:line references
- Observable symptoms (what you saw)
- Failed attempts (helps others avoid dead ends)
- Code examples (before/after)
- Prevention guidance

**Avoid:**

- Vague descriptions ("something was wrong")
- Missing technical details
- Just code dumps without explanation
- No prevention guidance

## Building Patterns

After documenting 3+ similar issues, consider:

1. **Extracting a pattern** — Create `docs/solutions/patterns/[name].md`
2. **Updating conventions** — Add to project coding standards
3. **Creating a checklist** — Add to review process
4. **Updating templates** — Prevent the issue in new code
```
