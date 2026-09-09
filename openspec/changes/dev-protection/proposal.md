## Why

Every quality gate in this public repo is currently policy enforced only by agent discipline — `main` is unprotected (404), `rulesets` is empty, no required PR reviews, no required status checks, merge commits are allowed, and third-party GitHub Actions are pinned to tags not SHAs. Nothing mechanical stops a force-push, a direct commit, or a merge-without-review entering history.

## What Changes

- Add branch protection on `main`: require status checks (coverage, deps, shell, secrets, agent-review), require branch up-to-date, prevent force-push, enforce admins
- Enforce the review gate as a **required status check** (the adversarial `agent-review` job) rather than a review event: a PR cannot merge until the agent review passes. This sidesteps the personal-repo wall (GitHub forbids authors approving their own PRs and offers no org-only bypass), so no GitHub App or second identity is needed — the check's exit code IS the approval
- Switch merge policy to squash-only (disable merge and rebase commits) so history stays linear
- Enable auto-delete of merged branches
- SHA-pin all third-party GitHub Actions in `.github/workflows/ci.yml` (replace `@v4`, `@v2`, `@stable` tags with commit SHAs)
- Add a `CODEOWNERS` scaffold signaling that governance/trust-boundary files require awareness
- Add Dependabot (monthly cadence) with grouped dependency PRs

## Capabilities

### New Capabilities
- `branch-protection`: GitHub-side enforcement of the merge gate on `main` — required checks, adversarial review, linear (squash-only) history, no force-push
- `codeowners`: ownership signals for trust-boundary and governance paths
- `dependabot`: automated monthly dependency updates, grouped to reduce PR noise
- `agent-review`: the adversarial agent review — a required status check that reads the PR diff + change artifacts against the review checklist and gates the merge (approval-equivalent without a review event)

### Modified Capabilities
- `ci-pipeline`: the merge on `main` now *requires* the coverage, deps, shell, secrets, and agent-review jobs to pass (they currently run but are not required checks)
- `pr-conventions`: the "review gate" moves from aspirational to mechanically enforced as the required `agent-review` status check, which performs the approval function without a review event

## Impact

- `.github/workflows/ci.yml` (action pinning)
- `.github/CODEOWNERS` (new)
- Dependabot config (new, `.github/dependabot.yml`)
- GitHub repository settings (branch protection, merge button, auto-delete) — enforced via `gh api`/rulesets, not committed files
- `docs/workflow.md` and `openspec/specs/` (pr-conventions, ci-pipeline deltas) updated to describe the enforced gates
- New `agent-review` workflow + script (dormant until the OpenRouter key is provisioned)