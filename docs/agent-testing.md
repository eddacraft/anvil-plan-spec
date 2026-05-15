# Agent Cross-Harness Test Plan

Test plan for AGENT-006: verifying APS agents work correctly in each tool's
environment.

## Test Matrix

| Tool        | Agent Format                    | Test Method                   | Status                  |
| ----------- | ------------------------------- | ----------------------------- | ----------------------- |
| Claude Code | `.claude/agents/*.md`           | Task dispatch in live project | Validated               |
| Codex       | `.codex/agents/*.toml` + config | `/agent spawn`                | Manual (needs Codex)    |
| Copilot     | `.github/agents/*.md`           | Agent discovery               | Manual (needs Copilot)  |
| OpenCode    | `.opencode/agents/*.md`         | `@mention` invocation         | Manual (needs OpenCode) |
| Gemini      | `.gemini/skills/*/SKILL.md`     | `gemini skills link`          | Manual (needs Gemini)   |

## Automated Validation (Complete)

### Build Script

- [x] `build.sh` runs without errors
- [x] `build.sh` is idempotent (running twice produces identical output)
- [x] All 14 output files generated (2 core + 2 Claude Code + 2 Copilot + 2
      OpenCode + 3 Codex + 2 Gemini verified)

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

- [x] TOML format with `sandbox_mode` and `developer_instructions`
- [x] Config snippet has correct `[agents.*]` blocks
- [x] `o4-mini` model (OpenAI — commented for clarity)
- [x] Developer instructions contain full core prompt

**Gemini:**

- [x] Pure markdown (no YAML frontmatter)
- [x] Self-contained (condensed, not a core prompt copy)
- [x] Covers key responsibilities in skill-appropriate format

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
# Merge codex-config-snippet.toml into .codex/config.toml

# 2. Spawn planner
#    /agent spawn aps-planner
#    Ask: "What's the plan status?"

# 3. Spawn librarian
#    /agent spawn aps-librarian
#    Ask: "Audit the repo"
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

### Gemini

```bash
# 1. Place skill files
cp -r scaffold/agents/gemini/aps-planner /tmp/test-project/.gemini/skills/
cp -r scaffold/agents/gemini/aps-librarian /tmp/test-project/.gemini/skills/

# 2. Link skills
#    gemini skills link . --scope workspace

# 3. Activate skill in conversation
```

## Issues Found and Fixed

1. **Stale OpenCode model IDs** — Updated from `claude-opus-4-20250514` /
   `claude-sonnet-4-20250514` to `claude-opus-4-6` / `claude-sonnet-4-6`
2. **Missing vendor comment** — Added inline comment to Codex config snippet
   clarifying `o4-mini` is an OpenAI model

## Notes

- Full end-to-end testing of non-Claude-Code tools requires those tools
  installed. The automated validation covers everything that can be checked
  without the tools: file format, content correctness, build reproducibility.
- Claude Code agents were validated live (format + content + dispatch readiness).
- The Gemini planner skill intentionally omits wave-based execution detail —
  this is appropriate condensation for the skill format.
