# APS Usability Review

**Date:** 2025-12-25
**Reviewer:** Claude (Critical Analysis)
**Scope:** Complete usability assessment of Anvil Plan Spec format, templates, documentation, and examples

---

## Executive Summary

APS presents a compelling vision for portable, tool-agnostic planning in AI-assisted development. However, several usability barriers may prevent adoption, particularly for new users. This review identifies 15 critical areas for improvement across documentation, templates, terminology, and workflow integration.

**Severity Ratings:**

- 🔴 **Critical** - Major barrier to adoption
- 🟡 **Important** - Degrades user experience significantly
- 🟢 **Minor** - Polish and refinement

---

## 1. Terminology & Naming Issues

### 🔴 "Leaf" is confusing and abstract

**Problem:** The term "Leaf" doesn't communicate its purpose. Users must understand it's a "module" and that it's called "leaf" because it's the end of the planning tree.

**Evidence:**

- README.md:58-60 requires a multi-sentence explanation of why it's called "leaf"
- The table alternates between "Leaf/Module" creating confusion
- File names use "leaf.template.md" but examples use descriptive names like "auth.aps.md"

**Impact:** New users won't understand what a "leaf" is without reading detailed explanation. The metaphor doesn't match common software terminology.

**Recommendation:**

- Rename to "Module Template" or "Work Module"
- Reserve "leaf" as a conceptual note in documentation
- Update all references: `module.template.md` instead of `leaf.template.md`

### 🟡 "SCOPE" placeholder creates confusion

**Problem:** Templates use `SCOPE` as a placeholder for module identifiers, but this conflicts with "In Scope / Out of Scope" sections.

**Evidence:**

- leaf.template.md:7 uses "SCOPE" in metadata table
- leaf.template.md:14-22 has "In Scope" and "Out of Scope" sections
- Task IDs use pattern "SCOPE-001" which reads awkwardly

**Impact:** Users may think "SCOPE" has special meaning or unclear what to replace it with.

**Recommendation:**

- Use `MODULE-ID` or `{module-name}` as placeholder
- Provide clear examples: "AUTH", "PAYMENTS", "UI"
- Add guidance: "Use 2-6 character uppercase identifier"

### 🟢 Inconsistent terminology for "execution authority"

**Problem:** The concept of "execution authority" is used but not consistently explained.

**Recommendation:**

- Define clearly in glossary
- Use consistently across all documentation
- Consider simpler phrasing: "tasks authorise implementation"

---

## 2. Template Complexity & Cognitive Load

### 🔴 Too many required decisions upfront

**Problem:** Users must understand the entire 4-layer hierarchy before creating their first spec.

**Evidence:**

- getting-started.md presents all concepts before letting users try anything
- Decision tree (lines 99-122) shows 7+ decision points
- No "quick start" template for trying APS without commitment

**Impact:** High activation energy. Users bounce before seeing value.

**Recommendation:**

- Create `quickstart.template.md` - single-file, minimal fields
- "Try APS in 5 minutes" tutorial
- Progressive disclosure: start simple, add complexity as needed

### 🟡 leaf.template.md has too many fields

**Problem:** The full leaf template has 13+ distinct sections. Unclear which are truly required.

**Fields count:**

- Metadata table (4 fields)
- Purpose, In/Out Scope, Interfaces, Boundary Rules, Acceptance Criteria, Risks, Tasks, Execution, Decisions, Notes

**Evidence:**

- simple.template.md exists as a "lightweight" alternative, proving leaf is too heavy
- Examples don't fill all sections consistently

**Impact:** Template feels bureaucratic, intimidating for small projects.

**Recommendation:**

- Mark optional sections clearly with _(optional)_ in template
- Provide field-by-field guidance in comments
- Create a "minimal leaf" example showing bare essentials

### 🟡 Task fields feel redundant

**Problem:** Tasks require: Intent, Expected Outcome, Scope, Non-scope, Files, Dependencies, Validation, Confidence, Risks

**Overlap:**

- "Intent" vs "Expected Outcome" - semantic overlap
- "Scope" vs "Non-scope" - inversions of each other
- "Files" - often wrong, becomes stale

**Impact:** Filling tasks feels repetitive and tedious.

**Recommendation:**

- Consolidate "Intent" and "Expected Outcome" into single field
- Make "Non-scope" explicitly optional (use only when clarification needed)
- Make "Files" optional with guidance: "Best effort, don't worry if incomplete"

---

## 3. Documentation & Onboarding

### 🔴 No minimal "hello world" example

**Problem:** Users can't quickly try APS. Must set up folder structure, understand templates, etc.

**Current flow:**

1. Read README (151 lines)
2. Read getting-started.md (148 lines)
3. Choose template
4. Create folder structure
5. Fill template
6. Hope you did it right

**Impact:** Hours to first value. High abandonment risk.

**Recommendation:**

