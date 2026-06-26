---
name: dev-workflow
description: Use at the start of any development task to route to the correct skill, agent, and command for each lifecycle stage, from planning through branch, TDD, review, PR, and cleanup.
---

# Dev Workflow

Routing layer for the development lifecycle. Every non-trivial task follows this sequence - do not skip stages.

```
Plan Truth Gate -> Isolate -> TDD Code -> Verify -> Review -> PR -> Cleanup
```

## Stage Map

| Stage                  | What                                                                | Skill                                                                    | Agent                                                     | Command                |
| ---------------------- | ------------------------------------------------------------------- | ------------------------------------------------------------------------ | --------------------------------------------------------- | ---------------------- |
| **Idea / spec**        | Explore intent, design before code                                  | `brainstorming`                                                          | -                                                         | -                      |
| **Plan / truth gate**  | Turn intent into validated work                                     | `planning-workflow`, `aps-planning`, `writing-plans`, `planning-council` | `anvil-plan-spec` when available                          | `/plan` when available |
| **Isolate**            | Create isolated branch/worktree from the project integration branch | `using-git-worktrees`                                                    | -                                                         | -                      |
| **Code**               | TDD implementation                                                  | `test-driven-development`                                                | `tdd-coach`                                               | `/test`                |
| **Debug**              | Root cause analysis                                                 | `systematic-debugging`                                                   | `debugger`                                                | `/debug`               |
| **Verify**             | Evidence before completion claims                                   | `verification-before-completion`                                         | -                                                         | -                      |
| **Review (streaming)** | Iterative council during implementation                             | `local-review-council`                                                   | `council-reviewer`                                        | `/council`             |
| **Review (batch)**     | Formal multi-persona dossier at milestone                           | `council`                                                                | `council-reviewer` + `adversarial-reviewer` + specialists | `/council batch`       |
| **Finish**             | Commit, push, open PR                                               | `finishing-a-branch`                                                     | -                                                         | `/commit`              |
| **Address review**     | Resolve PR feedback and CI failures                                 | `addressing-pr-reviews`                                                  | -                                                         | -                      |
| **Parallelise**        | Independent tasks concurrently                                      | `parallel-agents`                                                        | `autonomous`                                              | `/delegate`            |
| **Loop execution**     | Autonomous APS execution after interactive approval                 | `aps-loop`                                                               | `autonomous`                                              | -                      |

## Rules

1. **Start from planning truth.** If the work is not already validated and ready, invoke `planning-workflow`. If the project uses APS, invoke `aps-planning` before branch or code.
2. **Isolate from the project integration branch.** Follow the project's branch naming and worktree policy; do not assume the integration branch is named `dev`.
3. **Code through TDD.** Invoke `test-driven-development` for implementation. If test-first is not practical, record the replacement evidence before coding.
4. **Review before PR.** Use the project's review surface and address critical/major findings before opening or updating a PR.
5. **Verify before claiming complete.** Evidence before assertions - use `verification-before-completion`.
6. **Preserve post-merge validation.** If work needs post-merge checks, write them to the project's tracked post-merge review location.
7. **Run local CI-equivalent gates before opening the PR.** The repo-mandated
   validation commands must run green locally before push. Check `CLAUDE.md` or
   project docs for the exact commands; common examples are
   `pnpm format:check && pnpm lint:check && pnpm typecheck && pnpm test` for
   JS/TS projects and `cargo test --workspace` for Rust. CI is a backstop, not
   the primary signal. Tick test-plan checkboxes only after the command actually
   ran, not aspirationally.
8. **Keep lockfile and generated artefacts atomic with the change that causes
   them.** If a PR touches a dependency lockfile (for example `Cargo.lock` or
   `pnpm-lock.yaml`) or any file generated from it (for example
   `ACKNOWLEDGEMENTS.md` or `licenses/` manifests), regenerate those artefacts
   and include the diff in the same PR. Splitting them creates freshness-check CI
   failures that can block later integration work. Check `CLAUDE.md` for the
   project-specific regeneration command.
9. **Keep bookkeeping PRs single-purpose.** APS status updates, index counter
   adjustments, and runbook fixes ship as standalone PRs. Do not bundle them
   with dependency updates, code refactors, or feature work; a CI failure on the
   heavyweight change blocks the trivial bookkeeping, and the broader review
   surface erodes the "trivially mergeable" property bookkeeping PRs depend on.

## Decision Points

**Starting a new task:**
-> `planning-workflow` if no validated ready item exists -> `aps-planning` if APS exists -> `using-git-worktrees` -> `test-driven-development` -> code

**Implementation unclear:**
-> `planning-workflow` -> `brainstorming` when required -> `writing-plans` (or `planning-council` for multi-persona design) -> `aps-planning` when APS exists -> continue

**Tests failing unexpectedly:**
-> `systematic-debugging` before any other action

**About to commit:**
-> `verification-before-completion` gate -> `local-review-council` (streaming) or `council` (batch) -> `finishing-a-branch`

**PR review feedback returned:**
-> `addressing-pr-reviews` - fix CI first, then walk each unresolved thread

**Multiple independent tasks:**
-> `parallel-agents` to dispatch subagents per task

**Approved APS plan ready for autonomous execution:**
-> `aps-loop` after the user has approved the planning/readiness decision

## Loop Handoff Outcomes

When called from `aps-loop`, finish with one of these outcomes so the loop can
decide whether to continue, replan, or checkpoint:

| Outcome                 | Meaning                                                                    | Next step                                                              |
| ----------------------- | -------------------------------------------------------------------------- | ---------------------------------------------------------------------- |
| `done`                  | Implementation, validation, and review evidence are complete               | `aps-loop` reconciles and selects the next item                        |
| `blocked`               | Work cannot continue because of an external dependency                     | `aps-loop` records blocker and selects other work if available         |
| `needs-plan-update`     | The approved item is stale, too broad, already done, or missing validation | route to `planning-workflow` / `aps-planning`                          |
| `validation-failed`     | Focused implementation attempts did not make validation pass               | route to `systematic-debugging`, then re-enter implementation or block |
| `needs-user-checkpoint` | Product, scope, safety, destructive, or irreversible decision required     | stop and ask the user                                                  |

## APS Status Lifecycle

Use this lifecycle only in projects with APS:

```
Draft -> Proposed -> Ready -> In Progress -> Merged -> Released/Shipped -> Complete
                              ^              ^              ^              ^
                        (you start)   (PR merged)   (release record)  (cleanup agent)
```

`Committed` is legacy wording for `Merged`. New APS text should prefer
`Merged` and `Released/Shipped`. Cleanup agent (`scripts/aps-cleanup.sh` where
present) auto-advances post-merge states when release evidence is recorded.

## Project References

- Branching strategy: project-specific, commonly `docs/guides/branching-strategy.md`
- Worktree policy: project-specific, commonly `docs/guides/worktree-policy.md`
- APS rules: `plans/aps-rules.md` when present
- Council architecture: see `council` skill when available
