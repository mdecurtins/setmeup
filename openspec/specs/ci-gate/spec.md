# ci-gate Specification

## Purpose
Provide a reusable pre-PR checklist that agents load before opening or revising a PR that touches OpenSpec change artifacts, reducing agent-review gate rejections.

## Requirements
### Requirement: Pre-PR gate checklist for agent proposals
The change SHALL provide a reusable prompt (`.opencode/skills/ci-gate/SKILL.md`) that an agent loads before opening or revising a PR that touches OpenSpec change artifacts. The prompt SHALL include concrete, actionable checks — with grep/file-read commands — that verify the proposal against **all 5 of the agent-review gate's documented rejection categories** from PRs #30 and #33:

1. **PR body artifact linkage** — the `**Change:**` pointer and change-artifact fetch path via the GitHub API
2. **Spec/design/task internal consistency** — no contradictory claims between requirement statements and design decisions
3. **Truthful mechanism claims** — no unfulfillable promises (e.g., "zero manual steps" for per-clone config values)
4. **Delivery-path trust boundary** — the `curl | sh` bootstrap must never execute local CWD files or run `install-hooks.sh`
5. **Evidence completeness + scope hygiene** — renames/missing patches surfaced; no out-of-scope changes in archive/spec-sync PRs

#### Scenario: Agent loads the skill
- **WHEN** an agent loads the `ci-gate` skill before writing or modifying OpenSpec change artifacts
- **THEN** the agent reads the Pre-PR checklist and runs its verification commands against the current proposal
- **THEN** any violation in any of the 5 categories is surfaced before the PR opens

#### Scenario: Agent skips the checklist
- **WHEN** an agent opens a PR without running the `ci-gate` checklist
- **THEN** the agent-review gate may reject the PR; the skill's purpose is to prevent avoidable rejections on the first attempt

### Requirement: Cross-reference from authoring and review skills
The `ci-gate` skill SHALL be referenced from the `openspec-propose` and `pr-review` skills so agents encounter it at both artifact-writing and review-submission time.

#### Scenario: Agent writes artifacts
- **WHEN** an agent completes all `applyRequires` artifacts via `openspec-propose`
- **THEN** the `openspec-propose` skill's instructions reference loading the `ci-gate` skill before declaring the proposal ready

#### Scenario: Agent reviews a PR
- **WHEN** an agent performs a PR review via the `pr-review` skill
- **THEN** the `pr-review` skill's rules reference loading the `ci-gate` skill before completing the review