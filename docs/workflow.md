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
 explore → propose → implement → review → merge (PR) → converge
```

1. **Explore** (`/opsx-explore`): think before committing to a shape. This pass produced the design decisions recorded in each change's `design.md`.
2. **Propose** (`/opsx-propose`): create an OpenSpec change — proposal, specs, design, tasks — validated by `openspec validate`.
3. **Implement** (`/opsx-apply`): work tasks in order. Each task is a checkbox; each is marked complete only when the done-gate holds.
4. **Review**: every PR gets a checklist review — done-gate, alignment to spec/issue, security-sensitive paths, credible risk claim. See `dev-governance`.
5. **Merge**: every change lands as a branch + PR; no direct-to-main. The public history is the durable evidence of the process.

## What "done" means

A task or change is done only when all four hold, per `AGENTS.md`:

- `cargo fmt --check` clean
- `clippy -D warnings` clean
- `cargo test` green
- the change's spec scenarios are covered by tests

## The trust boundary, and why it is architectural

This repo is **public**. The contract therefore treats secret-handling as a correctness boundary, not a convention: credentials live only in the age/keyring vault and non-git backup paths; CI runs a gitleaks scan that fails a change if secret-like patterns appear in tracked content. That is not a README warning — it is a gate. The allowlist for legitimate false positives lives in `.gitleaks/setmeup.toml` and is itself reviewable.

## The pieces, and where they live

| Concern | Location |
|---|---|
| Agent contract | `AGENTS.md` (repo root) |
| OpenSpec changes | `openspec/changes/` — `setmeup-core`, `dev-workflow`, `dev-governance` |
| Project context for artifacts | `openspec/config.yaml` |
| Project skills (curated) | `.opencode/skills/` — `probe-verify`, `done-checklist`, plus the OpenSpec skills |
| CI / enforcement | `.github/workflows/ci.yml` + `.gitleaks/setmeup.toml` |
| This story | `docs/workflow.md` |

## Conventions honored by reference

The shared engineering principles from `~/.config/opencode/AGENTS.md` apply here as read-only reference: SOLID, YAGNI, testing discipline, idempotency, minimal dependencies, exact version pinning, conventional commits, focused commits, security hygiene. They are cited, not rewritten.

## Keeping this current

This doc is a live artifact. When the workflow materially changes — a skill is added or pruned, CI gates change, a new change convention appears — this file and `AGENTS.md` are updated together. A stale `docs/workflow.md` would be worse than none.