## Why

Over the last two development cycles (PR #30, PR #33), the agent-review gate rejected PRs 9 times across 5 distinct categories of findings, every one of which was avoidable through agent pre-load prompts. These patterns are now well enough understood to codify: a reusable skill that agents load before touching OpenSpec change artifacts will reduce first-attempt CI failures and the 3-5 round-trips they cost.

## What Changes

- Add a new `.opencode/skills/ci-gate/SKILL.md` containing actionable prompts that agents run before opening or revising PRs that touch OpenSpec change artifacts
- The skill covers: PR body artifact linkage, spec/design/task consistency, truthful mechanism claims, delivery-path trust boundary, evidence-completeness, the base-SHA reviewer constraint, and scope hygiene
- Optionally update `openspec-propose` or `pr-review` skills to cross-reference the new `ci-gate` skill

## Capabilities

### New Capabilities
- `ci-gate`: prompts that an agent loads before writing or modifying OpenSpec change artifacts to reduce agent-review gate rejections on the first attempt

### Modified Capabilities
<!-- No spec-level requirement changes — this is a developer tooling skill, not a product capability -->

## Impact

- `.opencode/skills/ci-gate/SKILL.md` (new)
- Possibly small cross-reference additions to `.opencode/skills/openspec-propose/SKILL.md` and `.opencode/skills/pr-review/SKILL.md`
- No changes to `.github/`, workflows, product code, or OpenSpec specs