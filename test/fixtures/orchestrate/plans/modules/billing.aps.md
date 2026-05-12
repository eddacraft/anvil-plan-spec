# Billing Module

| ID | Owner | Status |
|----|-------|--------|
| BILLING | @test | Ready |

## Purpose

Validate no-ready-item behavior.

## In Scope

- Billing tasks

## Out of Scope

- Authentication tasks

## Interfaces

**Depends on:**

- AUTH

**Exposes:**

- Billing functions

## Ready Checklist

- [x] Purpose and scope are clear
- [x] Dependencies identified
- [x] Work items defined

## Work Items

### BILL-001: Charge customer

- **Intent:** Bill a customer
- **Expected Outcome:** Customer charges are recorded
- **Validation:** `true`
- **Dependencies:** PAYMENTS

### BILL-002: Wait for manual approval

- **Intent:** Ensure explicit malformed statuses are skipped
- **Expected Outcome:** This item is not selected by `aps next`
- **Validation:** `true`
- **Status:** Waiting

### BILL-003: Use lowercase module dependency

- **Intent:** Ensure unrecognized dependency tokens fail closed
- **Expected Outcome:** This item is not selected by `aps next`
- **Validation:** `true`
- **Dependencies:** database
