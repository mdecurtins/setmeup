## Why

Zero git hooks exist in the repo or on this machine. Fresh clones get none (per-clone `.git/hooks` state), conventional commits are documented but unenforced, and gitleaks only runs at push-time in CI — by which point a secret is already in git history. The local prevent loop is missing entirely.

## What Changes

- Add a committed `git-hooks/` directory with hooks, activated per-clone via `core.hooksPath` so hooks are versioned and shared
- `commit-msg`: enforce Conventional Commits (`feat:`, `fix:`, `chore:`, `docs:`, `refactor:`, `test:` with optional scope)
- `pre-commit`: reject staged secret-like content before it enters history — gitleaks scan against `.gitleaks/setmeup.toml`, fail closed (block with install instructions) when gitleaks is absent
- `pre-push`: run the done-gate preflight (fmt, clippy `-D warnings`, tests when a crate is present) before a push
- Add `scripts/install-hooks.sh` — installed by an explicit developer action (dev-time install/verify step); the `curl | sh` product-delivery bootstrap never executes local files or installs hooks (trust boundary)
- Document the hooks in `AGENTS.md` and `docs/workflow.md`

## Capabilities

### New Capabilities
- `git-hooks`: repo-pinned local git hooks that enforce commit message convention, secret prevention at staging time, and the done-gate before push

### Modified Capabilities
- `bootstrap`: remains free of provisioning logic; the delivery path explicitly never executes local files (no hook auto-install)
- `workflow-docs`: `docs/workflow.md` and `AGENTS.md` describe the hook install + behavior

## Impact

- `git-hooks/` (new, committed; activated via `core.hooksPath` set by the install script)
- `scripts/install-hooks.sh` (new)
- `AGENTS.md`, `docs/workflow.md` (documentation, including the explicit dev-time install step)
- Local developer machines (hooks become active on clone + one explicit install command)
