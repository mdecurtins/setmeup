# AGENTS.md — setmeup development contract

This is a public repository. It is developed primarily by AI agents, with human oversight, under an OpenSpec spec-driven workflow. This file defines how agents work here. Read it before doing anything.

## Trust boundary (non-negotiable)

**Nothing secret ever enters git.**

- This repo is **public**. Anyone can read it.
- Credentials, tokens, and key material live **only** in the vault (age-encrypted file / OS keyring) and non-git backup paths (`/mnt/c/Users/<you>/.setmeup/backups/` on WSL2).
- The manifest and dotfiles are public — never include secrets in them, not even "for example."
- Enforced by architecture and by CI (gitleaks secret scan); treated as a correctness boundary, not a convention.

## Edit discipline

- **OpenSpec changes are the vehicle for development.** Do not write code before a change exists.
- Every actionable issue seeds an OpenSpec change; every change ships as a branch + PR; no direct-to-main.
- Work follows: explore → change → implement → review. Default to understanding before writing.
- Explore-first default: use `/opsx-explore` before writing code; ask before guessing. If a task is unclear, pause and ask.

## Issues & labels

- Issues drive changes. Open with `/opsx-propose` only after an issue captures the work; the change's proposal references the issue number.
- The `github-issues` skill is the entry point for issue-driven work: create issues from templates, apply the label taxonomy, and close issues when their PR merges.
- Issue → change → PR → merge trace: an issue seeds the change; the change's proposal references the issue; the PR references both the change and the issue; merging the PR closes the issue. No open issues, unarchived changes, or dangling artifacts after a branch merges.
- Label taxonomy (small, by design):
  - `bug` — something isn't working
  - `enhancement` — new feature or capability
  - `dev-workflow` — development workflow or governance work (templates, CI, process)
  - `security` — touches the trust boundary (secrets, keys, credentials, git hygiene)
- A bug that touches the trust boundary is labeled both `bug` and `security`.
- Branch naming: `feat/<issue-number>-<change-slug>` (e.g. `feat/1-dev-governance`). The branch references the issue that seeds the change.

## What "done" means

A task or change is complete only when ALL of the following hold:

- `cargo fmt --check` is clean
- `clippy -D warnings` is clean
- `cargo test` is green
- the change's spec scenarios are covered by tests

Anything declared "done" without this gate is not done.

## Local git hooks

The repo pins local git hooks in `git-hooks/`, activated per-clone by `scripts/install-hooks.sh`. A single explicit command (`scripts/install-hooks.sh`) sets `core.hooksPath` and activates them — no per-machine manual copying. Each hook enforces one thing:

- `git-hooks/commit-msg` — Conventional Commits (allowed types: `feat|fix|chore|docs|refactor|test`, optional scope).
- `git-hooks/pre-commit` — gitleaks staged scan via `.gitleaks/setmeup.toml`; **fails closed** when gitleaks is absent (blocks the commit with install instructions).
- `git-hooks/pre-push` — done-gate preflight (`cargo fmt --check`, `clippy --all-targets --all-features -- -D warnings`, `cargo test`) when `Cargo.toml` exists.

Install/verify: `scripts/install-hooks.sh` (idempotent, re-runnable, `--verify` flag); `scripts/verify-hooks.sh`.

**Framing: friction-reduction + prevention, not security.** Bypass is trivially possible (`git commit --no-verify` / `git push --no-verify`). The hard gates are branch protection and CI required checks. Bypass is an accepted overridable escape hatch, not a secret path.

**Trust boundary:** Hooks never affect the delivery bootstrap. `bootstrap.sh` never executes any file from the working directory and never installs hooks — hook installation is purely an explicit developer action.

## Global engineering principles (compose, don't duplicate)

The shared engineering principles from `~/.config/opencode/AGENTS.md` apply here as read-only reference: SOLID, YAGNI, testing discipline (everything testable must be tested), idempotency, minimal dependencies, exact version pinning, conventional commits, focused/small commits, and security hygiene. Do not rewrite them here — cite them as governing.

- **Exact dependency pinning:** application dependencies in `Cargo.toml` use exact versions (no `^`/`~` ranges); never use a dependency released less than 7 days ago (supply-chain hygiene). Enforced in review and by the `dev-governance` dependency gate.

## Repo navigation

- `openspec/changes/` — the OpenSpec changes (product + workflow + governance)
- `openspec/config.yaml` — project context injected into artifact generation
- `.opencode/skills/` — project skills (curated); a skill must pay rent or it is removed
- `docs/workflow.md` — the reviewer-facing story of how this project is developed
- CI behavior lives in `.github/workflows/ci.yml`

## Self-maintenance

This contract is a live tool. If a line stops paying rent (maps to no task, verification, or enforcement), remove it. If the workflow materially changes, update `docs/workflow.md` and this file together.