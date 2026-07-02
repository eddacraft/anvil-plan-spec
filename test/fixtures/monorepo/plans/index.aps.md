<!-- APS: See docs/ai/prompting/ for AI guidance -->
<!-- This document is non-executable. -->
<!-- Fixture: nested-plans federation ROOT. See plans/designs/2026-06-27-nested-plans.design.md -->

# Acme Platform

## Overview

A two-package monorepo whose packages keep independent plans. This root links
the child plans and rolls up their status; it owns no modules of its own.

## Problem & Success Criteria

**Problem:** `core` and `api` have separate owners and release cadences, so a
single shared backlog does not fit.

**Success Criteria:**

- [ ] Each package plan lints and ships independently
- [ ] Cross-package work is traceable via cross-tree references

## Constraints

- Packages stay independently releasable

## Child Plans

- [core](../packages/core/plans/index.aps.md) — shared domain + auth
- [api](../packages/api/plans/index.aps.md) — HTTP surface and handlers

## Roll-up

| Child | Modules (complete/total) | Next ready item | Status |
| ----- | ------------------------ | --------------- | ------ |
| core  | 0/1                      | AUTH-001        | Ready  |
| api   | 0/1                      | —               | Ready  |

## Modules

A federation root owns no modules of its own; work lives in the child plans.

| Module | ID  | Owner | Status | Priority | Tags | Dependencies |
| ------ | --- | ----- | ------ | -------- | ---- | ------------ |

## Risks & Mitigations

| Risk                          | Impact | Likelihood | Mitigation                         |
| ----------------------------- | ------ | ---------- | ---------------------------------- |
| Cross-tree refs drift untyped | medium | medium     | Lint traversal resolves them (MONO-002) |

## Decisions

- **D-001:** Adopt nested plans over tagged modules — packages need autonomy
