# Reconciliation Report Format

Output a reconciliation report in this format:

```
## APS Reconciliation Report

### Status Changes (proposed)
- ITEM-NNN: [current status] -> [proposed status] (reason)

### Files to Add
- ITEM-NNN: add [file path] to Files field

### New Work Items (proposed)
- [MODULE]-NNN: [title] — [rationale]

### Unblocked
- ITEM-NNN: dependency [DEP-ID] now complete

### Validation Results
- ITEM-NNN: [pass/fail] — [command output summary]

### No Changes Needed
- [list items reviewed but unchanged]
```

After presenting the report, ask if the user wants to apply the proposed changes.

> APS reconciliation found: 2 items potentially complete (AUTH-001, AUTH-003),
> 1 new file to track. Want me to apply the updates?

## Background Reconciliation (Optional)

For large plans where reconciliation is slow, you may run it as a background
shell process. Start it with `nohup` or in a separate terminal session and
redirect output to a temp file, then read the results when the user asks:

```bash
nohup bash -c 'cd /path/to/project && <reconciliation-steps> > /tmp/aps-reconcile.log 2>&1' &
```

When done, read `/tmp/aps-reconcile.log` and present the report (same format
as above).
