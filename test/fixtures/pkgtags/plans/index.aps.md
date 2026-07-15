# Packages Tag Fixture (dirty)

## Overview

Exercises W022: a `Packages:` scope tag that resolves to no workspace
directory. The workspace has `packages/core` and `apps/api`.

## Problem & Success Criteria

The linter must flag exactly the typo'd entries (`storefront`, `packages/wrong`)
and stay silent on resolvable ones.

## Modules

| Module | Status |
| --- | --- |
| [auth](./modules/auth.aps.md) | Complete |
