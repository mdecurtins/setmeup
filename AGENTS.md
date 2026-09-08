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

## What "done" means

A task or change is complete only when ALL of the following hold:

- `cargo fmt --check` is clean
- `clippy -D warnings` is clean
- `cargo test` is green
- the change's spec scenarios are covered by tests

Anything declared "done" without this gate is not done.

## Global engineering principles (compose, don't duplicate)

The shared engineering principles from `~/.config/opencode/AGENTS.md` apply here as read-only reference: SOLID, YAGNI, testing discipline (everything testable must be tested), idempotency, minimal dependencies, exact version pinning, conventional commits, focused/small commits, and security hygiene. Do not rewrite them here — cite them as governing.

## Repo navigation

- `openspec/changes/` — the OpenSpec changes (product + workflow + governance)
- `openspec/config.yaml` — project context injected into artifact generation
- `.opencode/skills/` — project skills (curated); a skill must pay rent or it is removed
- `docs/workflow.md` — the reviewer-facing story of how this project is developed
- CI behavior lives in `.github/workflows/ci.yml`

## Self-maintenance

This contract is a live tool. If a line stops paying rent (maps to no task, verification, or enforcement), remove it. If the workflow materially changes, update `docs/workflow.md` and this file together.