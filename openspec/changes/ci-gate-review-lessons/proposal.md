## Why

Issue: #40

12 agent-review gate failures have accumulated across 4 PRs (dev-hooks 5x, ci-gate-agent-lessons 4x, agent-review-timeout-resilience 2x, and ci-gate-review-lessons 1x). Each failure costs a 3–5 minute LLM round-trip plus a fix-and-push cycle. The failure patterns reveal gaps in the pre-PR checklist (ci-gate skill), the development contract (AGENTS.md), and the agent-review script itself. This change codifies the lessons from those failures into enforceable process rules and automated checks, so the same rejection patterns do not recur.

## What Changes

- Add a structured risk-assessment template to the ci-gate skill's pre-PR checklist, codifying the patterns that led to 3 failures (missing risk assessments, rollback-only notes instead of concrete analysis, unstated failure modes like cargo regressions).
- Add shell-script edge-case testing requirements to the ci-gate checklist (unreadable files, missing arguments, empty inputs) — caught 1 bug (commit-msg crash on unreadable file) and 1 missing feature (PATH extension after cargo install) that the code-review gate missed.
- Extend the trust-boundary inspection section to require enumerating all tool CWD-read surfaces (config files, environment variables, implicit file references) before claiming a bootstrap/hook script is inert — caught the `.cargo/config.toml` and `$TMPDIR` attack vectors the first trust-boundary fix missed.
- Add a "branch from main" rule to AGENTS.md to prevent archive-leak-style failures (branching from a dirty state carried unrelated files into the diff).
- Add an agent-review base-ref artifact note to the ci-gate checklist: the reviewer reads tasks.md/proposal.md from the base ref, so PRs that modify `openspec/changes/` must address this mismatch in the PR body with per-task probe evidence.
- (Spec-scope) Update the `ci-gate` spec to require leveraging `grep` or file-read checks across all change artifacts for internal-consistency verification, not just LLM review — caught 2 spec-alignment failures (5 vs 4 rejection categories, mandatory vs optional cross-ref).
- (Spec-scope) Update the `ci-gate` spec to include the new check categories enumerated above.

## Capabilities

### New Capabilities

- `ci-gate`: the updated skill captures all 12 failure modes across 6 categories (branch hygiene, risk assessment, shell edge-cases, tool CWD surfaces, internal consistency, base-ref artifacts). The existing skill covers the first 5 categories from PRs #30 and #33 — this change adds 6 new items and the 3 new categories discovered since then.

### Modified Capabilities

- `project-skills`: the ci-gate skill's pre-PR checklist gains 6 new items across 3 new categories (branch hygiene, shell edge-cases, tool CWD surfaces, risk template, agent-review base-ref note, spec-internal consistency grep check).
- `operator-contract` (`AGENTS.md`): gains a "branch from main" rule (always `git checkout -b <branch> main`) next to the existing branch-naming rule.

## Impact

- `.opencode/skills/ci-gate/SKILL.md` — pre-PR checklist gains 6 new items and restructures the risk-assessment section.
- `AGENTS.md` — gains a "branch from main" rule in the Issues & labels section.
- `docs/workflow.md` — the Branch + PR pipeline description augments to mention branching from main.
- No script, CI workflow, or binary code changes.
