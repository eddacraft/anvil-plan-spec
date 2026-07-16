<!-- APS: See docs/ai/prompting/ for AI guidance -->
<!-- Executable only if work items exist and status is Ready. -->

# Payments API Module

| ID  | Owner  | Priority | Status      |
| --- | ------ | -------- | ----------- |
| PAY | @priya | high     | In Progress |

**Last reviewed:** 2026-07-16

## Purpose

Own the server side of card payments: payment intents, webhook handling, and
the settled-payments export. Card data never touches this service (SAQ-A,
hosted fields — index D-001).

## In Scope

- Payment-intent creation and confirmation against Stripe
- Webhook ingestion (idempotent) and payment state transitions
- Daily settled-payments export for finance

## Out of Scope

- Checkout rendering (CKUI module)
- Refunds (open question on the index)
- Multi-provider abstraction (D-001)

## Interfaces

**Depends on:**

- Stripe API — payment intents, webhooks

**Exposes:**

- `POST /api/payments/intent` → client secret for hosted fields
- `POST /api/payments/webhook` → 2xx on processed-or-duplicate
- Daily `settled-payments.csv` export

## Boundary Rules

- PAY must not render UI or hold card data
- All webhook handlers are idempotent (delivery gaps risk, index)

## Acceptance Criteria

- [x] Intent endpoint returns a client secret in sandbox
- [ ] Replayed webhooks do not double-settle a payment
- [ ] Export matches Stripe's settlement report for a test day

## Work Items

### PAY-001: Payment-intent endpoint — Complete

- **Status:** Complete
- **Summary:** Shipped in PR #12; sandbox-verified. Detail in history.
- **Validation:** `npm test -- payments/intent.test.ts`

### PAY-002: Idempotent webhook handler

- **Status:** In Progress
- **Intent:** Process Stripe webhooks exactly once per event
- **Expected Outcome:** Replayed deliveries return 2xx without repeating
  state transitions; unknown events are logged and acknowledged.
- **Validation:** `npm test -- payments/webhook.test.ts` (includes a
  duplicate-delivery case)
- **Dependencies:** PAY-001

### PAY-003: Settled-payments export

- **Status:** Ready
- **Intent:** Give finance a daily reconciliation file
- **Expected Outcome:** `settled-payments.csv` generated nightly; row count
  and totals match Stripe's settlement report on the sandbox fixture day.
- **Validation:** `npm test -- payments/export.test.ts`
- **Dependencies:** PAY-002
