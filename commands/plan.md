# /plan

Start or continue APS planning for this project.

## Instructions

You are starting an APS (Anvil Plan Spec) planning session. Follow these steps:

### Step 1: Assess current state

Check what APS artefacts already exist:

1. Look for `plans/` directory
2. Read `plans/aps-rules.md` if it exists (agent guidance)
3. Read `plans/index.aps.md` if it exists (current plan)
4. Check `plans/modules/` for existing module specs
5. Check for any work items with Ready or In Progress status

### Step 2: Bootstrap if needed

**If no `plans/` directory exists**, scaffold the APS structure before
continuing:

1. Check if `scaffold/init.sh` exists in this repo — if so, run it:
   `bash scaffold/init.sh .`
2. Otherwise, create the structure manually:
   - `mkdir -p plans/modules plans/execution plans/decisions`
   - Create `plans/index.aps.md` with an Index template
   - Create `plans/aps-rules.md` with agent guidance
3. Tell the user what was created and ask what they're building

If hooks aren't installed, suggest:
`./aps-planning/scripts/install-hooks.sh`

### Step 3: Report what you found

Tell the user:

- Whether APS is already set up in this project
- What plans/modules exist and their current status
- What work items are in progress or ready
- What the logical next step is

### Step 4: Help plan

Based on what the user needs:

**If plans exist but are empty:** Ask what they're building and help create the
right spec. Use the template picker:

| Situation                 | Template        |
| ------------------------- | --------------- |
| Quick feature (1-3 items) | Simple spec     |
| Module with boundaries    | Module spec     |
| Multi-module initiative   | Index + Modules |

**If plans exist but need work items:** Help define work items with Intent,
Expected Outcome, and Validation fields.

**If work items are Ready:** Ask which to start executing. Create an Action Plan
if the work item is complex.

**If work items are In Progress:** Pick up where the last session left off.
Re-read the work item and continue.

### Step 5: Create or update files

Write APS files to `plans/` following the naming conventions:

- Index: `plans/index.aps.md`
- Modules: `plans/modules/NN-name.aps.md`
- Action plans: `plans/execution/ID.actions.md`
- Agent rules: `plans/aps-rules.md`

### Reminders

- Specs describe **intent**, not implementation
- Work items need: Intent, Expected Outcome, Validation
- Checkpoints are max 12 words, no implementation detail
- Run `./bin/aps lint` to validate if available
- Update specs as you work, not after
