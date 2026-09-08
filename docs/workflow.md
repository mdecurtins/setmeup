# How setmeup is developed

This is the reviewer-facing story of how this project is built. Every claim here is grounded in a real artifact in this repository — the conversation that produced it was shaped by these rules, and the rules are why the history looks the way it does.

## The belief

AI agents can develop software well — and do it faster than a human alone — **if** the workflow makes the right move the cheap move. The failure mode to engineer against is not "agents are lazy"; it is *drift*: an agent taking a shortcut because the correct path is friction. So the scaffold's job is to make the correct path the low-friction path, and the wrong path visibly worse.

Three ideas carry the whole design:

1. **Spec-first development.** Requirements become OpenSpec changes (`openspec/changes/`) before code exists. A change carries a proposal (why), specs (what), design (how), and tasks (order). Code that has no change behind it does not get written.
2. **The contract is discoverable and enforced.** `AGENTS.md` at the repo root states the non-negotiables: the public-repo trust boundary, what "done" means, and the discipline of exploring before writing. CI (`openspec/changes/dev-workflow`, the `.github/workflows/ci.yml`) turns the cheap part of the contract into a gate.
3. **Machinery and policy are separate changes.** `dev-workflow` built the machinery (contract, skills, CI scaffold, this doc). `dev-governance` adds the policy layer (issue → change flow, PR review gate, quality gates like coverage floor, `cargo deny`, shellcheck). The product itself lives in `setmeup-core`. Each concerns one thing, and they reference each other instead of overlapping.

## The loop

```
 explore → propose → implement → review → merge (PR) → archive → no dangling artifacts
```

1. **Explore** (`/opsx-explore`): think before committing to a shape. This pass produced the design decisions recorded in each change's `design.md`.
2. **Propose** (`/opsx-propose`): create an OpenSpec change — proposal, specs, design, tasks — validated by `openspec validate`. The proposal references the issue that seeds the change.
3. **Implement** (`/opsx-apply`): work tasks in order. Each task is a checkbox; each is marked complete only when the done-gate holds.
4. **Review**: every PR gets a checklist review before merge (see governance, below).
5. **Merge**: every change lands as a branch + PR; no direct-to-main. The public history is the durable evidence of the process.
6. **Archive**: once a branch's change is merged, the change is archived (`openspec archive`) as the *final commit* for that change — after the done-gate, review, and issue verification all pass. No open changes, open issues referenced by a merged change, or unarchived completed changes are left dangling after a branch merges.

## What "done" means

A task or change is done only when all four hold, per `AGENTS.md`:

- `cargo fmt --check` clean
- `clippy -D warnings` clean
- `cargo test` green
- the change's spec scenarios are covered by tests

## The trust boundary, and why it is architectural

This repo is **public**. The contract therefore treats secret-handling as a correctness boundary, not a convention: credentials live only in the age/keyring vault and non-git backup paths; CI runs a gitleaks scan that fails a change if secret-like patterns appear in tracked content. That is not a README warning — it is a gate. The allowlist for legitimate false positives lives in `.gitleaks/setmeup.toml` and is itself reviewable.

## Governance (issue → change → PR → archive)

The policy layer lives in the `dev-governance` change (archived spec: `openspec/specs/`). In force since that branch landed:

- **Branch + PR pipeline.** Every code change ships on a branch and lands via PR; no direct-to-main. Branch naming: `feat/<issue-number>-<change-slug>`, referencing the issue that seeds the change.
- **Checklist review gate.** Every PR receives a substantive checklist review — by an agent or a human — before merge, covering:
  1. Done-checklist satisfied (fmt, clippy, tests, spec scenarios)
  2. Alignment with the change's spec/design and the issue it resolves
  3. Security-sensitive paths inspected (secrets, git hygiene, identity, credentials — trust boundary)
  4. Risk claim credible (what could go wrong, mitigation)
  The PR template (`.github/PULL_REQUEST_TEMPLATE.md`) carries these; the review-gate entry point lives in the project skills.
- **Archive is the terminal step.** A change is archived as the *final commit* for its branch — only after the done-gate, review, and verification of the tagged issue(s) all pass. The archive syncs the change's delta specs into main specs and moves the change to `openspec/changes/archive/`.
- **No dangling artifacts.** After a branch merges: no open issues referenced by the merged change, no unarchived completed changes, no open changes whose work shipped. Everything closes; nothing is left in a half state.

## The pieces, and where they live

| Concern | Location |
|---|---|
| Agent contract | `AGENTS.md` (repo root) |
| OpenSpec changes | `openspec/changes/` — `setmeup-core`, `dev-workflow`, `dev-governance` |
| Project context for artifacts | `openspec/config.yaml` |
| Project skills (curated) | `.opencode/skills/` — `probe-verify`, `done-checklist`, `pr-review`, plus the OpenSpec skills |
| CI / enforcement | `.github/workflows/ci.yml` + `.gitleaks/setmeup.toml` |
| This story | `docs/workflow.md` |

## Conventions honored by reference

The shared engineering principles from `~/.config/opencode/AGENTS.md` apply here as read-only reference: SOLID, YAGNI, testing discipline, idempotency, minimal dependencies, exact version pinning, conventional commits, focused commits, security hygiene. They are cited, not rewritten.

## Keeping this current

This doc is a live artifact. When the workflow materially changes — a skill is added or pruned, CI gates change, a new change convention appears — this file and `AGENTS.md` are updated together. A stale `docs/workflow.md` would be worse than none.

## Lessons learned (from the first full cycle)

The `dev-governance` branch was the first to run the whole loop (issues → branch → PR → review → merge → close → archive). What it taught us:

- **The done-gate is narrower than it looks.** We verified gates in the *scaffold phase* (coverage script, shellcheck, cargo-deny against scratch fixtures) rather than on the merged repo. That verification caught real defects, but only because we probed with *negative* tests — simulating a leak, a bad script, a vulnerable dependency. A gate verified only by "it exists" is decoration.
- **Fail-open is the enemy of a quality gate.** Review found the coverage gate exited 0 when a report contained files but none matched the core modules — a silent bypass at 0% coverage. Fixed to fail closed (`#7`, `9031cd9`). Rule we now apply: if a gate is supposed to *enforce*, its "nothing to enforce" case must fail, not pass.
- **Probe-crafted tools need honest fixture tests.** The coverage script, shellcheck, and cargo-deny were all validated against synthetic inputs (pass, fail, no-match, empty). That's the `probe-verify` skill applied to our own tooling.
- **The issue → change → PR → merge → close → archive trace works, but only if you run it to the end.** GitHub auto-closed only the *first* issue listed in "Closes #1-5"; `#2-5` needed explicit closing. Lesson: don't assume the merge auto-closes every referenced issue — verify with a final sweep (no open issues, no open PRs, exactly the intended active changes), as we now do.
- **Conventional-commit discipline pays off at review.** Four focused commits (templates, docs, gates, artifacts) rendered a diff a reviewer could actually reason about. The one defect found was *inside* one commit, not smeared across the whole branch.
- **The `gh issue create` flags differ from `gh api`.** `--jq` isn't supported on `gh issue create` (the skill documented `gh api` for writes); we captured URLs from output instead. Detail, but it cost one failed call — worth noting in the `github-issues` skill.