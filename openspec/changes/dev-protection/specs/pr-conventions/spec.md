# pr-conventions (delta) — modified by dev-protection

## MODIFIED Requirements

### Requirement: Formal review gate
Each PR SHALL receive a checklist review by at least one agent and/or human before merge, covering:
1. Done-checklist satisfied (fmt, clippy, tests, spec scenarios)
2. Alignment with the change's spec/design and the issue it resolves
3. Security-sensitive paths inspected (secrets, git hygiene, identity, credentials, trust boundary)
4. Risk claim credible (what could go wrong, mitigation)

**The review gate SHALL be mechanically enforced on the `main` branch. Because this is a personal (user-owned) repo — where GitHub forbids authors approving their own PRs and provides no org-only bypass — enforcement is via a required status check (`agent-review`) rather than a review event. The adversarial agent review performs the same function as an approval: a PR cannot merge until the `agent-review` check is green. The check is dormant behind `AGENT_REVIEWER_ENABLED` and is added to the required-checks list once the maintainer provisions the OpenRouter key (`tasks.md` 7.4).**

#### Scenario: PR passes review
- **WHEN** a PR's checklist review passes all four items
- **THEN** the `agent-review` check reports green and the PR is eligible for merge

#### Scenario: PR fails review item
- **WHEN** a PR fails any review checklist item
- **THEN** the `agent-review` check reports red and the PR is not merged until the item is addressed and the review passes

#### Scenario: PR without passing review
- **WHEN** a PR targeting main has not passed the `agent-review` check
- **THEN** the merge is blocked by branch protection until the check is green

### Requirement: Every change ships as a branch + PR
Every code change to the repository SHALL be made on a branch and landed via pull request; direct commits to the main branch SHALL NOT be used for code changes. **Direct and force pushes to `main` SHALL also be mechanically rejected by branch protection.**

#### Scenario: Code change initiated
- **WHEN** a developer or agent intends to make a code change
- **THEN** a branch is created, the change is developed on it, and a PR proposes the merge

#### Scenario: Direct commit attempted
- **WHEN** a direct commit to main is attempted for a code change
- **THEN** branch protection rejects it; the change must be developed on a branch and merged via PR

#### Scenario: Force-push attempted
- **WHEN** a force-push to main is attempted
- **THEN** branch protection rejects it