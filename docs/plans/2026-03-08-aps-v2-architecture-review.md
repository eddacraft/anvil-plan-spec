# APS v2 Architecture Review: "What Would We Do Differently?"

| Field | Value |
|-------|-------|
| Status | Draft |
| Created | 2026-03-08 |
| Author | Architecture review after months of production use |

## Executive Summary

APS is genuinely good. The core insight — portable markdown specs as a trust
layer between humans and AI — is the right foundation. The hierarchy
(Index > Module > Work Item > Action Plan), the "specs describe intent not
implementation" philosophy, and the compound engineering lifecycle all hold up.

But after months of using it in anger, patterns emerge. Some are friction.
Some are missing capabilities. Some are structural decisions that made sense
at v0.1 but now constrain growth.

This review is structured as: **what's working**, **where we hit walls**,
**what a v2 would look like**, and **what can be done incrementally**.

---

## Part 1: What's Working (Don't Touch This)

### The Core Model

The four-layer hierarchy is sound and should be preserved:

```
Index (intent) → Module (bounded scope) → Work Item (authority) → Action Plan (execution)
```

This maps naturally to how teams actually think: initiative → area → task → steps.
Most competitors either flatten this (losing structure) or over-nest it (losing
clarity). APS hits the sweet spot.

### Plain Markdown

The "just files" approach is APS's single biggest competitive advantage over
BMAD, SpecKit, and every other framework in this space. No YAML frontmatter
gymnastics. No custom DSLs. No runtime. No database. Any editor, any AI, any
workflow. This should never change.

### The Trust Layer Concept

"Work items authorise execution" is philosophically correct and practically
useful. The explicit Ready gate prevents AI from going rogue. Keep this.

### The Hooks System

