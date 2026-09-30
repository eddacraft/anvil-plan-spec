# Contributing to Anvil Plan Spec

Thank you for your interest in contributing to APS! This document provides
guidelines for contributing to the project.

## Pull Request Process

1. **Open an issue first** for significant changes to discuss approach
2. **Create a feature branch** from `main`
3. **Update documentation** if behaviour changes
4. **Keep PRs focused** on one logical change per PR
5. **Ensure linting passes** before requesting review (`npx markdownlint-cli "**/*.md"`)
   - CI will automatically run markdown linting on all PRs
   - Fix any linting errors before requesting review

### Commit Messages

Use clear, descriptive commit messages:

```text
Feat: Add steps template for granular execution

Steps translate task intent into ordered, observable actions.
Each step has a checkpoint for verification.

Closes #12
```

## Plan Updates

This repo dogfoods APS — the roadmap lives in
[plans/index.aps.md](plans/index.aps.md) and module specs under
`plans/modules/`. Treat plan files like code: if your change affects what
the plans describe, update them **in the same PR**.

**A plan update is required when your change touches:**

- Templates, prompts, or examples
- Installer or scaffold behaviour
- Validation (lint/audit) behaviour
- Any in-flight work item's scope or status

**Marking status:** use the CLI (`./bin/aps start <ID>`,
`./bin/aps complete <ID>`) or hand-edit the `- **Status:**` field
(`Draft → Ready → In Progress → Complete`). Add a `Results:` line when
completing non-trivial items, and log discoveries in `plans/issues.md`.

