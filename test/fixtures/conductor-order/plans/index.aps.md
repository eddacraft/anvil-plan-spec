# Conductor Order Fixture

## Overview

Locks the W017-before-W002 emission order for an active conductor module.

## Problem & Success Criteria

A Ready conductor with a missing Last reviewed field and a bad cross-ref must
emit W017 before W002 in every CLI.

## Modules

### Core

| Module | Status |
| --- | --- |
| [auth](./modules/auth.aps.md) | Complete |

### Conductor / Crosscutting (Adopted)

| Module | Status |
| --- | --- |
| [release-planning](./modules/release-planning.aps.md) | Ready |
