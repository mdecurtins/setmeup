## Context

setmeup is a public repo whose quality gates are currently policy-only: `main` is unprotected (branch protection returns 404), rulesets is empty, no required PR reviews, no required status checks, merge commits enabled. The governance contract (`AGENTS.md`, `docs/workflow.md`, archived specs) *describes* enforcement but nothing *enforces* it. This change puts the GitHub-side enforcement layer in place so the contract becomes mechanically true.

Current state (probed):
- `GET /branches/main/protection` → 404 (unprotected)
- `GET /rulesets` → `[]`
- Repo settings: merge commit enabled, auto-merge off, delete-branch-on-merge off
- CI workflows exist and run (quality×2 OS, coverage, deps, shell, secrets) but none are *required checks*
- Third-party actions pinned to tags (`@v4`, `@v2`, `@stable`)

## Goals / Non-Goals

**Goals:**
- Mechanically prevent direct/force-pushes to `main`
- Require the four enforcement-heavy jobs (coverage, deps, shell, secrets) as gate checks on `main`, **plus the adversarial `agent-review` check that takes the review gate's place**
- Enforce squash-only merge so history stays linear (the "focused commits" story is the evidence)
- Require branches be up-to-date before merge
- Auto-delete merged branches
- SHA-pin all third-party GitHub Actions for supply-chain integrity
- Add a CODEOWNERS scaffold + Dependabot (monthly, grouped)

