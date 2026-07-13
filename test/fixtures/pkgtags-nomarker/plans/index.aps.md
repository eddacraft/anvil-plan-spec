# Packages Tag Fixture (no monorepo markers)

## Overview

A single-package project whose plan carries a stray `Packages:` tag. There is
no `packages/` or `apps/` directory, so W022 must not fire — non-monorepo
projects never pay for the check.

## Problem & Success Criteria

Zero W022 warnings despite an unresolvable tag.

## Modules

| Module | Status |
| --- | --- |
| [auth](./modules/auth.aps.md) | Complete |
