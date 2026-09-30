# Repository context

APS combines a portable Markdown planning format with optional tooling.
This describes this repository, not a consumer project or new work authority.

## Start here

- [README](README.md): value, installation, and format
- [Getting started](docs/getting-started.md): disposable CLI walkthrough
- [Architecture](docs/architecture.md): source map, data flow, and limitations
- [AGENTS.md](AGENTS.md): collaboration and plan-update rules
- [CONTRIBUTING](CONTRIBUTING.md): public work candidates and local checks
- [Current plans](plans/index.aps.md): statuses, dependencies, and decisions

## Sources of truth

The format is expressed through templates/, plans/aps-rules.md, examples,
and documented lint behaviour. The native CLI is in cli/src/; bash and
PowerShell peers are in bin/ and lib/. Shared parity fixtures live in
test/fixtures/. Behaviour changes account for all three implementations
under D-039; release-command parity debt is tracked by REL-005.

scaffold/ is the installation payload, including skills and neutral agent
cores. Generated agent variants are reviewable install-time envelopes.
See [agent documentation](docs/agents.md) before editing them. The top-level
aps-planning/ tree is not a substitute for checking which scaffold assets
the installer and native binary actually consume.

## Context ownership

Consumer plans/project-context.md is user-owned project knowledge. Its
[scaffold template](scaffold/plans/project-context.md) is not this repo's context.
Generated .aps/context/ briefs can be regenerated; they are not durable
decisions or claim leases. Keep credentials and private consumer material
out of public plans, examples, and context packages.

## Native platforms

Windows is a first-class native CLI platform, not a WSL workflow. Start with
[PowerShell setup](docs/installation.md#windows-details) and
[Windows contributor checks](CONTRIBUTING.md#windows-native-powershell).
The shipped Windows GNU zip is exercised by CI under PowerShell 7 and Windows
PowerShell 5.1. This does not imply every fallback script or terminal has the
same coverage; interactive Warp verification remains CIB-010.

## Scope and authority

Current work lives in plans/modules/; release history lives in plans/releases/
and plans/completed.aps.md. Historical proposals under docs/plans/ are not
automatically accepted architecture. Examples illustrate other projects and
do not authorise implementing those applications here.

No hosted APS service or mandatory agent harness is needed. Tooling does write
files, and optional audit/hooks can execute commands: review untrusted plans
before running them. Disclosure and community routes are in
[SECURITY](SECURITY.md) and [the code of conduct](CODE_OF_CONDUCT.md).
