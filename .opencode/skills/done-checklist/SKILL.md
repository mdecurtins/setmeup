---
name: done-checklist
description: Apply the explicit completion gate before declaring any work complete. Use when an agent is about to say a task, task-group, or change is done, or before reporting success on implementation work.
license: MIT
metadata:
  author: setmeup
  version: "1.0"
---

# The done gate

"Done" is a defined state in this repo, not a feeling. Before declaring any implementation work complete (a task, a task group, a change, a session report of "all done"), run this checklist and confirm each item against real evidence.

## The gate

A task or change is complete only when ALL of the following hold:

1. **`cargo fmt --check` is clean** — formatting matches rustfmt.
2. **`clippy -D warnings` is clean** — no lint warnings (warnings are denied).
3. **`cargo test` is green** — the full test suite passes.
4. **Spec scenarios are covered** — the change's spec scenarios have corresponding tests, and they pass.

Per the repo contract (`AGENTS.md`): "Anything declared 'done' without this gate is not done."

## How to apply it

1. For each item, run the actual command / inspect actual output. Do not recall or assume (pair with the `probe-verify` skill).
2. If any item fails, the work is **not done** — fix forward until every item passes.
3. When reporting completion, state which items you verified (e.g., "fmt/clippy/test verified; spec scenarios covered by tests X, Y").

## Notes

- Verify against the *current* state of the branch — the gate applies to what is now true, not what was true an hour ago.
- If a change cannot satisfy a gate (e.g., no tests possible for docs), say so explicitly and justify, rather than silently skipping.

## Maintenance

This skill must pay rent: if it ever stops mapping to real agent work, it is removed. Per `AGENTS.md`, unused skills are pruned — keep this doc current and genuinely used.