The PreToolUse/PostToolUse/Stop/SessionStart hook architecture is clever
and genuinely useful. The enforce-plan-update stop hook (blocking session end
if code changed but plans didn't) is the kind of thing that sounds annoying
but actually saves you. This is a differentiator.

### Lean Checkpoints

The "12 words max, no implementation detail" rule for action checkpoints is
one of the best ideas in APS. It prevents the spec from becoming a tutorial
that rots immediately. Keep this iron rule.

---

## Part 2: Where We Hit Walls

### Problem 1: The Ceremony Tax

**Symptom:** For a 2-hour feature, you spend 30 minutes setting up APS files.

The path from "I want to build X" to "I have a work item I can execute" is:

1. Create/update `index.aps.md`
2. Create a module file with the right `NN-name.aps.md` naming
3. Fill in Purpose, In Scope, Out of Scope, Interfaces, Constraints
4. Fill in the metadata table (ID, Owner, Priority, Status)
5. Write work items with Intent, Expected Outcome, Validation
6. Set status to Ready
7. Optionally create an action plan

That's 7 steps before you write a single line of code. For a large initiative,
this is appropriate. For "add a dark mode toggle", it's overkill even with the
Simple template.

**The quickstart template tries to solve this** but it's disconnected from the
rest of the system. It doesn't lint. It doesn't have an ID. It's a stepping
stone, not a first-class citizen.

**What competitors do better:** BMAD lets you start with a user story and
progressively elaborate. You don't front-load structure — it emerges as
complexity demands it.

### Problem 2: The Document Sprawl

**Symptom:** After a few months, you have 15+ files across plans/, execution/,
designs/, and decisions/. Navigation becomes archaeology.

Current file structure for a medium project:

```
plans/
├── aps-rules.md
├── index.aps.md
├── issues.md
├── modules/
│   ├── 01-core.aps.md
│   ├── 02-auth.aps.md
│   ├── 03-payments.aps.md
│   ├── 04-ui.aps.md
│   └── 05-notifications.aps.md
├── execution/
│   ├── CORE-001.actions.md
│   ├── CORE-002.actions.md
│   ├── AUTH-001.actions.md
│   ├── AUTH-002.actions.md
│   ├── AUTH-003.actions.md
│   └── PAY-001.actions.md
└── decisions/
    ├── 001-use-jwt.md
    └── 002-event-bus.md
designs/
├── 2025-01-05-auth.design.md
└── 2025-02-10-payments.design.md
```

That's 18 files. An AI agent starting a session has to read at least
aps-rules.md + index.aps.md + the relevant module + possibly the action plan
before it can do anything. That's 4+ file reads minimum, eating into context
window before any work begins.

**The real cost:** Every new document type (designs, issues, solutions,
decisions) adds another place to look and another thing to keep in sync. The
"update as you go" philosophy is aspirational — in practice, specs drift.

### Problem 3: Status Lives in the Wrong Place

**Symptom:** To know what's done, you have to read every module file and parse
markdown tables.

Status is scattered across:
- Module metadata tables (module-level status)
- Work item prose (per-item status via `- **Status:** Complete`)
- Action plan checkboxes
- Index module table
- Issues tracker

There's no single source of truth. The init-session.sh script tries to
aggregate this but it's fragile — it greps for patterns in markdown, which
breaks if formatting varies slightly.

**What this means:** The `/plan-status` command rebuilds state by re-reading
everything every time. For 15+ files, this is slow and error-prone.

### Problem 4: The AI Agent Getting-Started Problem

**Symptom:** A new AI agent needs to ingest 3-5 documents before it
understands APS, then 2-3 more before it understands this project.

The installed files an agent needs to potentially process:
- `aps-planning/SKILL.md` (250 lines)
- `aps-planning/reference.md` (149 lines)
- `aps-planning/examples.md` (185 lines)
- `plans/aps-rules.md` (325 lines)
- `plans/index.aps.md` (varies)
- The relevant module (varies)

That's 900+ lines of APS documentation before the agent reads a single line
of project code. SKILL.md and aps-rules.md overlap significantly. reference.md
is a condensed version of what's in SKILL.md. There's redundancy everywhere.

**The 5-Operation Rule** in SKILL.md ("pause every 5 tool operations and ask
yourself...") is a good idea that's unenforceable. It relies on the AI
self-policing. The hooks partially solve this, but the rule itself is just
aspirational text.

### Problem 5: Template Proliferation

**Symptom:** 9 templates, and users aren't sure which one to use.

Templates: quickstart, simple, module, index, index-expanded, index-monorepo,
actions, design, solution. Plus the scaffold copies hidden template files into
modules/ and execution/.

The getting-started guide has a decision tree. But AI agents (the primary
consumers) don't navigate decision trees well — they need a clear signal based
on the inputs they have.

**The overlap:** `quickstart` vs `simple` is confusing. quickstart is
"try APS in 5 minutes" but it doesn't lint. simple is the "real" small
template. Why do both exist?

### Problem 6: Terminology Drift

The codebase has terminology inconsistencies that create confusion:

- "Steps" vs "Actions" — the rename from v0.1 didn't fully propagate.
  `scaffold/plans/execution/.steps.template.md` still says "steps". Some docs
  say "Steps files" (workflow.md:236). The aps-rules.md still has a section
  called "Steps: The Lean Rule".
- "Task" vs "Work Item" — aps-rules.md uses "Task" extensively in headers and
  tables. The rest of the system uses "Work Item". These should be the same
  thing but the inconsistency suggests they might not be.
- "Scope" vs "ID" — Changelog mentions renaming SCOPE to ID, but some mental
  models still reference scope.

### Problem 7: The Validation Gap

**Symptom:** The linter validates structure but not semantics.

What `aps lint` checks:
- Required sections exist (Purpose, Work Items)
- Metadata table present
- Work item has Intent, Expected Outcome, Validation
- ID format (PREFIX-NNN)

What it doesn't check:
- Cross-file consistency (does the index reference modules that exist?)
- Dependency validity (does AUTH-002's dependency on AUTH-001 exist?)
- Status consistency (module says Ready but has no work items)
- Orphaned action plans (references work items that were deleted)

The Librarian agent fills some of these gaps, but it's an agent call (costs
tokens) not a fast local check.

### Problem 8: No Progressive Disclosure

**Symptom:** Small projects carry the same cognitive overhead as large ones.

A solo dev adding a single feature encounters the full APS vocabulary:
Index, Module, Work Item, Action Plan, Design, Issues, Decisions, Solutions,
aps-rules.md, SKILL.md. There's no way to "use APS lite" that's officially
supported and well-integrated.

The quickstart template is the attempt at this, but it's explicitly positioned
as a stepping stone ("When this grows, split into module.template.md files"),
not a legitimate end state.

### Problem 9: Dual Platform Tax

**Symptom:** Every script exists in both Bash and PowerShell, doubling
maintenance surface.

Files that have .sh and .ps1/.psm1 pairs:
- bin/aps + bin/aps.ps1
- lib/lint.sh + lib/Lint.psm1
- lib/output.sh + lib/Output.psm1
- lib/scaffold.sh + lib/Scaffold.psm1
- All 6 hook scripts (12 files total)
- All 6 rule files (12 files total)
- install + install.ps1
- update + update.ps1

That's ~30 duplicated files. Every bug fix, every feature addition, every
rule change needs to be made twice. This is not sustainable at the current
team size.

---

## Part 3: What a v2 Would Look Like

### Design Principle: Progressive Formality

APS v2 should let you start with almost nothing and add structure only when
you need it. The system should feel lightweight for small things and powerful
for large things — without requiring you to decide upfront which one you're
doing.

### Change 1: Single-File Mode as First-Class

Instead of quickstart being a stepping stone, make it THE entry point. A
single `.aps.md` file should be valid, lintable, and fully functional:

```markdown
# Add Dark Mode

## Problem
Users want to reduce eye strain.

## Work Items

### 001: Add theme context
- **Outcome:** ThemeProvider wraps app, exposes toggle
- **Test:** `npm test -- theme.test.tsx`

### 002: Add toggle to settings
- **Outcome:** Settings page has working theme toggle
- **Test:** Manual verification
- **Depends on:** 001
```

No metadata table required. No ID prefix required. No status field. The
linter infers:
- No metadata table → single-file mode, no module ID needed
- Work item IDs without prefix → auto-assigned sequential
- No status → Draft by default
- File location doesn't matter — can be in plans/, root, or anywhere

**When you outgrow single-file:** Run `aps promote feature.aps.md` and it
scaffolds out to the full Index + Module structure, preserving your work items.

### Change 2: Merge the Agent Documentation

Consolidate SKILL.md, reference.md, examples.md, and aps-rules.md into a
single document: `PLANNING.md` (or keep `aps-rules.md` as the sole file).

One file. ~300 lines max. Contains everything an AI agent needs. No chasing
references between 4 documents. The agent reads one file and knows APS.

The current split exists because of historical layering (skill vs rules vs
reference), not because it serves the reader. Kill the indirection.

### Change 3: Status as Data, Not Prose

Replace scattered status-in-markdown with a lightweight status sidecar:

```
plans/.status.json
```

```json
{
  "modules": {
    "AUTH": { "status": "ready", "updated": "2025-01-10" },
    "PAY": { "status": "draft", "updated": "2025-01-08" }
  },
  "items": {
    "AUTH-001": { "status": "complete", "completed": "2025-01-12" },
    "AUTH-002": { "status": "in_progress", "started": "2025-01-13" },
    "PAY-001": { "status": "draft" }
  }
}
```

**Benefits:**
- `aps status` is instant — read one file
- No markdown parsing required
- Git-diffable (JSON changes are clear in PRs)
- Hooks can update it automatically
- Status is authoritative in one place, not scattered across 10 files

**The specs stay as-is.** Modules still have their metadata tables for human
reading, but the machine reads `.status.json`. When there's a conflict, the
JSON wins (or the linter flags the discrepancy).

### Change 4: Flatten the Hierarchy by Default

For most projects, the Index is unnecessary overhead. The default structure
should be:

```
plans/
├── rules.md              # Consolidated agent guidance
├── .status.json           # Machine-readable status
├── auth.aps.md            # Module (flat, no subdirectory needed)
├── payments.aps.md        # Module
└── execution/             # Action plans (only when needed)
    └── AUTH-001.actions.md
```

**No modules/ subdirectory for < 6 modules.** The numbered prefix
(`01-auth.aps.md`) can go — it's ceremony that adds friction. If you want
ordering, use the Index table.

**The Index becomes optional.** For a single module or 2-3 small modules,
skip it. The linter doesn't require it. When you need one (5+ modules,
milestones, cross-cutting concerns), create it.

### Change 5: Kill the Dual Platform Approach

Pick one implementation language. Recommendations in order:

1. **Node.js (TypeScript)** — Already have package.json. npm ecosystem.
   Cross-platform by default. Better for building a real CLI later.
2. **Go** — Single binary, cross-platform, fast. Best for distribution but
   higher investment.
3. **Bash only** — Accept that Windows users use WSL. Most AI dev tools
   assume unix anyway.

The PowerShell port is a maintenance burden that slows everything down.
Windows users who are doing AI-assisted development almost certainly have
WSL or Git Bash.

### Change 6: Smarter Linting

The linter should do cross-file analysis:

```bash
aps lint                    # Structure + cross-reference checks
aps lint --strict           # + semantic checks
aps lint --fix              # Auto-fix what's possible
```

New rules:
- **Cross-ref validation:** Index references modules that exist, dependencies
  reference valid IDs
- **Status sync:** .status.json matches metadata tables
- **Orphan detection:** Action plans reference existing work items
- **Staleness warning:** Modules untouched for >30 days with In Progress items
- **Auto-fix:** Add missing metadata tables, normalize ID formats, add
  `.status.json` entries for new modules

### Change 7: The `aps` CLI as the Central Interface

Expand the CLI from a linter to a project management tool:

```bash
aps init                     # Scaffold (existing)
aps new "Add dark mode"      # Create a new single-file spec
aps new --module auth        # Create a module spec
aps promote feature.aps.md   # Upgrade single-file to module
aps status                   # Read .status.json, pretty print
aps next                     # What's the next Ready item?
aps start AUTH-001           # Mark as In Progress
aps done AUTH-001            # Mark as Complete, run validation
aps lint                     # Validate everything
aps lint --fix               # Auto-fix structural issues
```

Each command is simple, fast, and does one thing. The AI agent can use these
directly instead of parsing markdown.

### Change 8: Simplify Templates to Two

| Template | When |
|----------|------|
| **spec** | Default. Works for features, modules, everything. |
| **action-plan** | Breaking down complex work items. |

That's it. One spec template with progressive sections. The metadata table,
Interfaces, Constraints, etc. are all optional. The linter only requires
Problem/Purpose + Work Items.

Design documents, issues trackers, and solution docs are useful but they're
not core APS — they're adjacent practices. Document them in the guide but
don't make them first-class template citizens that clutter the template list.

### Change 9: Built-in Context Compression

Instead of relying on the AI to "read aps-rules.md first", bake the rules
into the CLI output. When the session starts:

```bash
aps context                  # Output everything the AI needs in one shot
```

This command:
1. Outputs the condensed APS rules (50 lines, not 300)
2. Shows current status (from .status.json)
3. Shows the relevant module spec for any In Progress items
4. Suggests what to work on next

One command. One read. The AI is fully oriented. No "read this file, then
that file, then that file."

### Change 10: Events, Not Just Hooks

The current hooks are good but limited to Claude Code. Generalize them:

```bash
aps hook session-start       # Run by any tool's session start
aps hook pre-edit            # Run before code changes
aps hook session-end         # Run at session end
```

The hooks become tool-agnostic CLI commands. Claude Code's hooks call these.
Cursor's .cursor-rules calls these. Any tool's lifecycle hooks call these.
The APS behavior (goal drift prevention, status enforcement) travels with
the project, not the tool.

---

## Part 4: What Can Be Done Incrementally (Without a Rewrite)

These are ordered by impact/effort ratio. High impact, low effort first.

### Increment 1: Consolidate Agent Docs (1 day)

Merge SKILL.md + reference.md + aps-rules.md into a single aps-rules.md.
Target: 300 lines max. Remove all duplication. This alone will improve
every AI interaction.

### Increment 2: Fix Terminology (half day)

Global find-and-replace:
- "Task" → "Work Item" everywhere in aps-rules.md
- "Steps" → "Actions" everywhere in aps-rules.md and templates
- `.steps.template.md` → `.actions.template.md`
- Update all docs that say "Steps file" to "Action Plan"

### Increment 3: Add `aps status` Command (1-2 days)

Write `.status.json` from existing module files (extract current state). Add
`aps status` that pretty-prints it. Don't change any module formats — just
add the sidecar. Make the SessionStart hook output from this instead of
grepping markdown.

### Increment 4: Promote quickstart to First-Class (1 day)

Make the linter accept quickstart-format files (no metadata table, numeric
IDs without prefix). Change the getting-started guide to lead with this
format. Kill the "this is just a stepping stone" messaging.

### Increment 5: Add `aps context` Command (1 day)

A single command that outputs everything an AI needs. Concatenates the
condensed rules + current status + relevant module spec. This replaces the
"read 4 files" ceremony.

### Increment 6: Cross-File Linting (2-3 days)

Add dependency validation, orphan detection, and index↔module consistency
to the linter. This turns the Librarian agent's job into a free, instant
local check.

### Increment 7: Simplify Template Set (half day)

Remove quickstart (absorbed into single-file mode). Remove index-expanded
(make it a section in the index template docs). Remove solution template
(move to workflow guide as a suggestion). Target: 4 templates
(spec, index, action-plan, design).

---

## Part 5: The "Ducks Nuts" Pitch

What makes someone using BMAD or rolling their own go "holy shit"?

### 1. Zero-to-planning in 10 seconds

```bash
aps new "Add user authentication"
```

Creates a lintable, executable spec. No scaffolding ceremony. No template
decisions. Write your work items and go.

### 2. AI orientation in one command

```bash
aps context
```

Outputs everything the AI needs. 50 lines of rules, current status, what to
work on next. One file read, not five.

### 3. The guardrails actually work

The hooks system prevents AI drift in a way no other framework does. Your AI
can't end a session without updating specs. It gets reminded of its goals
before every code change. This is invisible when it works (which is always)
and invaluable when you look at the output quality.

### 4. Instant status

```bash
aps status
```

Know exactly where you are. No grepping files. No scrolling through markdown.
JSON-backed, pretty-printed, always accurate.

### 5. It's just markdown

No lock-in. No build step. No runtime. Works with Claude Code, Cursor,
Copilot, ChatGPT, pen-and-paper. Your specs outlive every tool you use.

### 6. Progressive complexity

Single file for a small feature. Full Index + Modules + Action Plans for a
large initiative. The system grows with your needs without forcing you to
decide upfront.

---

## Part 6: Competitive Position

### vs BMAD

BMAD is opinionated about roles (Business Analyst, Architect, Developer,
QA) and generates heavy spec documents. APS is lighter, more portable, and
doesn't assume a specific team structure. BMAD's documents tend toward the
prescriptive — they tell the AI how to implement. APS's "intent not
implementation" philosophy is architecturally better for AI that can reason.

**APS wins on:** Portability, lightness, tool-agnosticism, lean checkpoints.
**BMAD wins on:** Opinionated workflow for teams new to AI dev, role-based
structure.

### vs SpecKit

SpecKit leans into structured YAML/frontmatter specifications with tighter
integration to specific tools. APS's pure markdown approach is more
portable but less machine-parseable.

**APS wins on:** Zero dependencies, portability, human readability.
**SpecKit wins on:** Machine parseability, tighter tool integrations.

### vs Rolling Your Own

Most teams that "roll their own" end up with CLAUDE.md files full of rules,
scattered TODO.md files, and inconsistent spec formats. APS gives them
structure without vendor lock-in.

**APS wins on:** Structure, validation, the hooks system, the compound
engineering philosophy.
**Rolling your own wins on:** Zero learning curve, perfect fit for your
specific workflow.

The v2 changes above close the gaps. Progressive formality eliminates the
ceremony tax that pushes people to roll their own. The consolidated docs
eliminate the learning curve. The CLI commands eliminate the "parse markdown
yourself" problem.

---

## Summary: If We Started Over

| Keep | Change | Add |
|------|--------|-----|
| 4-layer hierarchy | Single-file as first-class | `aps status` from JSON |
| Plain markdown | One agent doc, not four | `aps context` for AI orientation |
| Trust layer (Ready gate) | Flat directory by default | `aps new` for zero-ceremony start |
| Hooks system | Two templates, not nine | Cross-file linting |
| Lean checkpoints | Pick one language (drop PS) | `aps promote` for progressive upgrade |
| Compound engineering lifecycle | Status sidecar (.status.json) | Tool-agnostic hook commands |

The soul of APS stays. The ceremony goes down. The tooling goes up.
