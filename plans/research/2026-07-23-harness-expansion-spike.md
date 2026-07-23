# Harness-Expansion Spike: Candidates vs. the D-040 Native-Discovery Gate

> Research conducted: 2026-07-23
> Status: Complete
> Context: CLI-005 of the [cli-redesign](../modules/cli-redesign.aps.md) module.
> Decides which additional AI-coding harnesses APS should support in
> init/setup/wizard, by testing candidates against the D-040 native-discovery
> gate (the criterion that let Grok in for zero bespoke asset cost).

## Executive Summary

The gate that made **Grok** a zero-asset add (AGENT module, D-040) is: a harness
must **both** (a) natively read the `AGENTS.md` family **and** (b) auto-discover
skills at the shared `.agents/skills/<name>/SKILL.md` path with no per-tool
config. Half (b) is the discriminator — many tools read `AGENTS.md`, far fewer
scan the *shared* skills path (most scan only their own private dir).

Testing the named candidates (Antigravity, Hermes, OpenClaw) plus a sweep of the
broader ecosystem against that gate, with primary-source verification:

**Six harnesses clear the full bar as zero-asset adds** — **Antigravity, Amp,
Gemini CLI, Windsurf, Roo Code** (all vendor-documented, high confidence) and
**OpenClaw** (verified, recommend a hands-on confirm). Each reuses the exact
`.agents/skills/aps-planning/` payload APS already emits for Grok/Codex, untouched.

**Two are GO-WITH-ASSETS** (not zero-asset): **Cursor** (reads `AGENTS.md` but
scans only `.cursor/.claude/.codex` skills dirs — low-cost mirror/symlink) and
**Hermes** (global `~/.hermes/skills/` path + severe name collision).

**The AGENTS.md-only crowd is NO-GO for now** (Aider, Zed, Devin, Jules, Warp,
Continue, Factory, Junie, goose): they read `AGENTS.md` but lack shared
`.agents/skills/` discovery. Revisit per-tool as the convention spreads.

**Recommended D-045 set (proposed — needs approval):** add the five
high-confidence zero-asset harnesses (**Antigravity, Amp, Gemini CLI, Windsurf,
Roo Code**) in CLI-006; add **OpenClaw** after a 5-minute hands-on confirm; take
**Cursor** as a separate low-cost GO-WITH-ASSETS decision (its user base may
outweigh several zero-asset tools); deprioritise **Hermes**.

## The gate (D-040 native-discovery)

From `agents.aps.md:39`, the Grok row is the reference bar:

> **Grok** — none (auto-discovery) — Reads `AGENTS.md` family; discovers
> `.agents/skills/<name>/SKILL.md` and `.claude/` assets natively (D-040)

So a **zero-asset add** must satisfy **both**:

