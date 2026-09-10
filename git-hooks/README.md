# git-hooks — local quality & safety gates

These hooks run on your **local** clone to catch issues before they reach
CI. They are **friction-reduction + prevention**, not security — every
hook can be bypassed with `--no-verify`. The hard enforcement gates are
branch protection rules and CI on the remote.

## Hooks

### `commit-msg`

Enforces [conventional commits](https://www.conventionalcommits.org/):

```
type(scope): subject
```

Allowed types: `feat`, `fix`, `chore`, `docs`, `refactor`, `test`.

Merge commits and empty messages (aborted commits) are always permitted.

### `pre-commit`

Scans staged content for secrets using **gitleaks**.

- If gitleaks is **not installed**, the commit is **blocked** with install
  instructions (fail-closed).
- If gitleaks **is** installed, it runs against staged files using the
  project config at `.gitleaks/setmeup.toml`.

### `pre-push`

Runs the full **done-gate** before allowing a push:

1. `cargo fmt --check`
2. `clippy --all-targets --all-features -- -D warnings`
3. `cargo test`

When `Cargo.toml` is absent (e.g. non-Rust repos), the hook skips
silently.

## Installation

```sh
scripts/install-hooks.sh
```

Sets `core.hooksPath` to `git-hooks/` (repo-local config) and ensures all
hooks are executable.

## Verification

```sh
scripts/install-hooks.sh --verify
scripts/verify-hooks.sh
```

Both exit 0 when hooks are correctly installed, non-zero otherwise.
