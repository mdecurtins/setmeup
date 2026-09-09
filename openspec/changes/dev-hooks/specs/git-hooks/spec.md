## ADDED Requirements

### Requirement: Repository-pinned git hooks
The repository SHALL pin and commit its git hooks in a `git-hooks/` directory and, via the install step, register them as the repository hooks path (`core.hooksPath`), so a fresh clone activates the committed hooks with a single install command instead of manual per-machine setup.

#### Scenario: Fresh clone
- **WHEN** a contributor clones the repository and runs `scripts/install-hooks.sh` (or `bootstrap.sh` in a dev-clone context)
- **THEN** the clone's `core.hooksPath` points to the committed `git-hooks/` directory
- **THEN** hooks are active without manual copying into `.git/hooks` or per-machine configuration

#### Scenario: Hook update
- **WHEN** hooks are changed and committed
- **THEN** checkouts that have activated hooks pick up the new hooks on the next hook-triggering action (the hooks are read from the committed directory on each run); checkouts that have not run the install step continue without local hooks until they do

### Requirement: Conventional commit enforcement on commit
The commit-msg hook SHALL enforce Conventional Commits with an optional scope and the configured allowed types, rejecting non-conventional messages.

#### Scenario: Non-conventional message
- **WHEN** a commit message does not match the Conventional Commits format (e.g., a message without `type:` or `type(scope):`)
- **THEN** the commit is rejected and a hint to the expected format is printed

#### Scenario: Valid message
- **WHEN** a commit message matches `type(scope): subject` with an allowed type
- **THEN** the commit is accepted

### Requirement: Secret prevention at staging time
The pre-commit hook SHALL scan staged content for secret-like patterns and reject the commit if a match is found, so a secret never enters history. **The scan runs gitleaks against `.gitleaks/setmeup.toml`; if gitleaks is not installed the hook SHALL block the commit with install instructions (fail closed), because the prevention loop must be real — push-time CI detection is too late once a secret is in history.**

#### Scenario: Secret-like string staged
- **WHEN** gitleaks is available and a contributor stages a diff containing a string matching a known secret pattern (e.g., an API key, `sk-`, token-like)
- **THEN** the pre-commit hook fails and the commit is blocked

#### Scenario: gitleaks unavailable
- **WHEN** gitleaks is not installed and a contributor attempts a commit
- **THEN** the pre-commit hook blocks the commit and prints install instructions (fail closed)
- **THEN** the contributing developer installs gitleaks (from the printed one-liner) and retries the commit

#### Scenario: Clean staged content
- **WHEN** a contributor stages a diff with no secret-like pattern
- **THEN** pre-commit passes and the commit proceeds

### Requirement: Done-gate preflight on push
The pre-push hook SHALL run the done-gate preflight before allowing a push, at minimum: formatting check, lint with warnings denied, and the test suite when a Rust crate is present.

#### Scenario: Failing gate on push
- **WHEN** a contributor pushes a branch where fmt, clippy, or tests fail
- **THEN** the push is blocked and the failure is reported

#### Scenario: Passing gate
- **WHEN** a contributor pushes a branch where fmt, clippy, and tests all pass
- **THEN** the push proceeds

### Requirement: Hook installation wiring
The hooks SHALL be installable via `scripts/install-hooks.sh` and SHALL be wired into a dev-time install/verify path, so hooks are present in development and kept in place. **The product-delivery bootstrap SHALL NOT install hooks or execute any file from the caller's working directory — hook installation is an explicit developer action.**

#### Scenario: Dev-time install
- **WHEN** a developer runs the dev-time hook install/verify step
- **THEN** the hooks are installed and verified active

#### Scenario: Delivery-path bootstrap never installs hooks
- **WHEN** a user runs the `curl | sh` bootstrap (in any directory, including a clone or an unrelated or malicious worktree)
- **THEN** `scripts/install-hooks.sh` is not invoked and no local file is executed by the bootstrap

#### Scenario: Dev-time install
- **WHEN** a developer runs the dev-time hook install/verify step
- **THEN** the hooks are installed and verified active

## MODIFIED Requirements

### Requirement: Reviewer-facing workflow documentation
The repository SHALL include `docs/workflow.md` that explains the AI-driven development workflow for this project, grounded in the repository's real artifacts. **The workflow documentation SHALL describe the local git hooks (commit-msg, pre-commit, pre-push) and their install wiring, so the local prevent loop is part of the documented development story.**

#### Scenario: Reviewer reads the workflow
- **WHEN** a reviewer opens `docs/workflow.md`
- **THEN** it explains how the project is developed with AI agents and points to real artifacts (OpenSpec changes, skills, CI) as evidence

#### Scenario: Claims are grounded
- **WHEN** the workflow doc makes a process claim
- **THEN** the claim is verifiable against actual repository artifacts, not invented process

#### Scenario: Hooks documented
- **WHEN** a reviewer reads `docs/workflow.md`
- **THEN** it describes the local git hooks and how they are installed and kept active

## MODIFIED Requirements

### Requirement: Contract encodes edit discipline
The contract SHALL direct that OpenSpec changes are the vehicle for development, and that code is not written before specs exist. **The contract SHALL also record that the local git hooks enforce conventional commits, secret prevention, and the done-gate, so agents and humans alike are held to the same local gates.**

#### Scenario: New capability
- **WHEN** an agent wants to build a new feature or capability
- **THEN** the contract directs creating or updating an OpenSpec change first

#### Scenario: Secret-handling rule
- **WHEN** an agent performs any credential or secret operation in this repo
- **THEN** the contract directs that secret material is written only to non-git vault or backup paths

#### Scenario: Local hook enforcement
- **WHEN** an agent or human attempts a commit or push that violates a local hook gate
- **THEN** the hook blocks it locally, per the contract