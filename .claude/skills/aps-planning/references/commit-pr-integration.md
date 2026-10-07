# Commit and PR Integration

When APS context is active, enrich commits and PRs with work item references.
This is passive -- only add references when the file-to-item map produces
matches.

## Commit Messages

When creating a commit, look up the staged files against the file-to-item map.
If any staged files match one or more work items:

1. Resolve the list of matching work item IDs (deduplicated, sorted)
2. Add an `APS:` trailer to the commit message footer

Format:

```text
feat(auth): add login endpoint

Implement JWT-based login with bcrypt password hashing.

Fixes #42

APS: AUTH-001
```

Multiple items:

```text
APS: AUTH-001, AUTH-003, DB-002
```

Rules:

- Only add the trailer when the file-to-item map produces matches
- If no staged files match any work item, omit the trailer entirely
- Do not replace or interfere with the scope -- scope remains the code area
- The `APS:` trailer goes on its own line in the footer block
- If more than 5 items match, list the primary and note "and N others"

## Pull Requests

When creating a PR and APS context is active:

1. Collect all work item IDs from commits in the PR (scan for `APS:` trailers)
   or look up changed files against the file-to-item map
2. Add an **APS Work Items** section to the PR body, after Summary and before
   Test Plan:

```markdown
## APS Work Items

- **AUTH-001**: Add login endpoint (In Progress)
- **AUTH-003**: Add password hashing (Ready)
```

Rules:

- Include item ID, title, and current status
- If only one item, still use the section for consistency
- Do not add APS references to the PR title -- keep titles concise
- If no work items match, omit the section entirely
