## Why

Zero git hooks exist in the repo or on this machine. Fresh clones get none (per-clone `.git/hooks` state), conventional commits are documented but unenforced, and gitleaks only runs at push-time in CI — by which point a secret is already in git history. The local prevent loop is missing entirely.

## What Changes

- Add a committed `git-hooks/` directory with hooks, and pin it as the repo's hooks path (`core.hooksPath`) so hooks are versioned and shared
- `commit-msg`: enforce Conventional Commits (`feat:`, `fix:`, `chore:`, `docs:`, `refactor:`, `test:` with optional scope)
- `pre-commit`: reject staged secret-like content (gitleaks-style guard) before it can enter history
- `pre-push`: run the done-gate preflight (fmt, clippy `-D warnings`, tests when a crate is present) before a push
- Add `scripts/install-hooks.sh` — wired into BOTH `bootstrap.sh` (fresh-machine path) and a dev-time install/verify check
- Document the hooks in `AGENTS.md` and `docs/workflow.md`

## Capabilities

### New Capabilities
- `git-hooks`: repo-pinned local git hooks that enforce commit message convention, secret prevention at staging time, and the done-gate before push

### Modified Capabilities
- `bootstrap`: installing the hooks becomes part of bootstrapping a fresh machine
- `workflow-docs`: `docs/workflow.md` and `AGENTS.md` describe the hook install + behavior

## Impact

- `git-hooks/` (new, committed; path pinned via `core.hooksPath`)
- `scripts/install-hooks.sh` (new)
- `bootstrap.sh` (wires hook install)
- `AGENTS.md`, `docs/workflow.md` (documentation)
- Local developer machines (hooks become active on clone + install)