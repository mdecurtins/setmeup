# workflow-docs (delta) — modified by dev-hooks

## MODIFIED Requirements

### Requirement: Reviewer-facing workflow documentation
The repository SHALL include `docs/workflow.md` that explains the AI-driven development workflow for this project, grounded in the repository's real artifacts. **The workflow documentation SHALL describe the local git hooks (commit-msg, pre-commit, pre-push) and their install wiring, so the local prevent loop is part of the documented development story.**

#### Scenario: Reviewer reads the workflow
- **WHEN** a reviewer opens `docs/workflow.md`
- **THEN** it explains how the project is developed with AI agents and points to real artifacts (OpenSpec changes, skills, CI) as evidence

#### Scenario: Claims are grounded
- **WHEN** the workflow doc makes a process claim
- **THEN** the claim is verifiable against actual repository artifacts, not invented process

#### Scenario: Hooks documented
- **WHEN** a reviewer reads `docs/workflow.md`
- **THEN** it describes the local git hooks (commit-msg, pre-commit, pre-push) and how they are installed and kept active