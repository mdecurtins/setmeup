# ci-pipeline Specification

## Purpose
TBD - created by archiving change dev-workflow. Update Purpose after archive.
## Requirements
### Requirement: CI enforces the contract
The repository SHALL provide a GitHub Actions workflow that runs formatting check, lint with warnings denied, and the test suite, and gates PRs/pushes on their success.

#### Scenario: Push or PR to trunk
- **WHEN** code is pushed or a PR targets the main branch
- **THEN** CI runs fmt check, clippy with `-D warnings`, and the test suite

#### Scenario: Enforcement failure blocks
- **WHEN** any of fmt check, clippy, or tests fail
- **THEN** CI reports failure and the change is not considered mergeable

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

