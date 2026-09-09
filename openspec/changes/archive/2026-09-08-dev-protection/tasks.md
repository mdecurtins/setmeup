## 1. SHA-pin all GitHub Actions

- [x] 1.1 Enumerate every `uses:` reference in `.github/workflows/ci.yml` and resolve each action/tag to a full commit SHA (probe-verify: get the SHA before writing)
- [x] 1.2 Rewrite `ci.yml` to pin each action to its SHA, keeping the original tag as a trailing comment (e.g. `uses: actions/checkout@<sha> # v4`)
- [x] 1.3 Verify no unpinned `@<tag>` or `@<branch>` remains in any workflow (grep all `.github/workflows/`)

## 2. Apply GitHub repository settings

- [x] 2.1 Set the default merge method to squash and disable merge/rebase commits (`gh api` repo settings: `allow_merge_commit=false`, `allow_rebase_merge=false`, `allow_squash_merge=true`)
- [x] 2.2 Enable `delete_branch_on_merge=true`
- [x] 2.3 Probe-verify the settings via `gh api` read-back

## 3. Apply branch protection on main

- [x] 3.1 Add branch protection to `main` via `gh api`: `required_status_checks` = exactly {coverage, deps, shell, secrets}, `strict=true`, `required_approving_review_count` = **0** (a personal repo offers GitHub NO org-only bypass — self-approval is forbidden and `bypass_pull_request_allowances` is org-only), `enforce_admins` = true, `allow_force_push` = false. The approval function is instead provided by the `agent-review` required check (task group 7): once it is live and provisioned, it is appended to `required_status_checks.contexts`.
- [x] 3.2 Verify the protection via `gh api /branches/main/protection` read-back (checks names = {coverage, deps, shell, secrets}; `enforce_admins`, `allow_force_pushes=false`, `allow_deletions=false` confirmed). Job `name:` overrides were removed from the four gating jobs in `ci.yml` so check-run contexts equal the job IDs.
- [x] 3.3 Simulate the gate (adversary probe on disposable branch `sim/gate-probe`, now deleted): a PR with a failing required `shell` check reports `mergeStateStatus=BLOCKED`; a direct force-push to `main` is rejected by the protected-branch hook. (`ReviewDecision` self-approval cannot be probed — GitHub forbids it; the review-gate-as-check path is task group 7.)

## 4. CODEOWNERS scaffold

- [x] 4.1 Add `.github/CODEOWNERS` mapping the single maintainer to `.gitleaks/`, `.github/`, `AGENTS.md`, `docs/workflow.md`, `openspec/`
- [x] 4.2 Verify CODEOWNERS parses (no invalid pattern/target); the file signals ownership on governed paths. NOTE: mechanically *requiring* the code-owner review (`require_code_owner_reviews=true`) is left OFF — on a personal repo the code owner is the single maintainer/author, so it would deadlock every governed PR once enforcement is on. The signal stands; enforcement becomes viable when a distinct identity (e.g., a future collaborator or GitHub App) is added as a code owner.

## 5. Dependabot + grouped updates

- [x] 5.1 Add `.github/dependabot.yml` enabling `cargo` and `github-actions` (no npm — no `package.json`) at monthly cadence with grouped version-updates
- [x] 5.2 Verify config parses as valid YAML with the expected ecoystems/cadence/groups. Full "Dependabot sees it" pickup is confirmed after merge (Dependabot scans `.github` on the default branch; this config lands via PR).

## 6. Documentation + spec sync

- [x] 6.1 Update `docs/workflow.md` to describe the enforced gates (required checks, required review, squash-only, CODEOWNERS, Dependabot)
- [x] 6.2 Run `cargo fmt --check`, `clippy -D warnings`, `cargo test`, `openspec validate` — the done-gate for this (governance) change
- [x] 6.3 Confirm `openspec validate` passes and the branch is ready for PR + review gate (per `pr-review` skill)

## 7. Agent review as a required check (decision D7)

- [x] 7.1 Add `.github/workflows/agent-review.yml` (trigger `pull_request_target` — runs TRUSTED code from the default branch, checks out only the base SHA) + `.github/scripts/agent-review.py`: the **adversarial agent review** runs as a required status check (`agent-review`). It reads the PR diff and OpenSpec change artifacts via the GitHub API (never from the working tree, never PR-head code) and sends them (with the review checklist from `docs/workflow.md` / the PR template) to an OpenRouter model. The check's exit code IS the gate: `APPROVED` → exit 0 (check green, mergeable); a reject verdict or any fail-closed condition → exit 1 (check red, merge blocked) plus a verdict comment. No GitHub App or review event — no second GitHub identity is involved.
- [x] 7.2 Security architecture (dictated by the reviewer's own finding during the first live run): the workflow runs from the DEFAULT BRANCH only (pull_request_target + base-SHA checkout), so a PR can never edit the reviewer workflow/script or reach the `OPENROUTER_API_KEY`; the fork guard skips fork PRs entirely (external contributions use the human review gate — a deliberate, documented limitation); the system prompt declares PR-controlled text (title/body/diff/artifacts) to be UNTRUSTED evidence and ignores any instructions inside it (prompt-injection countermeasure); fail closed on missing secret, malformed verdict, or any exception.
- [x] 7.3 Script-level gate: because the workflow runs on `pull_request_target` (no `needs`), it polls (bounded, ~20 min) for the four quality checks (coverage, deps, shell, secrets) to settle on the PR head SHA and only considers approving when they are all SUCCESS. `GITHUB_TOKEN` is read-only (evidence + reject comment only). Workflow is dormant until `vars.AGENT_REVIEWER_ENABLED == 'true'`; `OPENROUTER_API_KEY` is scoped to the `setmeup_ci` environment.
- [x] 7.4 MANUAL (maintainer): set `AGENT_REVIEWER_ENABLED=true` and add `OPENROUTER_API_KEY` to the `setmeup_ci` environment; observe one live pass (note: pull_request_target only fires on PRs AFTER this change merges to `main`, so the first live observation is a subsequent PR) before adding `agent-review` to the required status checks list on `main` (single `gh api` PATCH). These are the only remaining tasks; they are external to this repo and will be done by the maintainer after merge.
- [x] 7.5 Fail closed on evidence truncation (dictated by the agent-review gate's own first live finding): `.github/scripts/agent-review.py` now (a) computes the full PR diff and fails closed (exit 1, check red) when it exceeds `DIFF_CAP`, logging actual-vs-cap, instead of silently reviewing a prefix; (b) marks truncated change artifacts `[TRUNCATED]` and fails closed rather than reviewing partial proposal/tasks. A reviewer cannot credibly approve based on incomplete evidence, so truncated evidence is an automatic rejection.
- [x] 7.6 Evidence completeness — renames visible to the gate (dictated by the second live run): the evidence builder now includes a changed-file manifest (filename, status, previous_filename for renames, +/- counts) in the prompt, because renames/moves contribute no patch text and were previously invisible (e.g. the archive move itself). The main spec `agent-review` captures the evidence-completeness requirement.