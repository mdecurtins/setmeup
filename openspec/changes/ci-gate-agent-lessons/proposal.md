## Why

Over the last two development cycles (PR #30, PR #33), the agent-review gate rejected PRs 9 times across 5 distinct categories of findings, every one of which was avoidable through agent pre-load prompts. These patterns are now well enough understood to codify: a reusable skill that agents load before touching OpenSpec change artifacts will reduce first-attempt CI failures and the 3-5 round-trips they cost.

## What Changes

- Add a new `.opencode/skills/ci-gate/SKILL.md` containing actionable prompts that agents run before opening or revising PRs that touch OpenSpec change artifacts
- The skill covers the 5 rejection categories surfacing across 9 agent-review rejections on PRs #30 and #33: (1) PR body Change pointer and artifact linkage, (2) spec/design/task internal consistency, (3) truthful mechanism claims vs unfulfillable promises, (4) delivery-path trust boundary (never execute CWD files), (5) evidence completeness for renames/missing patches and scope hygiene
- Cross-reference the new skill from `openspec-propose` and `pr-review` skills so agents encounter it at both artifact-writing and review-submission time

## Capabilities

### New Capabilities
- `ci-gate`: a reusable prompt that an agent loads before writing or modifying OpenSpec change artifacts, reducing agent-review gate rejections on the first attempt by surfacing common inconsistency and trust-boundary patterns

### Modified Capabilities
<!-- No spec-level requirement changes to existing product capabilities -->

## Impact

- `.opencode/skills/ci-gate/SKILL.md` (new)
- `.opencode/skills/openspec-propose/SKILL.md` and `.opencode/skills/pr-review/SKILL.md` (cross-reference additions)
- `openspec/specs/ci-gate/spec.md` (new — the change's own delta spec for the ci-gate capability, validated by `openspec validate --all`)
- No changes to `.github/`, workflows, or product code