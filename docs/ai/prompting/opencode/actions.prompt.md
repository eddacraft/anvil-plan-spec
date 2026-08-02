# APS Action Plans Prompt (OpenCode)

This variant defers to the [generic action plan prompt](../actions.prompt.md); only OpenCode-specific differences follow.

ROLE: Executor
MODE: Propose an action plan OR Execute an action plan (one action at a time)

## OpenCode Flow Control

- In execute mode, pause after each action to validate and report its checkpoint
  before proceeding.
- For a parallel wave, reconcile every checkpoint report before advancing to
  the next wave.
- If a checkpoint is blocked, stop and report the reason.
