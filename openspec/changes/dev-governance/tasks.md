## 1. Issue Workflow Tooling

- [ ] 1.1 Add issue templates (`.github/ISSUE_TEMPLATE/`): bug, feature, change
- [ ] 1.2 Define and document the label taxonomy: `bug`, `enhancement`, `dev-workflow`, `security`
- [ ] 1.3 Wire the `github-issues` skill as the entry point for issue-driven work (reference in `AGENTS.md`/workflow doc)
- [ ] 1.4 Define the issue → change → PR → issue-close linking convention (proposal references issue; PR references change + issue; merge closes issue)

## 2. PR Conventions

- [ ] 2.1 Add PR template (`.github/PULL_REQUEST_TEMPLATE.md`) capturing: change name/path, issue link, checklist review items, security-path note
- [ ] 2.2 Encode the branch + PR pipeline (every change via PR; no direct-to-main) in `AGENTS.md` + `docs/workflow.md`
- [ ] 2.3 Encode the checklist review gate (done-checklist, spec/issue alignment, security paths, risk claim) in the PR template + workflow doc
- [ ] 2.4 Add a review-gate entry point to the project skills (or reference where the checklist review lives)

## 3. Quality Gates

- [ ] 3.1 Add `rust-toolchain.toml` pinning latest stable
- [ ] 3.2 Extend CI: `cargo fmt --check`, `clippy -D warnings`, `cargo test` (already scaffolded in dev-workflow — enforce server-side status checks)
- [ ] 3.3 Add `cargo-llvm-cov` coverage gate on core logical modules (manifest, secrets, identity) with a configured floor
- [ ] 3.4 Add `cargo deny` (advisories, licenses, sources) and/or `cargo audit` as an advisory gate
- [ ] 3.5 Add `shellcheck` + `shfmt --check` gates for all shell in the repo (bootstrap.sh and any future shell)
- [ ] 3.6 Enforce exact-version pinning convention (document in `AGENTS.md`; add a lint/check if feasible)

## 4. Config Context & Wiring

- [ ] 4.1 Seed `openspec/config.yaml` `context:` (tech stack, trust boundary, issue→change flow, gates, principles) — already pending from this change
- [ ] 4.2 Update `dev-workflow`'s `docs/workflow.md` and `AGENTS.md` to reference governance policies (issues→changes, PR review, gates) without duplicating their definitions
- [ ] 4.3 Verify config context is actually injected into artifact generation (spot-check an artifact after seeding)

## 5. Verification

- [ ] 5.1 Local gate run on current repo: fmt, clippy `-D warnings`, tests (even before product code, this validates the toolchain setup)
- [ ] 5.2 Simulate an advisory/deny violation and confirm the gate fails
- [ ] 5.3 Simulate a shell violation (bad shellcheck) and confirm the gate fails
- [ ] 5.4 Confirm a new PR's default body renders the checklist review items
- [ ] 5.5 Confirm issue → change → PR → close works end-to-end with the github-issues skill