- **(a) AGENTS.md family** — natively auto-loads a root instructions file
  (`AGENTS.md`, the cross-tool convention at [agents.md](https://agents.md)).
- **(b) Shared skills discovery** — auto-discovers `.agents/skills/<name>/SKILL.md`
  (the [Agent Skills / agentskills.io](https://agentskills.io/) convention) with
  no per-tool config, so APS's existing skill payload is picked up untouched.

A harness meeting (a) but not (b) is **GO-WITH-ASSETS** — supportable, but APS
must ship a bespoke per-tool asset (mirror the skill into the tool's private dir).
A harness meeting neither, or unverifiable, is **NO-GO / defer**.

## Method & confidence

Four parallel research streams (three named candidates + one ecosystem sweep),
each required to cite primary/vendor sources and flag name collisions and thin
evidence. Two load-bearing facts were then spot-checked directly by the author:
Antigravity's skills doc (`.agents/skills` is the documented **default** path)
and the `agents.md` registry (confirms Gemini CLI, Windsurf, Amp, RooCode, Cursor
as adopters). This spike is **low-confidence by design** — the binding
verification for each harness is a real `aps init --tools <x>` smoke test during
CLI-006 (as CLI-004 did for Grok).

## Named candidates

### Antigravity (Google) — GO (zero-asset), high confidence

Google's agent-first development platform (VS Code fork + CLI), announced
2025-11-18 with Gemini 3, in public preview on Mac/Linux/Windows
([antigravity.google](https://antigravity.google/),
[Google Developers Blog](https://developers.googleblog.com/build-with-google-antigravity-our-new-agentic-development-platform/)).

- **(a)** Reads `AGENTS.md` natively since ~March 2026
  ([changelog](https://antigravity.google/changelog)); merges with `GEMINI.md`,
  **`GEMINI.md` winning on conflicts** (minor nuance; pre-March write-ups are stale).
- **(b)** Auto-discovers `.agents/skills/<folder>/SKILL.md` as the **default**
  workspace path ([Antigravity Skills docs](https://antigravity.google/docs/skills),
  author-verified) — the exact shape APS emits.

Strategically the strongest add (Google's flagship agentic platform). Target the
`.agents/skills` default (not legacy `.agent/skills`).

### Hermes (Nous Research) — GO-WITH-ASSETS, medium confidence

Real, installable open-source agent (CLI + TUI + desktop) that executes code
([github.com/NousResearch/hermes-agent](https://github.com/NousResearch/hermes-agent)),
but positioned as a *general* self-improving personal agent, not an IDE-scoped
coding copilot.

- **(a)** Reads the `AGENTS.md`/`CLAUDE.md` family via a `workdir`; unqualified
  workspace-root auto-load is **thinly documented**.
- **(b)** Auto-discovers agentskills.io-standard `SKILL.md` — but from a **global
  `~/.hermes/skills/<category>/<name>/`** path, **not** repo-local `.agents/skills/`.
  APS's in-repo skills would need importing via the Skills Hub → a bespoke asset.

**Severe name collision** (Hermes LLM series, Meta's JS engine, SEO spam sites).
Deprioritise; a discovery/confirmation step is needed before any commitment.

### OpenClaw — GO (zero-asset), medium confidence (confirm first)

A newer open-source personal-AI-agent / self-hosted gateway ("the lobster way")
at [github.com/openclaw/openclaw](https://github.com/openclaw/openclaw) /
[docs.openclaw.ai](https://docs.openclaw.ai/) — **distinct from** the Captain Claw
game re-implementation of the same name
([freeCodeCamp](https://www.freecodecamp.org/news/how-to-build-and-secure-a-personal-ai-agent-with-openclaw/)).

- **(a)** Auto-injects a fixed bootstrap set including `AGENTS.md`
  ([config-agents docs](https://docs.openclaw.ai/gateway/config-agents)).
- **(b)** Discovers any `SKILL.md` under `<workspace>/.agents/skills` and
  `~/.agents/skills` (up to 6 levels), first-class, no per-skill config
  ([skills docs](https://docs.openclaw.ai/tools/skills)).

Framed more as a broad assistant gateway than a code-first CLI, and some scraped
popularity metrics looked inflated/hallucinated. Both gate conditions are met,
but recommend a 5-minute hands-on install to confirm before committing.

## Ecosystem sweep (beyond the supported set)

`.agents/skills/` is a real cross-tool convention; the common failure mode is
half (b) — tools that read `AGENTS.md` but scan only a private skills dir.

| Harness | Maker | (a) AGENTS.md | (b) `.agents/skills/` discovery | Verdict |
| --- | --- | --- | --- | --- |
| **Antigravity** | Google | Yes (Mar 2026) | Yes — `.agents/skills` default | **GO** |
| **Amp** | Sourcegraph | Yes — cwd+parents+global | Yes — `.agents/skills/` + `~/.config/agents/skills/` | **GO** |
| **Gemini CLI** | Google | Yes | Yes — `.agents/skills/` alias, precedence over `.gemini/skills/` | **GO** |
| **Windsurf (Cascade)** | Cognition | Yes | Yes — `.agents/skills/` + `~/.agents/skills/` | **GO** |
| **Roo Code** | Roo Code Inc | Yes (auto-load is a toggle) | Yes — `.agents/skills/` (project + `~`) | **GO** (toggle caveat) |
| **OpenClaw** | openclaw | Yes (bootstrap inject) | Yes — `.agents/skills` first-class | **GO** (confirm) |
| **Cursor** | Anysphere | Yes | **No** — only `.cursor/.claude/.codex` | **GO-WITH-ASSETS** (low) |
| **Hermes** | Nous Research | Yes (family) | Global `~/.hermes/skills/` only | **GO-WITH-ASSETS** |
| Aider, Zed, Devin, Jules, Warp, Continue, Factory, Junie, goose | various | Yes (read AGENTS.md) | No / per-tool / unknown | **NO-GO / defer** |

Sources for the GO rows are vendor docs: [Amp manual](https://ampcode.com/manual),
[Gemini CLI skills](https://github.com/google-gemini/gemini-cli/blob/main/docs/cli/skills.md),
[Windsurf Cascade skills](https://docs.windsurf.com/windsurf/cascade/skills),
[Roo Code skills](https://roocodeinc.github.io/Roo-Code/features/skills). Cursor's
per-tool skills discovery per [skill-discovery analysis](https://agenticthinking.ai/blog/skill-discovery/)
(one aggregator claimed `.agents/skills` support; the detailed source contradicts
it — treat as GO-WITH-ASSETS until Cursor's own docs confirm).

## The real cost of a "zero-asset" add

"Zero-asset" means no bespoke **scaffold assets** — the harness reuses the shared
`.agents/skills/aps-planning/` payload Grok/Codex already install. It does **not**
mean zero effort. Each new harness still costs, under D-039 three-way lockstep:

- a tool key in `cli/src/config.rs` + the bash and PowerShell tool lists,
- a wizard/init entry and install dispatch (install the shared skill),
- docs (README supported-set, installation.md), and
- parity fixtures + tests, plus a real `aps init --tools <x>` smoke test.

So the gate bounds **asset** cost, not **plumbing** cost. Recommend a curated
set, not "everything that passes" — five well-chosen adds is meaningful lockstep
work in CLI-006.

## Recommendation — proposed D-045 set (needs approval)

1. **Approve as zero-asset adds (CLI-006):** **Antigravity, Amp, Gemini CLI,
   Windsurf, Roo Code.** High-confidence, vendor-documented, reuse the existing
   skill payload untouched. (Roo Code: document that `AGENTS.md` auto-load is a
   user-enabled toggle.)
2. **Approve pending a 5-minute hands-on confirm:** **OpenClaw.**
3. **Separate GO-WITH-ASSETS decision (not zero-asset):** **Cursor** — reads
   `AGENTS.md` but needs the skill mirrored into `.cursor/skills/` (or rely on its
   opportunistic `.claude/skills/` scan). Low asset cost, and Cursor's user base
   likely outweighs several Tier-1 tools — worth a dedicated follow-up.
4. **Deprioritise:** **Hermes** — global skills path + severe name collision;
   discovery step first.
5. **Not now:** the `AGENTS.md`-only crowd (Aider, Zed, Devin, Jules, Warp,
   Continue, Factory, Junie, goose). Revisit per-tool as shared `.agents/skills/`
   discovery spreads.

## Open confirm-steps (before CLI-006 commits each)

- **OpenClaw:** 5-minute install; confirm `.agents/skills/` pickup and code-harness path.
- **Cursor:** confirm from Cursor's own docs whether skills discovery ever reads
  `.agents/skills/`; if not, decide mirror-dir vs `.claude/skills/` fallback.
- **Roo Code:** confirm the `AGENTS.md` auto-load toggle default and document it.
- **All:** the binding check is a real `aps init --tools <x>` smoke test in all
  three CLIs during CLI-006 (Grok precedent, CLI-004).

## Validation

- This evaluation is linked from the [cli-redesign](../modules/cli-redesign.aps.md)
  module (CLI-005).
- The D-045 decision records the approved set **after** sign-off; until then the
  recommendation above is a proposal, not an accepted decision.
