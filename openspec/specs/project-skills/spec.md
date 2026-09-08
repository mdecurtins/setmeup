# project-skills Specification

## Purpose
TBD - created by archiving change dev-workflow. Update Purpose after archive.
## Requirements
### Requirement: Curated project skills
The repository SHALL provide a small, curated set of project skills under `.opencode/skills/`, each with a stated purpose and each paying rent to the workflow.

#### Scenario: Skill inventory is small and purposeful
- **WHEN** an agent or human lists the project's skills
- **THEN** the set is small and each skill has a clear, defined purpose with no unused skills retained

### Requirement: Probe-before-assume skill
The workflow SHALL provide a skill that directs agents to inspect actual machine and repository state before asserting it.

#### Scenario: Environment uncertainty
- **WHEN** an agent needs to assert a fact about the machine or repo (tool presence, config state, file layout)
- **THEN** the skill directs it to inspect real state rather than assume from prior context

#### Scenario: Verification before claim
- **WHEN** an agent claims a build, test, or provisioning outcome
- **THEN** the skill directs it to verify against actual output rather than claim from memory

### Requirement: Done-checklist discipline
The workflow SHALL provide an explicit done-checklist gate that agents apply before declaring work complete.

#### Scenario: Pre-completion gate
- **WHEN** an agent is about to declare a work item complete
- **THEN** it applies the done-checklist (fmt, clippy, tests, spec scenarios) and confirms each

### Requirement: OpenSpec skills wired in
The workflow SHALL wire the existing OpenSpec skills (propose, apply, explore) into the project's skill set so spec-driven development is the default path.

#### Scenario: Spec-driven development available
- **WHEN** an agent initiates a development episode in this repo
- **THEN** the OpenSpec propose/apply/explore skills are available and referenced by the workflow

