---
name: ci-gate
description: Must be loaded before opening or revising a PR that touches OpenSpec change artifacts — it provides the pre-PR checklist that prevents agent-review gate rejections. Invoke by mentioning `ci-gate-agent`.
license: MIT
metadata:
  author: setmeup
  version: "1.0"
---

# The agent-review gate

The agent-review gate (`.github/scripts/agent-review.py`) is a required status check on every PR — a rejection costs a 3–5 minute LLM round-trip plus a fix-and-push cycle. This skill codifies the gate's documented rejection categories from PRs #30 and #33 so you can clear them BEFORE the gate sees the PR. Load this skill and run the checklist before opening a PR or revising OpenSpec change artifacts.

Every item is specific: the rejection pattern that actually happened, the verification command, and the fix. PRs #30 and #33 are the immutable source material — do not rely on memory for what got rejected; re-read the artifacts if unsure.

## Pre-PR review checklist

### 1. PR body artifact linkage
**Rejection (#30):** missing `**Change:**` pointer. The gate parses `**Change:**\s*([\w.-]+)` from the PR body and fetches `openspec/changes/<slug>/proposal.md` and `tasks.md` at the base ref. No pointer → no change artifacts reach the reviewer → the gate cannot verify spec alignment and rejects.

- **Check:** read the drafted PR body and confirm it contains a `**Change:** <slug>` line; then verify the slug is a real change directory: `ls openspec/changes/`
- **Fix:** write `**Change:** <change-slug>` in the PR body, matching the change directory name exactly.

### 2. Spec/design/task internal consistency
**Rejections (#33):** the spec and design contradicted each other (gitleaks claim) and a spec section was duplicated. The gate compares proposal.md + tasks.md with the diff — contradictory or duplicated claims read as sloppy or divergent work.

- **Check:** for each requirement that appears in more than one artifact, confirm the statements agree: `grep -rn "<requirement keyword>" openspec/changes/<slug>/` (runs across spec.md, design.md, tasks.md)
- **Fix:** state the claim once, consistently; delete duplicated sections; if a design decision changes a requirement, update all artifacts together in the same change.

### 3. Truthful mechanism claims
**Rejections (#30, #33):** agent-review was overclaimed as "enforced" when the gate is advisory, and "zero-step activation" promised no manual setup for per-clone config values. The gate checks mechanism claims against what the code/diff actually does.

- **Check:** for every "automatic / zero-step / seamless / enforced" claim, find the code path that makes it true: `grep -rn "zero\|automatic\|seamless\|enforced" openspec/changes/<slug>/`
- **Fix:** claim only what the mechanism delivers. If a value is per-clone config, say "each clone must set X in Y"; if a gate is advisory, say so — never write "enforced".

### 4. Delivery-path trust boundary
**Rejection (#33):** the `curl | sh` bootstrap could execute local CWD files or run `install-hooks.sh` — a public-repo trust-boundary flaw. The gate scrutinizes anything that pipes remote code into a shell.

- **Check:** inspect every bootstrap/hook script the change touches: `grep -rn "curl\|install-hooks.sh\|source \./\|\. ./" <changed files>`. The piped script must not execute files from the CWD, must not run `install-hooks.sh`, and must reference explicit, pinned paths only.
- **Fix:** make the bootstrap self-contained; fetch only explicit URLs; never source or execute local files as part of the pipe.

### 5. Evidence completeness + scope hygiene
**Rejections (#30, #33):** an archive was a copy, not a move (git saw no rename); archive/spec-sync PRs carried out-of-scope changes; a rename left stale `bootstrap-dev-clone` references. The gate reviews the file manifest with statuses and previous filenames — renames/moves are never invisible.

- **Check:** list what the PR actually changes and confirm renames are real moves with all references updated: `git diff HEAD^ --name-status` (full scope), `git diff HEAD^ --name-status --diff-filter=R` (renames), then `grep -rn "<old name>" .` for stale references
- **Fix:** use `git mv` for moves and propagate the new path to every reference; keep archive/spec-sync PRs strictly to their stated scope.

## Source material

- PR #30 (feat/25-dev-protection) — archive/spec-sync scope creep, missing Change pointer, archive was copy not move, agent-review overclaimed as enforced
- PR #33 (feat/31-ci-tool-inputs) — spec/design contradictory (gitleaks), zero-step activation unrealistic, CWD-execution trust-boundary flaw, stale bootstrap-dev-clone refs, duped spec sections

Each rejection was substantively correct and specific — treat this checklist as the floor, not the ceiling. If the gate rejects on a new pattern anyway, append it here with its PR reference.