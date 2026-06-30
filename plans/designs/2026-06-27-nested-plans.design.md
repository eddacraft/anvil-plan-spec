# Nested Plans Convention — MONO Module

| Field   | Value                                              |
| ------- | -------------------------------------------------- |
| Date    | 2026-06-27                                         |
| Status  | Draft                                              |
| Modules | [monorepo](../modules/monorepo.aps.md)             |
| Scope   | MONO-001 — define the nested `index.aps.md` convention |
| Source  | [MONO-001 action plan](../execution/MONO-001.actions.md) |

## Problem

A monorepo whose packages have independent owners, lifecycles, or extraction
plans cannot share one `plans/` root: every module competes for one namespace,
the index grows unboundedly, and a package cannot carry its plan with it if
extracted. The federated tier needs a convention for **nested** `index.aps.md`
files — each a standalone plan, rolling up into one root view — that the
templates, fixtures, lint, and orchestration can all rely on.

This is the **authoritative spec** that the templates (MONO-001 Wave 2) and the
fixture tree (Wave 3) implement, and that lint traversal (MONO-002) and
orchestration (MONO-003) parse. The user-facing write-up in `docs/monorepo.md`
is a later transform (MONO-006); this design doc is the contract until then.

It covers the federated tier only. Package **tags** (the existing
[2026-01-21 monorepo design](./2026-01-21-monorepo-support-design.md)) remain the
default for small monorepos — see D-004.

## Decisions in force

Resolved 2026-06-26 in [monorepo.aps.md](../modules/monorepo.aps.md):

- **D-001 — co-located:** child plans live at
  `packages/<pkg>/plans/index.aps.md`; discovery is by directory layout.
- **D-002 — bare IDs, qualified cross-tree refs:** IDs are unprefixed within a
  tree; cross-tree references carry the child's path-derived name.
- **D-003 — standalone children:** each child is a complete APS plan that lints
  in isolation.

---

## Design

The convention (§1) and its machine-readable grammar (§2) below are the
contract. Sections are numbered so the templates and fixtures can cite them.

### 1. The convention (Action 1)

#### 1.1 Declaration

A **parent** `index.aps.md` declares its children under a single section whose
heading is exactly `## Child Plans`. Each child is one list item linking to the
child's `index.aps.md`:

```markdown
## Child Plans

- [core](../packages/core/plans/index.aps.md) — shared domain + persistence
- [api](../packages/api/plans/index.aps.md) — HTTP surface and handlers
```

The link text is the child's **path-derived name** (§1.2). The parent owns no
modules of its own beyond what a normal index carries; it links children and
(via MONO-004) rolls up their status. A parent is any plan root that contains a
`## Child Plans` section.

#### 1.2 Child name derivation

A child's name is the package path segment immediately above its `plans/`
directory:

| Child `index.aps.md` path                  | Derived name |
| ------------------------------------------ | ------------ |
| `packages/core/plans/index.aps.md`         | `core`       |
| `packages/api/plans/index.aps.md`          | `api`        |
| `apps/web/plans/index.aps.md`              | `web`        |

The name is the cross-tree reference prefix (§1.4). Names must be unique within
a federated tree; lint reports duplicates (handled in MONO-002). Names are
lowercase `[a-z0-9][a-z0-9-]*` — if a real directory segment violates this
(e.g. capitalisation), the link text is the normalised form and the path in the
link is the source of truth.

#### 1.3 Discovery without the parent

Tooling discovers plan roots **by directory layout**, not only by following
parent links: any file matching `**/plans/index.aps.md` at or below the
invocation root is a plan root. The topmost such file (shallowest path) is the
parent; the rest are children. This lets `--plans` point at any node — a child
extracted to its own repo is still a valid standalone plan (D-003).

#### 1.4 Cross-tree references

A dependency or interface reference that targets another tree is written
`<child-name>:<ID>`:

```markdown
- **Dependencies:** core:AUTH-001
```

A **bare** ID (`AUTH-001`) always resolves within the current tree. Resolution
and validation of cross-tree references is MONO-002's job; this convention only
fixes the syntax so the fixture (Wave 3) is well-formed.

---

### 2. Machine-readable markers (Action 2)

These are the exact tokens MONO-002's parser keys on. Authored here so
templates and fixtures are built against the real grammar.

#### 2.1 Child declaration

- **Section heading:** matched by `^##[ \t]+Child Plans[ \t]*$` (case-sensitive).
- **List item:** `- [<child-name>](<relative-path-to-child-index>) — <summary>`
  - `<child-name>`: `[a-z0-9][a-z0-9-]*`
  - `<relative-path-to-child-index>`: a relative path ending in
    `plans/index.aps.md`
  - separator is an em dash `—` (consistent with existing index link rows)

#### 2.2 Cross-tree reference

- **Regex:** `\b([a-z0-9][a-z0-9-]*):([A-Z]+-[0-9]+)\b`
  - group 1 = child name, group 2 = work-item / module ID

#### 2.3 Non-collision check

The new markers must not collide with existing single-plan grammar:

