## ADDED Requirements

### Requirement: Reviewer-facing workflow documentation
The repository SHALL include `docs/workflow.md` that explains the AI-driven development workflow for this project, grounded in the repository's real artifacts.

#### Scenario: Reviewer reads the workflow
- **WHEN** a reviewer opens `docs/workflow.md`
- **THEN** it explains how the project is developed with AI agents and points to real artifacts (OpenSpec changes, skills, CI) as evidence

#### Scenario: Claims are grounded
- **WHEN** the workflow doc makes a process claim
- **THEN** the claim is verifiable against actual repository artifacts, not invented process

### Requirement: Documentation stays current
The workflow doc SHALL be updated when the development workflow or repository structure materially changes.

#### Scenario: Material flow change
- **WHEN** the development workflow or repo structure materially changes (e.g., new project skill, changed CI, new change convention)
- **THEN** `docs/workflow.md` is updated to match

### Requirement: README points to the workflow
The repository README SHALL reference `docs/workflow.md` and the OpenSpec change workflows so the process is discoverable.

#### Scenario: Discovery from README
- **WHEN** a reader opens the repository README
- **THEN** it links to the workflow documentation and the spec-driven development entry points