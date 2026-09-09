# ci-pipeline Specification

## Purpose
TBD - created by archiving change dev-workflow. Update Purpose after archive.
## Requirements
### Requirement: CI enforces the contract
The repository SHALL provide a GitHub Actions workflow that runs formatting check, lint with warnings denied, and the test suite, and gates PRs/pushes on their success. **The coverage, deps, shell, secrets, and agent-review jobs SHALL be enforced as required status checks on the `main` branch, so the merge gate is mechanically enforced rather than aspirational. The `agent-review` job is the adversarial review: it runs from the default branch on `pull_request_target`, reads the PR diff and change artifacts against the review checklist via the GitHub API, and reports a pass/fail check run, functioning as the approval gate without a review event.**

#### Scenario: Push or PR to trunk
- **WHEN** code is pushed or a PR targets the main branch
- **THEN** CI runs fmt check, clippy with `-D warnings`, the test suite, and the agent review (once `agent-review` is enabled/provisioned and added to the required checks)

#### Scenario: Enforcement failure blocks
- **WHEN** any required CI gate (coverage, deps, shell, secrets, agent-review) fails
- **THEN** CI reports failure and the change is not considered mergeable

#### Scenario: Merge requires required checks
- **WHEN** a PR targeting main has not passed all required status checks (coverage, deps, shell, secrets, agent-review)
- **THEN** branch protection blocks the merge until they pass

### Requirement: Secret scan guards the trust boundary
The workflow SHALL run a secret scan on the repository content and fail when a credential-like pattern is detected in tracked content.

#### Scenario: Secret-like token in diff
- **WHEN** a commit or PR introduces a string matching a known secret pattern (e.g., an API key)
- **THEN** the scan flags it and CI fails

#### Scenario: Legitimate scan false positive
- **WHEN** the scan flags a non-secret that is a documented false positive (test fixture, documentation example)
- **THEN** the triage is handled via an explicit allowlist that is itself reviewed, keeping CI green without weakening the gate

### Requirement: Maintained OS matrix
The workflow SHALL run on at least the Linux platform, with macOS included and native Windows added deliberately when native-Windows work begins.

#### Scenario: Cross-platform baseline
- **WHEN** CI runs
- **THEN** it executes on the maintained platform set (Linux floor, plus configured additional runners)

### Requirement: Lean enforcement
The workflow SHALL be lean — no pipeline farm; each CI step maps to an enforcement the contract needs.

#### Scenario: CI step justification
- **WHEN** a reviewer inspects the workflow
- **THEN** each step serves a stated enforcement purpose (format, lint, test, secret scan)

