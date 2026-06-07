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

**Validation to run before requesting review:**

```bash
./bin/aps lint plans              # plan structure
./test/run.sh                     # CLI test suite
npx markdownlint-cli "**/*.md"    # markdown style (CI-enforced)
```

See [AGENTS.md](AGENTS.md) → "Keeping the plans honest" for the full
conventions, and [docs/workflow.md](docs/workflow.md) for the lifecycle.

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
- Project management tool integrations (Jira, Linear, etc.) **We may revist this in the future**
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

## Questions?

Open an issue for questions about contributing or the project.

## License

By contributing, you agree that your contributions will be licensed under
the Apache-2.0 License.