**Non-Goals:**
- NOT adding Windows to the CI matrix (tracked separately in #23; deliberately out of scope per maintainer)
- NOT changing the per-OS `quality` jobs to be required (they remain visible but non-gating; four jobs are the gate)
- NOT introducing secret-scan tooling changes (gitleaks already runs; dev-hooks handles the local prevention loop separately)
- NOT adding a second human reviewer for this single-maintainer repo — the gate is the automated agent-review check

## Decisions

### D1. Enforce via branch protection on `main` (not rulesets)
**Decision:** Gate `main` with classic branch protection, not a repository ruleset.
**Why:** The repo already has branch protection semantics documented in `pr-conventions`; classic protection is the smallest, most auditable delta for a single-maintainer repo. Rulesets add overlap and are unnecessary here.
**Alternative considered:** Rulesets — more powerful (bypass actors, metadata conditions) but more surface for one maintainer; deferred unless multiple environments appear.

### D2. Required checks = {coverage, deps, shell, secrets} (+ agent-review)
**Decision:** Only these five (coverage, deps, shell, secrets, agent-review) become required checks. The two `quality` OS lanes (ubuntu/macos) stay visible but not required.
**Why:** Per maintainer answer to the scope question, plus the D7 decision. The four are the *enforcement-heavy* jobs (fail-closed coverage, dependency deny/advisories, shellcheck, secret scan); `quality` is multiply-redundant across two runners and they re-run every push; gating neither slows merge nor weakens real enforcement. `agent-review` is the adversarial review gate as a check (D7).

### D3. Squash-only merge (linear history)
**Decision:** Set repo default merge method to squash; disable merge and rebase commits via protection/allowed-merge-methods.
**Why:** Linear history is the stated evidence of the process (`docs/workflow.md`: "focused commits... is the durable evidence"). A squash merge produces one focused commit per change and keeps `main` bisectable.
**Alternative considered:** Rebase-merge (preserves original commits) — but that keeps the intermediate commit-noise that the "focused" story rejects. Squash chosen.

### D4. SHA-pin third-party actions, keep tag as comment
**Decision:** Replace every `uses: owner/action@vX` with the full commit SHA, and add the original tag as a trailing comment for provenance.
**Why:** Mutable tags are a supply-chain risk (a tag can be moved). A pinned SHA + comment preserves both safety and readability.
**At risk:** First-party `actions/checkout@v4` too — the same reasoning applies (supply-chain), and `@v4` could be retagged. All third-party and first-party-out-of-GitHub actions pinned.

### D5. CODEOWNERS scaffold for governance/trust-boundary paths
**Decision:** Add a `CODEOWNERS` assigning the single maintainer to `.gitleaks/`, `.github/`, `AGENTS.md`, `docs/workflow.md`, `openspec/`.
**Why:** In a single-maintainer repo ownership is nominal, but the file signals intent to viewers and makes governance-path changes require awareness. Cost is ~0; value is signaling + future-proofing if collaborators join.
**Alternative:** Skip entirely. Rejected: the user explicitly wants it scaffolded to "signal that I took codeowners into account."

### D6. Dependabot monthly + grouped
**Decision:** Enable Dependabot for `cargo` and `github-actions` at monthly cadence with grouped PRs. (No `npm` — this repo has no `package.json`.)
**Why:** "cargo-deny gates but nothing proposes updates" is the gap. Monthly grouped PRs keep update noise low while ensuring the dependency gate has real candidates. Exact pinning stays in the manifest; Dependabot proposes, CI + review approve.

### D7. Agent review as a required status check (no GitHub App)
**Decision (replacing the count=1 / GitHub App approach):** On a user-owned (personal) repo, GitHub provides NO org-only bypass for required approvals (`bypass_pull_request_allowances` 422s outside orgs) and forbids the PR author from approving their own PR. Requiring a *review event* would deadlock a single maintainer. The decided solution sidesteps review events entirely: **the adversarial agent review is enforced as a required *status check* named `agent-review`**. A PR cannot merge until that check is green — functionally identical to an approval, but a check, so no second GitHub identity, no App, no private key, and no self-approval rules are involved.
**Mechanics:** the `agent-review` job lives in `.github/workflows/ci.yml` with `needs: [coverage, deps, shell, secrets]` (so GitHub schedules it only after those settle — no race between the review and the quality checks), with a fork guard and opting into the `setmeup_ci` environment housing the only secret, `OPENROUTER_API_KEY`. `.github/scripts/agent-review.py` reads the PR diff + OpenSpec change artifacts with the built-in read-only `GITHUB_TOKEN` and sends them (with the review checklist from `docs/workflow.md`/`PR_TEMPLATE.md`) to an OpenRouter model. The check's exit code IS the gate: approve → exit 0 (green, mergeable); reject or any fail-closed condition → exit 1 (red, merge blocked) + a verdict comment.
**Security posture (public repo):** `pull_request` trigger + fork guard, so fork PRs (read-only token, no secrets) never reach the elevated key; the key is further scoped to the `setmeup_ci` environment (defense in depth). `GITHUB_TOKEN` does only read-only evidence + the reject comment — it never posts approvals (there are none). The script fails closed: no green quality checks, missing key, malformed LLM verdict, or any exception → red check.
**Sequencing:** the workflow and script ship now, dormant behind `vars.AGENT_REVIEWER_ENABLED=='true'` (so CI stays quiet until provisioned); the maintainer sets the env var + `OPENROUTER_API_KEY` secret, observes one live pass, then adds `agent-review` to the required status checks on `main` (one `gh api` PATCH, `tasks.md` 7.4). Until then the four quality checks remain the required gate.
**Honest caveat:** an automated checklist gate, not an independent human review — it mechanically forces the checklist to run on every PR (the intended outcome), and the substantive human review remains the process gate (`pr-review` skill).

### D8. CODEOWNERS is a signal, not a mechanical requirement (on this repo)
**Decision:** CODEOWNERS assigns the maintainer to governance paths but `require_code_owner_reviews` stays **off**. On a personal repo the code owner IS the author, so mechanically requiring the code-owner review would deadlock every governed PR (GitHub forbids self-approval). The file signals intent and surfaces the owner in the GitHub UI; it could become mechanically relevant only if a distinct code-owner identity joins later.

### D9. Remove the four gating jobs' `name:` overrides
**Decision:** `coverage`, `deps`, `shell`, `secrets` jobs in `ci.yml` drop their human-readable `name:` overrides so the check-run context names **equal** the job IDs the branch protection requires. Opaque display strings like `shell gate (shellcheck + shfmt)` create a silent mismatch risk between the protection config and the actual check context; short job IDs are self-documenting and the job purposes are already explained in the `ci.yml` header comments.
**Why:** the `required_status_checks.contexts` API matches on the check-run (job) *context name*, so name overrides must exactly match what the protection lists. Removing them makes the spec's "{coverage, deps, shell, secrets}" literally true.

## Risks / Trade-offs

- **[Required checks misconfigured] → Mitigation:** Each `required_status_checks` entry must match the exact CI job/context name; verified via `gh api` after applying (probe-verify the running check name list, not assume).
- **[Squash-only surprises contributors] → Mitigation:** `docs/workflow.md` + PR template note the squash policy; it matches the existing "focused commit" narrative.
- **[SHA-pin breaks when actions update] → Mitigation:** Dependabot opens update PRs for the pinned action SHAs (version-updates include github-actions); the tag-comment makes the current version obvious when updating.
- **[Enforce-admins preferred] → Mitigation:** Keep `enforce_admins: true` so maintainer is subject to the same gate (the point of a public governance demo).
- **[CODEOWNERS required-reviewer friction] → Mitigation:** Ownership paths are the governance files; the single owner reviewing them is expected overhead, not burden (and it is a signal, not a binding gate — see D8).
- **[Agent-review check misconfigured or the LLM is unreliable] → Mitigation:** Fail closed (red check on any uncertainty or error), dormant until enabled, and enabled only after live observation; the human `pr-review` gate remains the substantive review. A red agent-review is a hard stop, never a suggestion.

## Migration Plan

1. Update `.github/workflows/ci.yml`: SHA-pin all actions, drop the four gating jobs' `name:` overrides
2. Apply repo settings: squash-only, delete-branch-on-merge, auto-merge (optional)
3. Apply branch protection on `main`: required checks = exactly {coverage, deps, shell, secrets}, strict, enforce admins, no force-push (`agent-review` is added to the list in step 9 once provisioned; see D7)
4. Add `.github/CODEOWNERS` (signal; `require_code_owner_reviews` stays off, see D8)
5. Add `.github/dependabot.yml`
6. Add the agent-review job to `ci.yml` + the `.github/scripts/agent-review.py` script (dormant behind `AGENT_REVIEWER_ENABLED`)
7. Probe-verify each gate (`gh api` read-back + a test PR simulation)
8. Update `docs/workflow.md` + archive flow
9. Post-merge (maintainer, `tasks.md` 7.4): set `AGENT_REVIEWER_ENABLED` + `OPENROUTER_API_KEY` in `setmeup_ci`, observe one live pass, then add `agent-review` to the required checks

Rollback: revert repo settings via `gh api` (remove branch protection) if a gate misbehaves; the committed files are trivially revertable.

## Open Questions

- None blocking. (Windows matrix, merge-button cosmetics, and future rulesets are tracked separately; all in-scope items are decided.)