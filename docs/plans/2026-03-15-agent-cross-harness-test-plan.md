# Agent Cross-Harness Test Plan

**Date:** 2026-03-15
**Status:** Validated for Claude Code in v0.3.0; other harnesses pending live
verification. See [plans/modules/agents.aps.md](../../plans/modules/agents.aps.md)
(AGENT-006) for the original work item.

Test plan for AGENT-006: verifying APS agents work correctly in each tool's
environment.

## Test Matrix

| Tool        | Agent Format                  | Test Method                   | Status                  |
| ----------- | ----------------------------- | ----------------------------- | ----------------------- |
| Claude Code | `.claude/agents/*.md`         | Task dispatch in live project | Validated               |
| Codex       | `.codex/agents/*.toml`        | Named delegation + `/agent`   | Manual (needs Codex)    |
| Copilot     | `.github/agents/*.md`         | Agent discovery               | Manual (needs Copilot)  |
| OpenCode    | `.opencode/agents/*.md`       | `@mention` invocation         | Manual (needs OpenCode) |
| Grok        | `.agents/skills/*/SKILL.md`   | Skill auto-discovery          | Manual (needs Grok)     |

## Automated Validation (Complete)

### Build Script

- [x] `build.sh` runs without errors
- [x] `build.sh` is idempotent (running twice produces identical output)
- [x] All 12 output files generated (3 each for Claude Code, Copilot, OpenCode,
      and Codex); Grok consumes shared skills without bespoke agent files

### Format Validation

**Claude Code:**

- [x] YAML frontmatter: `name`, `description`, `model`, `tools`
- [x] Model values use valid shorthand (`opus`, `sonnet`)
- [x] Tools list matches expected (planner: +Task, librarian: no Task)

**Copilot:**

- [x] YAML frontmatter: `name`, `description` only
- [x] No unsupported fields (model, tools)
- [x] Body identical to Claude Code variant

**OpenCode:**

- [x] YAML frontmatter: `description`, `mode`, `model`, `steps`, `tools`,
      `permission`
- [x] `mode: subagent` (not primary)
- [x] Model uses `provider/model-id` format (`anthropic/claude-opus-4-6`,
      `anthropic/claude-sonnet-4-6`)
- [x] No `name` field (filename-derived)
- [x] Permission maps set dangerous tools to `"ask"`

**Codex:**

- [x] TOML format with non-empty `name`, `description`, and
      `developer_instructions`
- [x] No obsolete registration snippet emitted under `.codex/`
- [x] `aps update` and repeated `aps setup codex` refresh existing roles and
      remove both historical snippet locations
- [x] Optional model and sandbox settings inherit from the parent session
- [x] Developer instructions contain full core prompt

**Grok:**

- [x] Shared `.agents/skills/aps-planning/` payload installed
- [x] No bespoke Grok agent files generated

### Content Validation

**Planner (all variants):**

- [x] Project init
- [x] Index/module/work-item creation
- [x] Status tracking
- [x] Work item execution
- [x] Wave-based parallel coordination
- [x] Action plan support
- [x] References `plans/` paths (D-017 compliance)
- [x] Does not duplicate SKILL.md content

**Librarian (all variants):**

- [x] Archiving completed modules
- [x] Orphan detection
- [x] Cross-reference maintenance
- [x] Stale doc flagging
- [x] References `plans/` paths (D-017 compliance)

## Manual Test Procedures

### Claude Code

```bash
# 1. Copy agents to test project
cp scaffold/agents/claude-code/aps-planner.md /tmp/test-project/.claude/agents/
cp scaffold/agents/claude-code/aps-librarian.md /tmp/test-project/.claude/agents/

# 2. Dispatch planner via Task tool
#    Ask: "What's the plan status?"
#    Expect: Agent reads plans/, reports module statuses

# 3. Dispatch librarian
#    Ask: "Audit the repo for orphaned files"
#    Expect: Agent scans plans/, reports findings
```

### Codex

```bash
# 1. Place agent files
cp scaffold/agents/codex/aps-planner.toml /tmp/test-project/.codex/agents/
cp scaffold/agents/codex/aps-librarian.toml /tmp/test-project/.codex/agents/

# 2. Ask Codex: "Use the aps-planner agent to report the plan status."
#    Use /agent to inspect or switch to the spawned thread.

# 3. Ask Codex: "Use the aps-librarian agent to audit the repo."
```

### Copilot

```bash
# 1. Place agent files
cp scaffold/agents/copilot/aps-planner.md /tmp/test-project/.github/agents/
cp scaffold/agents/copilot/aps-librarian.md /tmp/test-project/.github/agents/

# 2. In Copilot Chat, agents should appear as available
# 3. Invoke @aps-planner and @aps-librarian
```

### OpenCode

```bash
# 1. Place agent files
cp scaffold/agents/opencode/aps-planner.md /tmp/test-project/.opencode/agents/
cp scaffold/agents/opencode/aps-librarian.md /tmp/test-project/.opencode/agents/

# 2. Switch to subagent via Tab or @aps-planner
# 3. Ask for plan status
```

### Grok

```bash
# 1. Install the shared planning skill
APS_LOCAL="$PWD" ./bin/aps init /tmp/test-project --tools grok

# 2. Ask Grok to report APS plan status and confirm skill auto-discovery
```

## Issues Found and Fixed

1. **Stale OpenCode model IDs** — Updated from `claude-opus-4-20250514` /
   `claude-sonnet-4-20250514` to `claude-opus-4-6` / `claude-sonnet-4-6`
2. **Stale Codex role schema** — Added standalone role identity fields and
   removed the obsolete `.codex/config.toml` registration snippet (AGENT-008)

## Notes

- Full end-to-end testing of non-Claude-Code tools requires those tools
  installed. The automated validation covers everything that can be checked
  without the tools: file format, content correctness, build reproducibility.
- Claude Code agents were validated live (format + content + dispatch readiness).
- Grok uses the shared planning skill and has no bespoke agent variant.