```markdown
# hello-world.aps.md - Complete minimal example

Copy this file, replace the bracketed parts, and you're using APS!

## What: [One sentence - what you're building]

## Why: [One sentence - what problem it solves]

## Success:

- [ ] [Observable outcome 1]
- [ ] [Observable outcome 2]

## Tasks:

### Task 001: [First thing to build]

- **Outcome:** [How you know it's done]
- **Test:** `[command to verify]`
```

### 🟡 Getting started is comprehensive but overwhelming

**Problem:** getting-started.md is thorough but front-loads too much information.

**Flow issues:**

- Step 1 shows full folder structure before explaining why
- Presents all template choices before user knows which they need
- Decision tree is helpful but comes late (line 99)

**Recommendation:**

- Lead with decision tree
- One-page quick start for common cases
- Move comprehensive guide to separate "Complete Guide" doc

### 🟡 Examples are buried

**Problem:** Examples (user-auth, opencode-companion) are excellent but not prominently featured.

**Evidence:**

- README.md:99-102 mentions them briefly
- getting-started.md:145 references them at the end
- Not shown in context of "when to use each template"

**Recommendation:**

- Create "Examples" section in README with screenshots/previews
- Link specific examples next to each template description
- Add "See example: user-auth/modules/auth.aps.md" in template comments

---

## 4. Hierarchy & Structure Clarity

### 🟡 "Non-executable" vs "Executable if Ready" is confusing

**Problem:** Index is "non-executable", Leaf is "executable if tasks exist and status is Ready", Tasks are "execution authority"

**Evidence:**

- index.template.md:2 "This document is non-executable"
- leaf.template.md:2 "Executable only if tasks exist and status is Ready"
- Not clear what "executable" means in practice

**Impact:** Users don't understand when to act on documents.

**Recommendation:**

- Replace "executable/non-executable" with clearer language:
  - Index: "Planning document - describes intent, doesn't authorize work"
  - Module: "Work can begin when status=Ready and tasks exist"
  - Task: "Authorises implementation"
- Add workflow diagram showing the progression

### 🟡 Unclear when to change status from Draft to Ready

**Problem:** No clear criteria for when a module is "Ready"

**Current guidance:** None explicit

**Impact:** Users uncertain when to flip status, may start work prematurely or wait too long.

**Recommendation:**

- Add checklist to templates:

```markdown
## Ready Checklist

Mark status as "Ready" when:

- [ ] Purpose and scope are clear
- [ ] Interfaces defined (or confirmed none needed)
- [ ] Dependencies resolved or explicitly blocked
- [ ] At least one task defined
- [ ] Stakeholder approval (if required)
```

---

## 5. File Organization & Conventions

### 🟡 Folder structure is prescribed but not enforced

**Problem:** Documentation shows `plans/` folder structure but examples use different structure.

**Evidence:**

- README.md:123-133 shows `plans/` at root
- getting-started.md:14-24 shows same
- examples/user-auth/ doesn't follow this (no `plans/` folder)

**Impact:** Confusion about "correct" structure.

**Recommendation:**

- Either make examples follow the structure OR
- Explicitly state "structure is flexible, adapt to your needs"
- Provide 2-3 structure options (monorepo, standalone, embedded)

### 🟢 .aps.md extension isn't explained

**Problem:** Files use `.aps.md` extension but this isn't documented.

**Recommendation:**

- Explain in README: "We use .aps.md to distinguish spec files from regular docs"
- Note it's optional: "Plain .md works fine if you prefer"

---

## 6. Workflow & Tool Integration

### 🔴 Unclear how to actually "execute" tasks

**Problem:** APS defines tasks but doesn't explain how they connect to actual development workflow.

**Questions users will have:**

- Do I create a git branch per task?
- Do I update the APS file as I work?
- When do I mark tasks complete?
- What if implementation reveals the task was wrong?

**Current guidance:** README.md:79-86 shows "how to use APS" in different tools but not the workflow.

**Impact:** APS feels like extra work, not workflow enhancement.

**Recommendation:**

- Add "Development Workflow" guide:
  - How to pick a task
  - How to track progress
  - How to handle changes
  - How to mark completion
  - Integration with git/PRs
- Show concrete example: "Day in the life with APS"

### 🟡 AI prompt usage is unclear

**Problem:** docs/ai/prompting/ contains great prompts but unclear how to use them.

**Evidence:**

- getting-started.md:135-142 says "Reference" the prompts
- No examples of actually doing this in different tools
- OpenCode variants exist but unclear what's different

**Recommendation:**

- Add "Using AI Prompts" guide with tool-specific examples:
  - Claude/ChatGPT: "Paste this into your conversation: [prompt + spec]"
  - Cursor: "Add to .cursorrules: [prompt]"
  - Claude Code: "Use slash command: /plan [spec-file]"
- Show before/after of AI assistance with vs without prompts

### 🟢 No guidance on completion/archival

**Problem:** What happens to tasks/modules when done?

**Recommendation:**

- Add completion guidance:
  - Mark completed tasks with ~~strikethrough~~ or `✓ Completed: YYYY-MM-DD`
  - Archive old specs to `plans/archive/` or keep as history
  - Update Index when modules complete

---

## 7. Validation & Quality

