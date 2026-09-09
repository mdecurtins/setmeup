## Context

The agent-review gate on this repo (`.github/scripts/agent-review.py`) runs as a required status check on every PR. Over two development cycles (dev-protection/PR #30, dev-hooks/PR #33), it rejected PRs 9 times. Each finding was substantively correct and specific — the gate does not make vague statements. The cost of each rejection is a 3-5 minute LLM round-trip plus fix-and-push cycle.

The rejection patterns cluster into 5 categories. Some are OpenSpec-proposal hygiene, others are trust-boundary or spec-design psychology. All can be expressed as agent e-load prompts — not abstract principles, but specific checks to run before opening a PR.

## Goals / Non-Goals

**Goals:**
- A `.opencode/skills/ci-gate/SKILL.md` that an agent loads before writing or modifying OpenSpec change artifacts
- The skill must give the agent a concrete checklist (ideally a few grep commands or file-read checks) to verify against before declaring the proposal ready for review
- Must cover the 5 categories that produced actual rejections

**Non-Goals:**
- NOT modifying the agent-review gate itself (it works correctly; the gap is in proposal authoring)
- NOT a change to `.github/` workflows or CI
- NOT adding OpenSpec specs for the new skill (it's developer tooling, not a product capability)

## Decisions

### D1. Skill format: a SKILL.md with a prompt section the agent reads, not a script the agent runs
**Decision:** The skill is a loaded-markdown file (standard `.opencode/skills/<name>/SKILL.md`). The key section is a "Pre-PR review checklist" that the agent walks through — each item with a concrete grep or file-read command to run.
**Why:** A script would need to be maintained and scheduled; a checklist that the agent runs by hand (they're already reading code) costs nothing to maintain and adapts to evolving rejection patterns.
**Alternative considered:** a Python check script that mirrors `agent-review --selftest` — adds a maintenance surface for a problem the agent can solve by reading.

### D2. Content: specific, not abstract
**Decision:** Each checklist item names the *thing that got rejected*, shows the *actual rejection text*, and gives the *fix pattern* — not a general principle.
**Why:** Abstract "ensure spec consistency" gets ignored. "The design D3 says fail-open but the spec says 'reject unconditionally' — grep for contradictions between these two files" gets followed.

### D3. Which skills to cross-reference
**Decision:** Cross-reference the skill from the `openspec-propose` skill's instructions (add a "Before declaring done, load `ci-gate`" line) and from the `pr-review` skill's "Rules" section.
**Why:** The agent encounters the gate at two points: when writing artifacts (openspec-propose) and when reviewing/submitting (pr-review). Both are natural places to trigger the checklist.

## Risks / Trade-offs

- **[Skill becomes stale] → Mitigation:** the skill references specific PR numbers (30, 33) as source material — those are immutable history. New rejection patterns can be appended as PR references.
- **[Agent ignores the checklist] → Mitigation:** the same. The checklist is loaded by the agent, not enforced by infrastructure. If it doesn't pay rent (prevents avoidable rejections), remove it per the repo's skill-maintenance policy.

## Source material

The following PRs produced agent-review rejections that this skill codifies:

| PR | Branch | Rejections | Categories |
|----|--------|------------|------------|
| #30 | feat/25-dev-protection | 4 | Archive/spec-sync scope creep, missing Change pointer, archive was copy not move, agent-review was overclaimed as enforced |
| #33 | feat/31-ci-tool-inputs | 5 | Spec/design contradictory (gitleaks), zero-step activation unrealistic, CWD-execution trust-boundary flaw, stale bootstrap-dev-clone refs, duped spec sections |