- `## Child Plans` is a new heading — no existing template or module uses it.
- The conductor `Type:` marker, the `W003` cross-module dependency check, and
  completed-index roll-ups all operate on bare IDs and unaffected headings; the
  cross-tree regex requires a `name:` prefix, which bare intra-tree IDs never
  carry.
- **Validation:** the cross-tree regex must return **zero** matches against the
  current single-tree repo, *excluding files that document the convention*:

  ```sh
  grep -rnE '\b([a-z0-9][a-z0-9-]*):([A-Z]+-[0-9]+)\b' plans/ --include='*.md'
  ```

- **Finding (2026-06-27):** run bare, this matches only the `core:AUTH-001`
  examples inside this design doc, the MONO-001 action plan, and the monorepo
  module's decision prose — all inside inline code spans or a fenced example.
  Excluding those three files yields zero matches, so there is **no
  pre-existing collision** in real plan content. The consequence for MONO-002:
  its parser must **scan only the value side of `Dependencies:` / interface
  fields and skip inline code spans and fenced code blocks** — a naive
  whole-file regex would treat documentation examples as live references.

---

## Scope boundary

MONO-001 defines this convention, the templates, and a lintable fixture where
the root and **each child** lint clean *independently*. A single `aps lint`
invocation that walks the whole tree is **MONO-002**.

## Findings from the MONO-001 fixture (2026-06-27)

The fixture tree under `test/fixtures/monorepo/` (root + `core` + `api`, with
`api`'s `HND-001` carrying `Dependencies: core:AUTH-001`) was linted per-root
and whole-tree. Two facts for MONO-002 fell out:

1. **Recursive find already resolves cross-tree deps when the whole tree is
   linted.** `lib/lint.sh build_id_index` indexes IDs from *every* file found
   under the lint root, so `./bin/aps lint test/fixtures/monorepo` resolves
   `core:AUTH-001` and reports no W003. MONO-002 can build on this rather than
   reinvent traversal — its job is to scope by `## Child Plans` links, add
   collision detection, and make the *prefix* meaningful (today the prefix is
   ignored, see below).
2. **W003 strips the `<name>:` prefix.** `check_w003_dependencies`
   (`lib/rules/workitem.sh`) extracts `[A-Z]+-[0-9]{3}` and so reads
   `core:AUTH-001` as bare `AUTH-001`. Linting the `api` child *alone* therefore
   emits a W003 ("Dependency 'AUTH-001' not found in plan") — correct in spirit
   (the dep is genuinely outside an isolated child) but imprecise. **MONO-002
   must** recognise `<name>:<ID>` as a cross-tree reference: resolve it against
   the named sibling when present, and when absent (true isolation) emit a
   distinct "unresolved cross-tree dependency" notice rather than the generic
   "not found in plan". Combined with the Wave-1 finding (§2.3), the parser must
   key on the *value side* of dependency/interface fields, honour the `name:`
   prefix, and skip code spans/fences.

At MONO-001 close, the one expected `api`-alone W003 was the only non-clean
result; parent, `core`, and whole-tree all lint clean. MONO-002 (below) changes
the `api`-alone result to clean by making W003 prefix-aware.

## As-built: lint traversal (MONO-002, 2026-06-27)

Implemented in the bash linter (`lib/lint.sh`, `lib/rules/workitem.sh`):

- **Traversal.** `cmd_lint` follows `## Child Plans` links from a parent index
  (`expand_child_plans`, transitive, deduped on lexically-normalised paths), so
  `aps lint <parent>/plans` validates every child tree as one plan. This closes
  the canonical case where children live at `packages/<pkg>/plans/`, outside the
  parent dir (a whole-tree target already pulled children in via recursive find).
- **Prefix-aware W003.** Dependencies are parsed keeping the `<name>:` prefix.
  A `<name>:<ID>` ref resolves against the named child's IDs
  (`build_child_registry` → `APS_CHILD_IDS`) when that child is in scope; when it
  is not (a child linted alone), the ref is an intentional external link and
  stays silent, so standalone children lint clean. A bare ID keeps its existing
  behaviour (resolve in-file, then against the whole linted scope).
- **Collision detection (W020).** `check_cross_tree_collisions` warns once per
  work-item ID defined in more than one child tree (warning, not error — D-002
  keeps each tree independently valid; collisions only make `<name>:<ID>` refs
  ambiguous). Fires only when a federation parent is in scope.

Known nuance: a *bare* ID still resolves across the entire linted scope, not
strictly within its own tree (pre-existing `APS_TREE_IDS` leniency). The
`name:` prefix is the explicit cross-tree mechanism; tightening bare resolution
to in-tree-only would change long-standing single-repo behaviour and is out of
scope here.

## Open items deferred downstream

- **PowerShell parity** for the MONO-002 behaviour (`lib/Lint.psm1`,
  `lib/rules/WorkItem.psm1`) — the repo keeps a PS port in step with the bash
  linter; not done yet (no `pwsh` in the dev env to verify a port against)
- Federated `aps next`/`graph`/`audit` and scoping → MONO-003
- Roll-up section population in the parent → MONO-004
- User-facing docs + migration guide → MONO-006
