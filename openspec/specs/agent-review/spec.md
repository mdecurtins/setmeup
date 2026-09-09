# agent-review Specification

## Purpose
Define the adversarial agent review gate: an automated reviewer that runs from trusted default-branch code, reviews the complete PR evidence, and acts as the merge gate (approval-equivalent) without a GitHub review event.

## Requirements

### Requirement: Adversarial agent review as the merge gate
Each PR to the `main` branch SHALL be adversarially reviewed by an automated agent before merge, enforced as a required status check named `agent-review` rather than a review event (a personal repo forbids authors approving their own PRs and offers no org-only bypass). The check's exit code IS the gate: an approve verdict exits 0 (check green, mergeable); a reject verdict or any fail-closed condition exits 1 (check red, merge blocked) plus a verdict comment.

**The review SHALL be performed from TRUSTED code:** the workflow runs on `pull_request_target` and checks out the base (default) branch SHA, never the PR head, so a PR cannot modify the reviewer workflow/script or reach the secrets they use. PR evidence (diff, files, change artifacts) is read via the GitHub API only.

#### Scenario: PR passes review
- **WHEN** the agent review finds no blocking issues
- **THEN** the `agent-review` check reports green and the PR is eligible for merge

#### Scenario: PR fails review
- **WHEN** the agent review finds a blocking issue
- **THEN** the check reports red, a verdict comment is posted, and the merge is blocked until the issue is fixed and re-reviewed

#### Scenario: Evidence is incomplete
- **WHEN** the PR diff or change artifacts exceed the reviewer's evidence caps (and truncation is detected)
- **THEN** the review fails closed (check red) rather than approving on partial evidence

#### Scenario: Reviewer cannot run
- **WHEN** the review secret is missing, the LLM verdict is malformed, or the reviewer throws an exception
- **THEN** the check fails closed (check red, merge blocked); there is no quiet success

### Requirement: Evidence completeness
The reviewer SHALL review the complete PR evidence: the full diff (failing closed if it exceeds the diff cap), the full change artifacts (proposal.md/tasks.md; failing closed if truncated), and the file list with change statuses (including renames' `previous_filename`), so renames and security-sensitive path changes are visible to the gate.

#### Scenario: Rename in the PR
- **WHEN** a PR contains a file rename or move
- **THEN** the rename is reported with its previous filename and is not invisible to the reviewer