<!-- markdownlint-disable MD048 -- the ~~~ fence below is the point of this fixture -->

# Feature With Fenced Example Items

| ID    | Owner | Status |
| ----- | ----- | ------ |
| FENCE | @test | Ready  |

## Purpose

Work-item headers inside fenced code blocks are documentation examples, not
real items (ISS-001). They must produce no findings (no E005, no W003 vouch)
and no phantom `next`/`graph` entries, and a fenced heading must not
terminate the enclosing item's content early.

## Work Items

### FENCE-001: Real item with a fenced fake header inside — Ready

- **Intent:** Prove a fenced example header is content, not a terminator.
- **Expected Outcome:** The fields below the fence still count for E005.

```markdown
### FAKE-999: This is an example, not a real work item

- **Intent:** If the parser sees this, it will demand fields or index FAKE-999.
- **Dependencies:** GHOST-123
```

- **Validation:** `aps lint` reports no findings for this file; `aps next`
  and `aps graph` list only FENCE-001 and FENCE-002.

### FENCE-002: Real item after a tilde fence — Ready

- **Intent:** Cover the `~~~` fence form as well as backticks.
- **Expected Outcome:** The fenced `## Work Items` lookalike below is inert.

~~~text
## Work Items

### TILDE-777: Another example-only header
~~~

- **Validation:** `aps lint` reports no findings for this file.
