# ci-pipeline (delta) — modified by dev-protection

## MODIFIED Requirements

### Requirement: CI enforces the contract
The repository SHALL provide a GitHub Actions workflow that runs formatting check, lint with warnings denied, and the test suite, and gates PRs/pushes on their success. **The coverage, deps, shell, and secrets jobs SHALL be enforced as required status checks on the `main` branch, so the merge gate is mechanically enforced rather than aspirational. The `agent-review` job is the adversarial review: it runs from the default branch on `pull_request_target`, reads the PR diff and change artifacts against the review checklist via the GitHub API, and reports a pass/fail check run, functioning as the approval gate without a review event. As of this archive, `agent-review` is pending provisioning (task 7.4) and dormant behind `AGENT_REVIEWER_ENABLED == 'true'`; it becomes a required check only after the maintainer provisions it and adds it to the branch protection list.**

#### Scenario: Push or PR to trunk
- **WHEN** code is pushed or a PR targets the main branch
- **THEN** CI runs fmt check, clippy with `-D warnings`, the test suite, and the agent review (once `agent-review` is enabled/provisioned and added to the required checks)

#### Scenario: Enforcement failure blocks
- **WHEN** any required CI gate (coverage, deps, shell, secrets) fails
- **THEN** CI reports failure and the change is not considered mergeable

#### Scenario: Merge requires required checks
- **WHEN** a PR targeting main has not passed all required status checks (coverage, deps, shell, secrets)
- **THEN** branch protection blocks the merge until they pass (agent-review additionally once provisioned)