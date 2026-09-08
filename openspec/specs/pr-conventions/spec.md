# pr-conventions Specification

## Purpose
TBD - created by archiving change dev-governance. Update Purpose after archive.
## Requirements
### Requirement: Every change ships as a branch + PR
Every code change to the repository SHALL be made on a branch and landed via pull request; direct commits to the main branch SHALL NOT be used for code changes.

#### Scenario: Code change initiated
- **WHEN** a developer or agent intends to make a code change
- **THEN** a branch is created, the change is developed on it, and a PR proposes the merge

#### Scenario: Direct commit attempted
- **WHEN** a direct commit to main is attempted for a code change
- **THEN** the PR/merge policy requires it to instead be developed on a branch and merged via PR

### Requirement: PR references its change
Each PR SHALL reference the OpenSpec change it implements (by name/path) and the issue it resolves, so the trail issue → change → PR is complete.

#### Scenario: PR body
- **WHEN** a PR is opened
- **THEN** its description names the OpenSpec change and links the issue(s) it resolves

### Requirement: Formal review gate
Each PR SHALL receive a checklist review by at least one agent and/or human before merge, covering:
1. Done-checklist satisfied (fmt, clippy, tests, spec scenarios)
2. Alignment with the change's spec/design and the issue it resolves
3. Security-sensitive paths inspected (secrets, git hygiene, identity, credentials, trust boundary)
4. Risk claim credible (what could go wrong, mitigation)

#### Scenario: PR passes review
- **WHEN** a PR's checklist review passes all four items
- **THEN** the PR is eligible for merge

#### Scenario: PR fails review item
- **WHEN** a PR fails any review checklist item
- **THEN** the PR is not merged until the item is addressed and re-reviewed

### Requirement: Review by agent or human
The review gate SHALL accept either an agent or human reviewer, since the project is developed by agents with human oversight.

#### Scenario: Agent review
- **WHEN** an agent performs the checklist review of a PR
- **THEN** the review is recorded and satisfies the gate

### Requirement: Review is a checklist, not LGTM
The review SHALL apply the substantive checklist, not a lightweight approval, so the public repo's quality bar is real.

#### Scenario: Review record
- **WHEN** a merge is reviewed
- **THEN** the review record reflects which checklist items were checked

### Requirement: Change archived as part of the branch lifecycle
The change SHALL be archived before its branch is considered closed, so the OpenSpec archive is the terminal artifact of the change, not a follow-up chore.

#### Scenario: Branch lands for a change
- **WHEN** a branch implementing a change passes all gates and is merged
- **THEN** the change is archived (delta specs synced to main specs; change dir moved to `openspec/changes/archive/`)

#### Scenario: Archive commit is the last commit for a change
- **WHEN** a change is fully implemented and merged
- **THEN** the archive commit is the final commit carrying that change's work, landing only after the done-gate, review, and issue verification all pass

### Requirement: No dangling open artifacts
The workflow SHALL NOT leave open changes, open issues referenced by a merged change, or unarchived completed changes after the relevant branch is merged.

#### Scenario: Merged change with open issue
- **WHEN** a PR that resolves an issue is merged
- **THEN** the referenced issue is closed and the change is archived, leaving no dangling artifact

#### Scenario: Completed but unarchived change
- **WHEN** a change's tasks are all complete and its branch merged
- **THEN** the change is archived; it is not left sitting in the active changes list

