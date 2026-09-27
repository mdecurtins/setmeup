## Context

The agent-review gate has rejected 12 PRs across 4 branches with 6 distinct failure categories (issue #40). The existing ci-gate skill only covers the first 2 PRs' rejection patterns (#30 from dev-protection and #33 from ci-tool-inputs). Subsequent PRs (dev-hooks, agent-review-lessons, agent-review-timeout-resilience, ci-gate-agent-lessons) produced 10 new failures that the pre-PR checklist did not prevent. The failure patterns reveal gaps not just in the checklist content but in the development workflow itself (branch hygiene, shell script testing discipline, risk assessment conventions).

## Goals / Non-Goals

**Goals:**
- Extend the ci-gate skill's pre-PR checklist with all 6 failure categories observed to date, so no known rejection pattern recurs
- Add a "branch from main" rule to AGENTS.md to prevent archive-leak and dirty-state branching
- Add a structured risk-assessment template to the ci-gate checklist (concrete failure-mode table, not rollback notes)
- Add a shell-script edge-case testing requirement to the checklist (unreadable/empty files, missing arguments, PATH issues)
- Add a tool-CWD-surface enumeration requirement to the trust-boundary checklist section
- Add an agent-review base-ref artifact mismatch note to the checklist
- Update the ci-gate spec to codify the expanded checklist

**Non-Goals:**
- NOT modifying the agent-review.py script itself (its evidence handling, timeout logic, and review prompt are managed by separate changes)
- NOT adding CI workflow changes (this is documentation and process)
- NOT changing branching mechanics in CI (branch protection, squash-only merges)
- NOT retrofitting changes to the dev-hooks branch (that PR handles its own issues)

## Decisions

### D1. Keep the ci-gate skill as the single pre-PR checklist source
**Decision:** Add all new items to the existing `.opencode/skills/ci-gate/SKILL.md` rather than creating a separate skill.
**Why:** The ci-gate skill is defined as the entry point for pre-PR review. Splitting into multiple skills would create confusion about which checklist to run. Adding to the existing skill keeps one canonical source.
**Alternative considered:** Creating a separate `agent-review-analysis` skill — rejected because it would fragment the pre-PR workflow.

### D2. AGENTS.md gains a "branch from main" rule, not a mention in the ci-gate skill
**Decision:** Add `Always branch from main: git checkout -b <branch> main` to the Issues & labels section of AGENTS.md. The ci-gate checklist item references this rule but does not replicate it.
**Why:** Branch hygiene is a development workflow discipline, not just a pre-PR check. It belongs in the development contract alongside the branch-naming rule. The ci-gate item checks that the branch was created from main (via `git merge-base`) but the rule itself lives in AGENTS.md.
**Alternative considered:** Putting the branching rule only in the ci-gate skill — rejected because branching is a development action, not a PR preparation action.

### D3. Risk assessment uses a structured table template
**Decision:** The ci-gate checklist's risk-assessment item prescribes a specific 4-column table format: Failure mode, Likelihood, Impact, Mitigation. A "rollback" entry is explicitly flagged as a recovery procedure, not a risk.
**Why:** The three failures from missing/insufficient risk assessments all shared the same pattern — either absent entirely or a single "rollback" note. A prescribed format prevents the lazy pattern.
**Alternative considered:** Free-form prose — rejected because it produced the rollback-only pattern that the gate rejected.

### D4. Shell-script edge-case testing is a separate checklist item
**Decision:** New checklist item requiring testing of every new or modified shell script with: unreadable input, empty input, missing argument, non-zero exit from expected-failure commands.
**Why:** The commit-msg hook crash on unreadable files and the missing PATH extension in bootstrap.sh were both missed by code review but found by the agent-review. A dedicated checklist item would catch these before the gate sees them.
**Alternative considered:** Relying on the existing code-review skill — rejected because the code-review gate also missed these issues.

### D5. Tool CWD-surface enumeration is a separate checklist item
**Decision:** New checklist item under the trust-boundary section requiring enumeration of every way a tool reads the working directory before claiming a hook/script is "CWD-inert". Research the tool with `--help`, `man`, or source documentation.
**Why:** The `.cargo/config.toml` and `$TMPDIR` attack vectors were missed because no one enumerated cargo's CWD-read surfaces. The first fix only addressed the `Cargo.toml`/`build.rs` vector.
**Alternative considered:** Relying on a single "check the script" step — rejected because it's too vague.

### D6. Base-ref artifact mismatch is a checklist item with a concrete prescription
**Decision:** New checklist item: the PR body must address the base-ref vs head-ref mismatch when `openspec/changes/` is modified. The fix: include an evidence table with per-task probe results rather than just checking off task boxes that the base ref doesn't see.
**Why:** The agent-review fetches tasks.md/proposal.md from the base ref. The 3 dev-hooks runs all failed partly because the LLM saw 20 unchecked tasks from the base ref.
**Alternative considered:** Modifying the agent-review to read from the head ref — not in scope for this change; the script is managed separately.

### D7. Spec internal consistency check uses grep, not just LLM review
**Decision:** The ci-gate spec requirement for internal-consistency verification is augmented: the pre-PR checklist must include `grep` or file-read checks across all change artifacts counting categories, requirements, and scenarios. If a count or constraint appears in one artifact, verify it appears in all others with the same value.
**Why:** Two failures (5 vs 4 rejection categories, mandatory vs optional cross-ref) slipped past the LLM-based review but were caught by the agent-review gate. A mechanical grep check is cheaper and more reliable than relying on the LLM to notice discrepancies.
**Alternative considered:** Relying solely on `openspec validate` — rejected because validate checks structure, not semantic consistency across artifacts.

## Risks / Trade-offs

- **Richer checklist leads to skipped items** → Mitigation: each item is specific with a concrete verification command; items that cannot be verified (e.g., "no base-ref mismatch" when the change doesn't touch `openspec/changes/`) are marked N/A explicitly.
- **"Branch from main" is redundant for experienced contributors** → Mitigation: it's one line in AGENTS.md and saves a full agent-review failure cycle. Low cost, high signal.
- **Grep-based consistency check is fragile with renames** → Mitigation: the item directs checking each artifact for shared keywords, not exact path matching. If keywords evolve, the grep pattern updates with the artifacts.

## Migration Plan

1. Update `.opencode/skills/ci-gate/SKILL.md` — add 6 new checklist items (items 6–11) and restructure the risk-assessment section
2. Update `AGENTS.md` — add "branch from main" rule to Issues & labels section
3. Update `docs/workflow.md` — mention branching from main in the Branch + PR pipeline paragraph
4. Run `openspec validate --all` to verify spec integrity
5. Open PR with the archive note documenting that the existing dev-hooks branch is not affected by these process changes

Rollback: revert the three documentation files; the ci-gate skill reverts to its pre-change state.

## Open Questions

- None blocking. (The 12 failure runs provide an exhaustive audit of known rejection patterns; any new pattern will be added to the checklist as it occurs, per the skill's "floor, not ceiling" framing.)
