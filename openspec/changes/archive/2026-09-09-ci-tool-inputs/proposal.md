## Why

PR #29 (a Dependabot bump of `taiki-e/install-action` in the `github-actions` group) breaks the CI `deps` job: the action's default `tool` input changed from `cargo-deny` to `cargo-llvm-cov` between pinned SHAs, and the workflow relies on that default. The "Install cargo-deny" step silently installs the wrong tool, `cargo deny check` fails with `no such command: deny`, and the `agent-review` gate fails closed downstream. See issue #31.

## What Changes

- Add an explicit `with: tool:` input to both `taiki-e/install-action` call sites in `.github/workflows/ci.yml`:
  - `deps` job → `tool: cargo-deny`
  - `coverage` job → `tool: cargo-llvm-cov`
- Behavior no longer depends on the action's changing default, so any future SHA bump (including the pending Dependabot PR #29) is safe.
- No action SHAs change in this change; only the explicit tool inputs are added.

## Capabilities

### New Capabilities

- none

### Modified Capabilities

- `ci-pipeline`: the "Lean enforcement" contract gains an explicit requirement that quality-gate tool installation declares the tool as an explicit input rather than relying on the action's default.

## Impact

- `.github/workflows/ci.yml` — two step blocks gain a `with: tool:` input.
- CI behavior is unchanged on the current `main` SHAs (both coincidentally correct today) but becomes robust against future install-action bumps.
- Fixes issue #31; supersedes the Dependabot-required follow-up for PR #29.