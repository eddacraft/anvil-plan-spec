# Prompts Module

| ID      | Owner  | Priority | Status |
| ------- | ------ | -------- | ------ |
| PROMPTS | @aneki | medium   | Ready  |

**Last reviewed:** 2026-06-27

## Purpose

Maintain APS prompting entry points for generic agents and tool-specific
harnesses without drifting away from the shared APS lifecycle.

## In Scope

- Generic index, module, work item, and actions prompts
- OpenCode-specific prompt variants
- Claude Code orchestration prompts
- Guidance for when tool-specific variants should exist versus linking back to
  generic prompts
- Stub prompts for harnesses that defer wholesale to the generic prompts +
  `AGENTS.md` (D-006)

## Out of Scope

- Full agent packaging, which belongs to AGENT
- Prompt marketplace distribution

## Interfaces

**Depends on:**

- AGENT (agents) — stubs and variants defer to `AGENTS.md` / `aps-rules.md`
- SPEC (spec) — prompts must teach the canonical status vocabulary (D-026)

**Exposes:**

- `docs/ai/prompting/*.prompt.md`
- `docs/ai/prompting/opencode/*.prompt.md`
- `docs/ai/prompting/claudecode/*.prompt.md`
- `docs/ai/prompting/README.md` — variant-vs-stub policy

## Current Coverage

| Harness     | Prompt assets                                              | Shape          |
| ----------- | ---------------------------------------------------------- | -------------- |
| generic     | index, module, work-item, actions                          | Full set       |
| OpenCode    | index, module, work-item, actions                          | Full variant   |
| Claude Code | agent-assignment, sync-status, tasks-from-module, wave-planning | Orchestration deltas |
| Copilot     | —                                                          | Uncovered      |
| Codex       | —                                                          | Uncovered      |
| Gemini      | —                                                          | Uncovered      |

The five harnesses APS targets are fixed by D-013/D-019 (Claude Code, Copilot,
OpenCode, Codex, Gemini). Three have no prompt entry point today, which is the
gap D-006 calls out.

## Decisions

- **D-006:** Tool-specific prompt variants — _decided: yes; each harness needs
  either a variant or a stub pointing at the generic prompts + `AGENTS.md`._
  (Roadmap D-006.) This module implements that decision.
- **D-001 (module):** Variant vs stub per harness — **proposed.** A harness gets
  a full variant only when its workflow diverges from the generic lifecycle
  (OpenCode flow control, Claude Code orchestration/Tasks). Otherwise it gets a
  one-screen stub that links to the generic prompt and `AGENTS.md`. Resolved by
  PROMPTS-002.

## Work Items

### PROMPTS-001: Normalize existing prompt variants — Ready

- **Intent:** Keep the OpenCode and Claude Code variants consistent with the
  generic APS rules so tool prompts state only their tool-specific deltas and
  otherwise defer to shared APS concepts.
- **Expected Outcome:** Each variant under `opencode/` and `claudecode/` opens
  with a one-line "defers to generic; differences below" pointer, carries no
  duplicated stale lifecycle rules, and uses the canonical status vocabulary
  (`Draft / Ready / In Progress / Complete / Blocked`, D-026).
- **Validation:** Diff each variant against its generic counterpart — only
  intentional deltas remain; `markdownlint docs/ai/prompting/` passes.
- **Confidence:** high
- **Dependencies:** None
- **Files:** docs/ai/prompting/opencode/, docs/ai/prompting/claudecode/

### PROMPTS-002: Document variant-vs-stub policy — Ready

- **Intent:** Make the 4th In-Scope rule concrete so future harnesses have a
  rule for whether to ship a full variant or a stub (resolves D-038).
- **Expected Outcome:** `docs/ai/prompting/README.md` states the decision
  criteria (full variant only when the harness workflow diverges; stub
  otherwise), lists current coverage, and links the generic prompts + `AGENTS.md`
  as the source of truth. Module D-001 marked decided.
- **Validation:** README review against the coverage table; markdownlint passes;
  a reader can decide variant-vs-stub for a new harness from the doc alone.
- **Confidence:** high
- **Dependencies:** None
- **Files:** docs/ai/prompting/README.md, plans/modules/prompts.aps.md,
  plans/index.aps.md

### PROMPTS-003: Add stub prompts for uncovered harnesses — Ready

- **Intent:** Close the D-006 gap for Copilot, Codex, and Gemini, which have no
  prompt entry point today.
- **Expected Outcome:** A stub prompt per uncovered harness (e.g.
  `docs/ai/prompting/{copilot,codex,gemini}/README.md`) that points at the
  generic prompts and the harness's instruction file (per D-019: `.github/agents`
  for Copilot, `.codex/config.toml` for Codex, `GEMINI.md` for Gemini), adding a
  full variant only where PROMPTS-002's policy says one is warranted.
- **Validation:** Each targeted harness resolves from prompt entry point →
  generic prompt → `AGENTS.md` with no dead links; markdownlint passes.
- **Confidence:** medium
- **Dependencies:** PROMPTS-002, AGENT
- **Files:** docs/ai/prompting/copilot/, docs/ai/prompting/codex/,
  docs/ai/prompting/gemini/

## Ready Checklist

- [x] Purpose and scope are clear
- [x] Dependencies identified (AGENT, SPEC)
- [x] Work items defined with validation
- [ ] D-001 (module) resolved (variant-vs-stub policy — lands with PROMPTS-002)

## Notes

Promoted Draft → Ready on 2026-06-27. PROMPTS-001 and PROMPTS-002 have no
dependencies and can start immediately; PROMPTS-003 follows once the
variant-vs-stub policy (D-038) is written.
