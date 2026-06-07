# Security Policy

eddacraft takes security reports seriously. This policy explains which versions
of anvil-plan-spec are supported and how to report a vulnerability responsibly.

## Supported Versions

anvil-plan-spec is an open-source specification and tooling repository. Security
fixes are provided for the latest public release and the current `main` branch.

| Version / Branch | Supported |
| --- | --- |
| Latest public release | Yes |
| `main` | Yes |
| Older releases | Best effort |
| Unreleased experimental branches | No |

Users should upgrade to the latest public release when a security update is
published.

## Reporting a Vulnerability

Please do not report security vulnerabilities through public GitHub issues,
discussions, or pull requests.

To report a vulnerability, use one of the following channels:

- GitHub private vulnerability reporting, if enabled for this repository
- Email: `security@eddacraft.ai`

Please include as much detail as you can safely provide:

- Affected anvil-plan-spec version, commit, or branch
- Affected specification rule, CLI command, scaffold script, template, or
  validation path
- Description of the vulnerability and likely impact
- Steps to reproduce or proof of concept
- Relevant operating system, shell, configuration, plan file, logs, and
  environment details
- Whether the issue is already public or known to be exploited

Because anvil-plan-spec is open source, source-level details are welcome in
private reports. Please keep exploit details and patches private until we have
triaged the report and agreed a disclosure path.

## Scope

This policy covers vulnerabilities in the anvil-plan-spec source, specification,
tooling, and published artefacts, including:

- Scaffold, installer, and shell/PowerShell scripts
- CLI validation and linting behaviour
- Template generation and file-writing paths
- Specification rules that could cause unsafe automation behaviour
- Documentation that could cause users or agents to configure unsafe workflows

The following are generally out of scope unless they demonstrate a clear
security impact:

- Specification disagreements that do not create a security risk
- Vulnerabilities in shell, Git, package managers, or agent tools that do not
  require an anvil-plan-spec-specific fix
- Dependency version reports without a reachable exploit path
- Denial-of-service claims without practical impact
- Social engineering or physical attacks
- Issues requiring compromised developer machines, leaked credentials, or
  malicious maintainers
- Automated scanner output without validation

## What To Expect

We aim to acknowledge valid reports within 3 business days.

After acknowledgement, we will triage the report and may ask for additional
information. For accepted vulnerabilities, we will work on a fix, publish an
updated release where appropriate, and credit the reporter if they want to be
credited.

For declined reports, we will explain the reason where it is safe and practical
to do so.

We aim to provide status updates at least every 14 days while an accepted report
remains unresolved.

## Coordinated Disclosure

Please give us a reasonable opportunity to investigate and fix the issue before
publishing details publicly.

We will not ask you to keep a vulnerability confidential forever, but we do ask
that disclosure timing be coordinated to reduce harm to users.

## Safe Harbour

We will not pursue legal action against good-faith security research that:

- Avoids privacy violations, data destruction, service disruption, or
  unauthorised access to third-party systems
- Uses only the minimum access necessary to demonstrate the issue
- Reports the vulnerability promptly and privately
- Does not use the vulnerability for extortion, persistence, or lateral
  movement

This safe harbour does not authorise testing against systems, accounts, data, or
infrastructure you do not own or do not have permission to test.

## Secrets and Sensitive Data

If you accidentally discover secrets, tokens, private keys, credentials, or
sensitive data, stop testing and report the issue immediately. Do not copy,
reuse, disclose, or retain sensitive material beyond what is necessary to
demonstrate the finding.
