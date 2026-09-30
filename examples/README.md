# Worked planning examples

These directories contain plans, not runnable authentication, payment, or
companion applications. Validation commands describe checks in the imagined
consumer project; those application tests are not included here. Do not
implement example work items in the APS repository.

| Example | Plan root | Demonstrates |
| --- | --- | --- |
| [User authentication](user-auth/) | examples/user-auth | module boundaries, dependencies, design and action plan |
| [OpenCode companion](opencode-companion/) | examples/opencode-companion | multi-module product planning |
| [Team payments](team-payments/) | examples/team-payments | multiple owners and crosscutting launch concern |
| [Nested monorepo](monorepo-nested/plans/) | examples/monorepo-nested/plans | federated root and child plan trees |

From the repository root, choose your native shell.

**Windows PowerShell 5.1 / PowerShell 7**, after building:

```powershell
$Aps = (Resolve-Path -LiteralPath '.\cli\target\debug\aps.exe').Path
& $Aps lint examples
if ($LASTEXITCODE -ne 0) { throw 'Example lint failed' }
& $Aps next --plans examples/user-auth
if ($LASTEXITCODE -ne 0) { throw 'Example selection failed' }
& $Aps next --plans examples/team-payments
if ($LASTEXITCODE -ne 0) { throw 'Team example selection failed' }
```

**macOS / Linux (bash):**

```bash
./bin/aps lint examples
./cli/target/debug/aps next --plans examples/user-auth
./cli/target/debug/aps next --plans examples/team-payments
```

Build first using [CONTRIBUTING](../CONTRIBUTING.md#local-development-checks).
Lint may report review-age or missing-review warnings: example dates are
illustrative history, not a promise of current project review. Do not alter
them simply to hide warnings.

For start/complete, copy the plan into a temporary directory first and follow
the [disposable walkthrough](../docs/getting-started.md#try-the-cli-without-changing-your-project).
Keep example state transitions separate from real delivery claims. Never run
validation commands from untrusted plans without reviewing what they execute.
