# ci-gate (delta) — Modified by ci-gate-review-lessons

## MODIFIED Requirements

### Requirement: Pre-PR gate checklist for agent proposals
The change SHALL provide a reusable prompt (`.opencode/skills/ci-gate/SKILL.md`) that an agent loads before opening or revising a PR that touches OpenSpec change artifacts. The prompt SHALL include concrete, actionable checks — with grep/file-read commands — that verify the proposal against **all 11 documented rejection categories from the agent-review gate's full audit history (PRs #30, #33, #34, #35, #36, #39)**:

1. **PR body artifact linkage** — the `**Change:**` pointer and change-artifact fetch path via the GitHub API
2. **Spec/design/task internal consistency** — no contradictory claims between requirement statements and design decisions; verify with `grep` counts across all artifacts
3. **Truthful mechanism claims** — no unfulfillable promises (e.g., "zero manual steps" for per-clone config values)
4. **Delivery-path trust boundary** — the `curl | sh` bootstrap must never execute local CWD files or run `install-hooks.sh`; **enumerate all tool CWD-read surfaces** (config files, environment variables, implicit file references) before claiming a script is inert
5. **Evidence completeness + scope hygiene** — renames/missing patches surfaced; no out-of-scope changes in archive/spec-sync PRs; **confirm branch was created from main**
6. **Structured risk assessment** — PR body must contain a 4-column risk table (failure mode, likelihood, impact, mitigation); "rollback" is a recovery procedure, not a risk
7. **Shell-script edge-case testing** — every new shell script tested with: unreadable input, empty input, missing argument, expected-failure commands with non-zero exit
8. **Base-ref artifact mismatch** — when `openspec/changes/` is modified, the PR body must include an evidence table with per-task probe results (the gate reads tasks.md/proposal.md from the base ref)
9. **Dependency regression assessment** — changes touching `cargo install`, network calls, PATH, or environment variables must assess regression risk (network failure, toolchain incompatibility, PATH resolution)
10. **Tool-specific CWD enumeration** — before claiming a script or tool is "CWD-inert", enumerate every documented surface via `--help`, `man`, or source; verify each surface is neutralized
11. **Spec-internal consistency via grep** — for every requirement count, category count, or constraint mentioned in one artifact, grep all other artifacts and confirm the same value appears

#### Scenario: Agent loads the skill
- **WHEN** an agent loads the `ci-gate` skill before writing or modifying OpenSpec change artifacts
- **THEN** the agent reads the Pre-PR checklist and runs its verification commands against the current proposal
- **THEN** any violation in any of the 11 categories is surfaced before the PR opens

#### Scenario: Agent skips the checklist
- **WHEN** an agent opens a PR without running the `ci-gate` checklist
- **THEN** the agent-review gate may reject the PR; the skill's purpose is to prevent avoidable rejections on the first attempt

#### Scenario: Agent verifies branch origin
- **WHEN** an agent prepares a PR
- **THEN** the agent runs `git merge-base HEAD main` and confirms the branch fork point is main (not another branch or a dirty state)
- **THEN** the PR body notes the base ref vs head ref context for any changed OpenSpec artifacts

#### Scenario: Agent tests shell script edge-cases
- **WHEN** an agent creates or modifies a shell script
- **THEN** the agent tests the script with: unreadable input file (if applicable), empty input, missing argument, and expected non-zero exit paths
- **THEN** the agent documents the edge-case results in the PR body or task evidence

#### Scenario: Agent enumerates tool CWD surfaces
- **WHEN** an agent modifies a bootstrap, hook, or delivery-path script that invokes a tool (cargo, gitleaks, git, etc.)
- **THEN** the agent enumerates every way that tool reads the working directory using `--help`, `man`, or source documentation
- **THEN** each surface is either neutralized or documented as an accepted risk

#### Scenario: Agent writes risk assessment
- **WHEN** an agent prepares the PR body
- **THEN** the agent includes a risk assessment section with a 4-column table: Failure mode, Likelihood, Impact, Mitigation
- **THEN** each entry is a concrete failure mode (not "rollback" or "revert")

### Requirement: Cross-reference from authoring and review skills
The `ci-gate` skill SHALL be referenced from the `openspec-propose` and `pr-review` skills so agents encounter it at both artifact-writing and review-submission time.

#### Scenario: Agent writes artifacts
- **WHEN** an agent completes all `applyRequires` artifacts via `openspec-propose`
- **THEN** the `openspec-propose` skill's instructions reference loading the `ci-gate` skill before declaring the proposal ready

#### Scenario: Agent reviews a PR
- **WHEN** an agent performs a PR review via the `pr-review` skill
- **THEN** the `pr-review` skill's rules reference loading the `ci-gate` skill before completing the review

## ADDED Requirements

### Requirement: Branch origin discipline during development
The development contract (AGENTS.md) SHALL record that branches are always created from main: `git checkout -b <branch> main`. The ci-gate checklist SHALL verify the branch originated from main before opening a PR.

#### Scenario: Branch created from main
- **WHEN** a developer creates a new branch
- **THEN** `git checkout -b <name> main` is the only valid creation command
- **THEN** a branch created from a dirty state or another branch is a spec violation

#### Scenario: Pre-PR origin check
- **WHEN** an agent runs the ci-gate checklist before opening a PR
- **THEN** `git merge-base HEAD main` is checked; if the fork point is not main (i.e., the branch was created from anything other than main), the agent catches it before the PR opens
