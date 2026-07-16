<!-- APS: See docs/ai/prompting/ for AI guidance -->
<!-- Conductor module: coordinates work across PAY and CKUI. -->

# Payments Launch (Conductor)

| ID     | Type      | Owner | Priority | Status      |
| ------ | --------- | ----- | -------- | ----------- |
| LAUNCH | Conductor | @sam  | medium   | In Progress |

**Last reviewed:** 2026-07-16

## Purpose

Coordinate the cross-module launch work that no vertical module owns: the
end-to-end sandbox rehearsal and the go-live checklist. This is the
team-shaped pattern for "work that spans two owners" — it lives in a
conductor instead of being duplicated into PAY and CKUI.

## Coordinated Modules

- [payments-api](./payments-api.aps.md) — PAY (@priya)
- [checkout-ui](./checkout-ui.aps.md) — CKUI (@marco)

## Work Items

### LAUNCH-001: End-to-end sandbox rehearsal

- **Status:** Ready
- **Intent:** Prove the whole flow before go-live, across both owners' work
- **Expected Outcome:** A scripted rehearsal (happy path, decline, webhook
  replay) passes in sandbox with PAY-002 and CKUI-002 in place.
- **Validation:** `npm run e2e -- payments-rehearsal`
- **Dependencies:** PAY-002, CKUI-002

### LAUNCH-002: Go-live checklist and flag flip

- **Status:** Draft
- **Intent:** Make launch a checklist, not a heroic afternoon
- **Expected Outcome:** Checklist covering keys rotation, webhook endpoint
  registration, finance sign-off on one exported day, and the feature-flag
  flip; each line has an owner.
- **Validation:** Checklist reviewed at the launch review; flag flipped in
  staging first
- **Dependencies:** LAUNCH-001