**Validation before requesting review:** use the
[Windows native checks](#windows-native-powershell) or the Unix summary below.

```bash
./bin/aps lint plans              # plan structure
./test/run.sh                     # CLI test suite
npx markdownlint-cli "**/*.md"    # markdown style (CI-enforced)
```

See [AGENTS.md](AGENTS.md) → "Keeping the plans honest" for the full
conventions, and [docs/workflow.md](docs/workflow.md) for the lifecycle.

## Finding bounded public work

Start with [the roadmap](plans/index.aps.md) and
the linked module's status, dependencies, and validation. Run
`./bin/aps next`; an empty queue means no work item is currently Ready, not
that the repository has no useful work. Draft design work is not implementation
authority. Check open PRs and agree a bounded slice with a maintainer before
starting an already In Progress item.

Current public-only contribution candidates:

- **REL-005 fixture extraction:** move the synthetic pre-sweep corpus from
  `cli/src/release.rs` into shared `test/fixtures/release-close/` files and
  make the Rust tests consume those exact bytes. Preserve dry-run non-mutation,
  apply idempotence, lifecycle guards, and existing output. Keep this separate
  from the bash/PowerShell ports; do not use live plan files as mutation fixtures.
- **CIB-010 Windows verification:** on native Windows PowerShell in Warp, build
  the current CLI, run interactive `aps setup` in a disposable directory, and
  record whether one arrow press moves one option and holding/releasing keys
  causes no duplicate navigation. Include commit, Windows, Warp, and PowerShell
  versions. Unit tests alone do not close this manual gate.
- **CIB-009 toolchain decision evidence:** reproduce local-versus-CI toolchain
  drift and propose either an explicit stable-update preflight or a shared
  exact pin. A `stable` toolchain file alone does not update an already-installed
  stale stable toolchain. Implementation waits for the maintainer decision.

These slices need only this public checkout and public dependencies; no private
consumer repository, publishing token, or production service is required.
REL-005 remains In Progress until all three CLI implementations agree.
TEAM/PROG implementation and CLI-002 remain design/dependency-gated.

### Local development checks

Use Rust/Cargo, Node.js 24 or newer, and the pnpm version in `package.json`.
Run from the repository root. Windows uses the native route below; Unix shell
suites are additional CI coverage, not a Windows onboarding prerequisite.

#### Windows native PowerShell

Use Windows PowerShell 5.1 or PowerShell 7, Git for Windows, Node.js 24+ and
pnpm from package.json. A native source build also needs Rust via rustup and
its target's linker: the default MSVC toolchain needs Visual Studio C++ Build
Tools and a Windows SDK. Alternatively an explicitly selected GNU toolchain
needs a compatible MinGW linker. Installing build tools may need your device
administrator; running APS from the released zip does not.

For native Node tooling use pnpm.cmd, avoiding PowerShell's pnpm.ps1 shim if
local script policy blocks it. Do not weaken the machine's execution policy.

```powershell
$ErrorActionPreference = 'Stop'
function Invoke-Checked {
    param([scriptblock]$Command)
    $global:LASTEXITCODE = 0
    & $Command
    if ($LASTEXITCODE -ne 0) { throw "Command failed: exit $LASTEXITCODE" }
}
Invoke-Checked { pnpm.cmd install --frozen-lockfile }
Invoke-Checked { pnpm.cmd --dir mcp install --frozen-lockfile }
Invoke-Checked { pnpm.cmd lint }
Invoke-Checked { cargo fmt --manifest-path cli/Cargo.toml --check }
Invoke-Checked { cargo clippy --manifest-path cli/Cargo.toml --locked --all-targets -- -D warnings }
Invoke-Checked { cargo test --manifest-path cli/Cargo.toml --locked }
Invoke-Checked { cargo build --manifest-path cli/Cargo.toml --locked }
$Aps = (Resolve-Path -LiteralPath '.\cli\target\debug\aps.exe').Path
Invoke-Checked { & $Aps lint plans }
$PreviousApsBin = $env:APS_BIN
try {
    $env:APS_BIN = $Aps
    Invoke-Checked { pnpm.cmd --dir mcp test }
    Invoke-Checked { pnpm.cmd --dir mcp exec tsc -p . }
} finally { $env:APS_BIN = $PreviousApsBin }
```

With PowerShell 7 installed, run the fallback parity harness in its own process
so its exit cannot terminate your interactive session:

```powershell
pwsh -NoProfile -File .\test\ps-parity.ps1
if ($LASTEXITCODE -ne 0) { throw 'PowerShell parity failed' }
```

The native Windows release journey in CI runs the shipped GNU zip under both
PowerShell 7 and Windows PowerShell 5.1. The fallback parity harness runs under
PowerShell 7; this is not a promise that every fallback script supports 5.1.
If local policy blocks a reviewed .ps1 test, report it and rely on CI rather
than changing policy. The bash shell suite and cross-CLI bash harness run in
Unix CI; Windows contributors need not install WSL to submit a change.
Native Windows/Warp CIB-010 manual verification remains a separate gate.

#### macOS or Linux

```bash
pnpm install --frozen-lockfile
pnpm --dir mcp install --frozen-lockfile
pnpm lint
./bin/aps lint plans
./test/run.sh
cargo fmt --manifest-path cli/Cargo.toml --check
cargo clippy --manifest-path cli/Cargo.toml --locked --all-targets -- -D warnings
cargo test --manifest-path cli/Cargo.toml --locked
cargo build --manifest-path cli/Cargo.toml --locked
APS_RUST_BIN=cli/target/debug/aps ./test/cli-parity.sh
```

For the REL-005 slice, use `cargo test --manifest-path cli/Cargo.toml --locked
release::tests` as the focused check. For CIB-010, use `cargo test
--manifest-path cli/Cargo.toml --locked setup` and the native manual check.
With PowerShell installed, also run `pwsh -NoProfile -File test/ps-parity.ps1`.
The parity harness skips its PowerShell leg when `pwsh` is absent; the shell
suite skips MCP tests without MCP dependencies. Report skips explicitly, not
as full coverage. Compare `rustc --version` and `cargo clippy --version` with
CI before diagnosing a lint disagreement.

## Scope Guardrails

APS is a specification format for planning and task authorisation.
Contributions should align with this scope.

### In Scope

- Template improvements and new templates
- Prompting guidance for AI assistants
- Examples and worked use cases
- Documentation and getting-started guides
- Tooling for validation or linting APS files

### Out of Scope

These belong to downstream implementations and will not be accepted:

- Runtime execution engines
- IDE plugins or integrations
- Project management tool integrations (Jira, Linear, etc.) **We may revisit this in the future**
- AI model fine-tuning or training data

If you're unsure whether something is in scope, open an issue to discuss
before investing time.

### Feature Requests

For net-new functionality, start with a design conversation. Open an issue
describing:

- The problem you're solving
- Your proposed approach (optional)
- Why it belongs in APS

The maintainers will help decide whether it should move forward. Please wait
for approval before opening a feature PR.

## AI-Assisted Contributions

When using AI tools to contribute:

- Follow the guidance in [AGENTS.md](AGENTS.md)
- Ensure AI-generated content is reviewed and validated

## Reporting and review context

For bugs, include the command, CLI version or commit, OS, expected/actual
output, and a minimal public plan fixture. Remove secrets and private material.
Proposals should identify the owning APS module and which CLI surfaces change.
PRs should list validation, skips, and the matching plan update; publishing
access is not required for an ordinary contribution.

Use [SECURITY.md](SECURITY.md) for private vulnerability reports, not a public
issue. Participation follows the [code of conduct](CODE_OF_CONDUCT.md).
Maintainers review scope and design decisions; a Draft proposal is not approval.
See [context](CONTEXT.md) and [architecture](docs/architecture.md) before
changing implementation boundaries.

## Questions?

Open an issue for questions about contributing or the project.

## License

By contributing, you agree that your contributions will be licensed under
the Apache-2.0 License.
