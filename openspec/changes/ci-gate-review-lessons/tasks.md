## 1. Update ci-gate skill checklist

- [ ] 1.1 Add structured risk-assessment template item: 4-column table (Failure mode, Likelihood, Impact, Mitigation); note that "rollback" is a recovery procedure, not a risk
- [ ] 1.2 Add shell-script edge-case testing item: test every new or modified script with unreadable input, empty input, missing argument, expected non-zero exit paths
- [ ] 1.3 Add tool-CWD-surface enumeration item under trust-boundary section: research tool `--help`/`man`/source for every CWD-read surface before claiming a script is inert
- [ ] 1.4 Add branch-origin verification item: `git merge-base HEAD main` must return main; the branch must be created via `git checkout -b <name> main`
- [ ] 1.5 Add base-ref artifact mismatch item: when `openspec/changes/` is modified, PR body must include an evidence table with per-task probe results (the gate reads tasks.md/proposal.md from the base ref)
- [ ] 1.6 Add spec-internal consistency via grep item: for every count/category/constraint mentioned in one artifact, grep all others and confirm the same value appears
- [ ] 1.7 Re-number the existing 5 items + 6 new items into a single sequential list (items 1–11) and update the spec requirement's claim from "5 categories" to "11 documented rejection categories from the full audit history (PRs #30, #33, #34, #35, #36, #39)"
- [ ] 1.8 Load the ci-gate skill and run the updated checklist against the current proposal/spec/design to confirm no violation

## 2. Update development contract

- [ ] 2.1 Add "Always branch from main" rule to `AGENTS.md` Issues & labels section: "Always branch from main: `git checkout -b <branch> main` — never from a dirty state or another branch"
- [ ] 2.2 Update `docs/workflow.md` Branch + PR pipeline description to mention that branches originate from main

## 3. Verification

- [ ] 3.1 Run `openspec validate --all` (21 specs + this change = 22 items expected)
- [ ] 3.2 Verify the ci-gate skill file is structurally valid markdown
- [ ] 3.3 Load the ci-gate skill and confirm the updated checklist loads without errors
- [ ] 3.4 Confirm `git diff --name-status` shows only the expected files: ci-gate skill, AGENTS.md, docs/workflow.md, change artifacts (proposal.md, design.md, specs/, tasks.md)