### 🟡 No validation tooling

**Problem:** Users can create invalid APS files with no feedback.

**Evidence:**

- ROADMAP.md mentions validation tooling (planned)
- Currently only markdownlint (formatting, not structure)

**Impact:** Users may create malformed specs, miss required fields.

**Recommendation:**

- Build `aps validate` CLI tool:
  - Check required fields present
  - Validate task ID format
  - Check dependency references resolve
  - Warn on common issues
- JSON Schema for APS structure (enables IDE autocomplete)

---

## 8. Missing Use Cases & Guidance

### 🟡 Solo developer guidance missing

**Problem:** Templates assume teams (owner field, stakeholders, etc.)

**Evidence:**

- All templates have "Owner: @username"
- Examples reference team scenarios

**Recommendation:**

- Add note: "Solo? Use @me or your username. Owner field helps AIs understand responsibility."
- Show solo developer example

### 🟡 No guidance on evolving/refactoring specs

**Problem:** Requirements change. How do you handle?

**Questions:**

- Can I change a task after starting?
- What if I split a module?
- How to handle scope creep?

**Recommendation:**

- Add "Living Documents" guide:
  - When to update vs create new task
  - How to handle discovered work
  - Refactoring specs (splitting modules, etc.)

### 🟢 No migration guide

**Problem:** Projects mid-flight can't easily adopt APS.

**Recommendation:**

- "Adopting APS Mid-Project" guide:
  - Start with Index capturing current state
  - Add modules for active work areas only
  - Gradually expand coverage

---

## 9. Template-Specific Issues

### 🟢 steps.template.md: Checkpoint examples needed

**Problem:** "Observable checkpoint" is abstract.

**Good checkpoint:** "Migration file exists with users table schema"
**Bad checkpoint:** "Database setup complete"

**Recommendation:**

- Add checkpoint guidance to template:

```markdown
<!-- Checkpoint should be verifiable by someone else
Good: "File X exists with function Y"
Bad: "Setup complete"
-->
```

### 🟢 index.template.md: System Map often empty

**Problem:** Mermaid graph placeholder but many projects won't need this.

**Recommendation:**

- Mark as optional
- Show alternative: bullet list of module relationships

---

## 10. Content & Clarity Issues

### 🟢 "Boundary Rules" section is unclear

**Problem:** leaf.template.md:34-37 has "Boundary Rules" but unclear vs "Out of Scope"

**Example shows:** "AUTH must not depend on SESSION"

**This is:** An architectural constraint, not a scope boundary.

**Recommendation:**

- Rename to "Architectural Constraints" or "Module Rules"
- Add guidance: "Use for dependency rules, layering, coupling constraints"

### 🟢 Confidence field lacks guidance

**Problem:** Tasks have "Confidence: medium" but no definition of low/medium/high.

**Recommendation:**

- Add to template comments:

```markdown
<!-- Confidence:
- high: Clear requirements, familiar patterns
- medium: Some unknowns, moderate risk
- low: Exploratory work, high uncertainty
-->
```

---

## Positive Findings

Worth preserving and emphasizing:

1. **Portability is genuinely valuable** - Markdown specs work everywhere
2. **Examples are excellent** - user-auth shows realistic usage
3. **Separation of planning and execution** - Conceptually sound
4. **Checkpoint-based steps** - Better than time/commit based (per ADR-001)
5. **Tool-agnostic AI prompts** - Smart approach to AI integration
6. **Principle-driven** - Four principles are clear and memorable

---

## Priority Recommendations

### Quick Wins (1-2 days)

1. Create `quickstart.template.md` with minimal fields
2. Add "hello world" single-file example to README
3. Mark optional fields in templates with _(optional)_
4. Add checkpoint and confidence guidance to templates
5. Fix terminology: SCOPE → MODULE-ID in examples

### High Impact (1 week)

1. Write "Development Workflow" guide with concrete examples
2. Reorganize getting-started.md: decision tree first, details later
3. Create "Using AI Prompts" guide with tool-specific examples
4. Add "Ready Checklist" to module templates
5. Unify folder structure between docs and examples

### Strategic (2-4 weeks)

1. Build basic `aps validate` CLI tool
2. Create video walkthrough / interactive tutorial
3. Add JSON Schema for IDE support
4. Write "Solo Developer" and "Mid-Project Adoption" guides
5. Consider renaming "Leaf" to "Module" throughout

---

## Conclusion

APS has strong conceptual foundations and solves a real problem (planning portability). However, current usability barriers may limit adoption:

- **Too much upfront investment** - Users need hours to understand before first use
- **Terminology creates confusion** - "Leaf", "SCOPE", "executable" aren't intuitive
- **Missing workflow integration** - Unclear how APS fits into daily development
- **Template complexity** - Heavy templates for simple needs

**Core recommendation:** Embrace progressive disclosure. Give users a 5-minute path to value, then layer in sophistication. The comprehensive approach is great for power users but needs a gentle on-ramp.

**Biggest opportunity:** The "hello world" example could become APS's best marketing tool - show don't tell.
