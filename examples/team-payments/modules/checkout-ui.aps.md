<!-- APS: See docs/ai/prompting/ for AI guidance -->
<!-- Executable only if work items exist and status is Ready. -->

# Checkout UI Module

| ID   | Owner  | Priority | Status |
| ---- | ------ | -------- | ------ |
| CKUI | @marco | high     | Ready  |

**Last reviewed:** 2026-07-16

## Purpose

Render the card step of checkout with Stripe hosted fields and surface
payment failures as retryable errors. Owned by @marco; PAY endpoints are the
only backend contact surface.

## In Scope

- Card entry step using hosted fields (no card data in our DOM)
- Success / failure / retry states in the checkout flow

## Out of Scope

- Payment state transitions (PAY module)
- Cart and shipping steps (existing checkout)

## Interfaces

**Depends on:**

- PAY — `POST /api/payments/intent` for the client secret

**Exposes:**

- `<CardPaymentStep />` in the checkout flow

## Boundary Rules

- CKUI must not call Stripe's server APIs directly — only hosted fields
- Failures render a retry affordance; no silent drops (index success criteria)

## Acceptance Criteria

- [ ] Card step renders hosted fields in sandbox
- [ ] Declined card shows a retryable error state

## Work Items

### CKUI-001: Card entry step with hosted fields

- **Status:** Ready
- **Intent:** Let a customer enter card details without expanding PCI scope
- **Expected Outcome:** Card step mounts hosted fields using the client
  secret from PAY-001 and confirms the payment in sandbox.
- **Validation:** `npm test -- checkout/card-step.test.tsx`
- **Dependencies:** PAY-001

### CKUI-002: Failure and retry states

- **Status:** Ready
- **Intent:** Make failed payments visibly retryable
- **Expected Outcome:** Declines and network failures render distinct,
  retryable error states; retry re-uses the same intent when Stripe allows.
- **Validation:** `npm test -- checkout/card-errors.test.tsx`
- **Dependencies:** CKUI-001
