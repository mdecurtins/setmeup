# branch-protection Specification

## Purpose
Define the branch protection rules enforced on the `main` branch to mechanically prevent direct pushes, force-pushes, and unreviewed merges.

## Requirements

### Requirement: Branch protection on main
The repository SHALL enforce branch protection on the `main` branch such that direct pushes, force-pushes, and unreviewed merges are mechanically prevented.

#### Scenario: Direct push to main attempted
- **WHEN** a contributor attempts a direct push to `main`
- **THEN** the push is rejected by branch protection
- **THEN** the change must be developed on a branch and merged via PR

#### Scenario: Force-push to main attempted
- **WHEN** a contributor attempts a force-push to `main`
- **THEN** the force-push is rejected by branch protection

#### Scenario: Merge without required checks
- **WHEN** a PR targets `main` and required status checks have not passed
- **THEN** the merge is blocked until the required checks pass

### Requirement: Required status checks enforced
The `main` branch SHALL require the coverage, deps, shell, and secrets CI jobs to pass before a PR can merge. The **agent-review** check is the adversarial review: it performs the same function as a PR approval (a PR cannot merge until the review passes) but is enforced as a *status check* rather than a review event, so no second GitHub identity is required. It runs from the default branch via `pull_request_target` so the PR cannot modify its own reviewer. **As of this archive, agent-review is pending provisioning (task 7.4 — maintainer sets `AGENT_REVIEWER_ENABLED` + the OpenRouter key, then adds it to the required-checks list); until then the four checks remain the required gate and agent-review is dormant behind `AGENT_REVIEWER_ENABLED == 'true'`.**

#### Scenario: Check fails
- **WHEN** any required status check (coverage, deps, shell, secrets) fails on a PR
- **THEN** the PR is not mergeable until the check is fixed and re-passes

#### Scenario: Non-gating CI job fails
- **WHEN** a CI job that is not a required check (e.g., an OS-quality lane) fails
- **THEN** the failure is visible but does not block the merge gate

#### Scenario: Agent review passes
- **WHEN** the adversarial agent review finds no blocking issues (once provisioned per task 7.4)
- **THEN** the `agent-review` check reports green and the PR is mergeable (subject to the other gates)

#### Scenario: Agent review rejects
- **WHEN** the adversarial agent review finds blocking issues (or cannot run)
- **THEN** the `agent-review` check reports red and the merge is blocked until the issues are addressed and the review passes

### Requirement: Linear history via squash-only merges
The repository SHALL restrict merge methods to squash-only so that merged history is linear and each merge produces a single focused commit.

#### Scenario: PR merge
- **WHEN** a PR is merged
- **THEN** its commits are squashed into a single commit on `main`

#### Scenario: Other merge methods
- **WHEN** a contributor examines available merge options
- **THEN** merge and rebase merge methods are disabled; only squash is available

### Requirement: Branch up-to-date before merge
The `main` branch SHALL require that a PR branch is up-to-date with `main` before it can be merged.

#### Scenario: Branch behind main
- **WHEN** a PR branch is behind `main`
- **THEN** the PR is not mergeable until it is rebased/updated onto current `main`

#### Scenario: Branch current
- **WHEN** a PR branch is up-to-date with `main`
- **THEN** the PR is mergeable (subject to other gates)

### Requirement: Auto-delete merged branches
The repository SHALL automatically delete a branch after its PR is merged.

#### Scenario: PR merged
- **WHEN** a PR targeting `main` is merged
- **THEN** the source branch is automatically deleted
