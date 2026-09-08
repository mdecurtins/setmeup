## ADDED Requirements

### Requirement: Issues seed OpenSpec changes
Every actionable GitHub issue SHALL seed (or be referenced by) an OpenSpec change, and the change SHALL reference the issue it resolves.

#### Scenario: New feature request issue
- **WHEN** an actionable feature issue is filed
- **THEN** an OpenSpec change is created (or updated) for it, and the change's proposal references the issue number

#### Scenario: Bug report issue
- **WHEN** an actionable bug issue is filed
- **THEN** the resolving change's proposal (or design) references the issue number

### Requirement: Issue templates catalogued
The repository SHALL provide issue templates for at least the types: bug, feature, and change, so issue reporters and agents file consistent issues.

#### Scenario: Bug template available
- **WHEN** a user opens a new issue
- **THEN** a bug issue template is offered

#### Scenario: Feature template available
- **WHEN** a user opens a new issue for a feature
- **THEN** a feature issue template is offered

### Requirement: Label taxonomy maintained
The repository SHALL maintain a small set of issue labels covering at least enhancement, bug, dev-workflow, and security, with the taxonomy used consistently on issues.

#### Scenario: Bug labeled
- **WHEN** a bug issue is triaged
- **THEN** it is labeled `bug` (and `security` if it touches the trust boundary)

#### Scenario: Move/process labeled
- **WHEN** a workflow or governance issue is filed
- **THEN** it is labeled `dev-workflow`

### Requirement: Merge closes the issue
The PR that lands a change SHALL close the issue(s) the change resolves, completing the issue → change → PR → merge trace.

#### Scenario: PR merges
- **WHEN** a PR that references a change resolving an issue is merged
- **THEN** the referenced issue is closed