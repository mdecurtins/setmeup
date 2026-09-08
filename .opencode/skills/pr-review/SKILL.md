---
name: pr-review
description: Perform the checklist review gate on a pull request before it may be merged. Use when a PR is proposed and needs the substantive governance review (not just CI) that the merge bar requires.
license: MIT
metadata:
  author: setmeup
  version: "1.0"
---

# PR review gate

Every PR in this repo must pass a substantive, checklist-based review (by an agent or a human) before merge — see `docs/workflow.md` (Governance) and `dev-governance` (pr-conventions). This is not an LGTM; it is a gate.

## The checklist

Review the PR against all four items and record the outcome for each:

1. **Done-checklist satisfied** — `cargo fmt --check` clean, `clippy -D warnings` clean, `cargo test` green, and the change's spec scenarios are covered by tests. Pair with the `done-checklist` skill; verify against actual output, not claims.
2. **Spec/issue alignment** — the change matches its OpenSpec change's spec/design, and satisfies the issue(s) it resolves. Skim the change artifacts and confirm the code does what the spec says (no scope creep, no silent divergence).
3. **Security-sensitive paths** — inspect anything touching secrets, keys, credentials, identity, or git hygiene against the trust boundary in `AGENTS.md`: no secret material in tracked paths, secure handling, correct permissions. If the PR is security-relevant and a `security`-relevant diff exists, call it out explicitly.
4. **Risk claim credible** — what could go wrong (a real scenario), and how the change mitigates it. If the PR doesn't state a risk, the reviewer should assess one from the diff.

## Rules

- **A failed item blocks merge.** Address and re-review before the PR is eligible.
- **Review the diff, not the description.** Claims in the PR body are inputs; actual code is evidence.
- **Be specific.** "LGTM" is not a review record; name what you checked and what you found (or didn't).
- **Record the outcome.** State which items passed and any findings, so the review is auditable in the PR thread.

## Maintenance

This skill must pay rent: if it ever stops mapping to real agent work, it is removed. Per `AGENTS.md`, unused skills are pruned — keep this doc current and genuinely used.