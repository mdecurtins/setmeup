## Why

`setmeup` is developed by AI agents as much as by a human. For that workflow to stay good software engineering — *especially* in a public repo holding security-sensitive claims — the way agents work here must be explicit, low-friction, and enforced. A curated scaffold (operator contract, project skills, CI as enforcement, reviewer-facing docs) makes the cheapest path the correct path, keeps quality non-negotiable, and — as a deliberate side effect rather than a goal — demonstrates sound AI-driven software engineering to any reviewer.

## What Changes

- Add a root-level operator contract (`AGENTS.md`) defining how agents work in this repo: the trust boundary, edit discipline, and the definition of "done."
- Add a small, curated set of project skills (not a kitchen sink), including a "probe before assume" skill for gathering environment truth and a done-checklist discipline.
- Add CI that enforces the contract, not just runs tests: `fmt --check`, `clippy -D warnings`, `cargo test`, and a secret-scan guarding the public-repo trust boundary.
- Add `docs/workflow.md` — a reviewer-facing narrative of the AI-driven development process, grounded in the real artifacts of this repo (not invented claims).
- Wire these into the existing OpenSpec / `.opencode` scaffold so the workflow is spec-driven and self-consistent.
- Confirm the global conventions (conventional commits, focused commits from the project's shared engineering principles) are honored and referenced, not rewritten.

## Capabilities

### New Capabilities
- `operator-contract`: the root `AGENTS.md` — trust boundary, edit discipline, done-definition, explore-first default
- `project-skills`: the curated skill set agents load on demand (verify/probe skill, done-checklist discipline, OpenSpec skills wired)
- `ci-pipeline`: GitHub Actions enforcement — fmt, clippy, tests, secret scan — with a maintained OS matrix
- `workflow-docs`: `docs/workflow.md` reviewer-facing narrative plus README wiring

### Modified Capabilities
- (none — greenfield, this is the first workflow change)

## Impact

- **New files:** `AGENTS.md`, `docs/workflow.md`, `.github/workflows/ci.yml`, curated skills under `.opencode/skills/`.
- **Repo trust boundary reinforced:** the public-repo + secrets constraint becomes enforced (CI scan) rather than a README warning.
- **No product code changes:** this change affects how the project is *developed*, not what setmeup does.
- **CI cost for a personal tool:** the floor is Linux; macOS is cheap and included; native-Windows runners are optional and can be added later — keeping the enforcement meaningful without a heavy triple-matrix burden.
- **Deliberate non-goals:** no skill-zoo, no process theater, no demo-special tooling. This scaffold is for the work; any demonstration value is emergent.
- **`setmeup-core` impact:** none to its specs; it stays as-is. The workflow scaffold runs alongside it.