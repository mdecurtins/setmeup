## Context

`setmeup` is developed primarily by AI agents. The repository is greenfield beyond the OpenSpec scaffold and a single bare initial commit. There is no `AGENTS.md`, no project skills, no CI, and no reviewer-facing documentation of how the project is built. The shared global engineering principles (SOLID, conventional commits, testing discipline, etc.) already apply via the operator's global config, so the project scaffold must *compose* with them rather than duplicate them.

The foundational design of the tool itself is captured in `openspec/changes/setmeup-core/` (declarative manifest, secret ladder, identity lifecycle, etc.). The core constraint that shapes the workflow: **the repo is public, and secrets/key material must never enter git** — a correctness boundary, not a convention.

## Goals / Non-Goals

**Goals:**
- Make the *cheapest path for an agent also the correct path* (right way is the low-friction way)
- Encode the trust boundary, edit discipline, and done-definition so they are discoverable and enforced
- Keep the scaffold small enough to be a live tool, not a museum
- Compose with, not duplicate, the operator's global engineering principles
- Ground every claim in real repo artifacts

**Non-Goals:**
- Building a skill-zoo or a "look how many agents" demonstration
- Process theater: no ceremony that adds cost without raising quality
- Regulating the product's runtime behavior (that's `setmeup-core`)
- Perfecting every skill — only the ones that pay rent for *this* project

## Decisions

### 1. A single root operator contract (`AGENTS.md`)
The contract lives at the repo root, in the location every agent checks first. It encodes:
- **The trust boundary** — public repo: nothing secret ever enters git; the vault (age/keyring) is the only home for credentials; enforce by architecture and CI, not by reminder
- **Edit discipline** — OpenSpec changes are the vehicle; do not code before specs exist
- **What "done" means** — `cargo fmt` clean, `clippy -D warnings`, tests green, spec scenarios covered
- **Explore-first default** — `/opsx-explore` before writing; ask before guessing

Why root vs. hidden? Root `AGENTS.md` is agent-discoverable, human-reviewable, and unmistakable. Hidden config would hide the contract from the very review it needs.

### 2. Project skills, curated and small
Three that earn their place:
- **verify/probe skill** — the antidote to the #1 agent failure mode (assuming config, claiming work ran). Directs: inspect the actual machine/repo state before asserting.
- **done-checklist discipline** — the explicit gate: fmt/clippy/test/specs before calling it done.
- **OpenSpec skills** (already present) — the propose/apply/explore engine. The workflow doc references them; no need to re-author.

The rule: a skill must pay rent. Unused skills get removed. A kitchen-sink `skills/` dir is an anti-pattern and a red flag to a reviewer.

### 3. CI as enforcement, not decoration
A single GitHub Actions workflow (Linux floor + macOS; Windows optional later):
- `fmt --check` + `clippy -D warnings` + `cargo test`
- **Secret scan** on every push/PR — `gitleaks` (or equivalent) makes the public-repo trust boundary *observable*. This is the highest-signal CI line in the repo.
- The workflow is deliberately lean: a reviewer sees *enforcement*, not a pipeline farm.

### 4. `docs/workflow.md` as the reviewer-facing story
A compact, high-signal doc that (a) tells a reviewer what is believed about AI-driven development, and (b) points to the *real* artifacts — `openspec/`, `.opencode/`, the commits — rather than describing invented process. It is honest, non-aspirational, and updated when the workflow materially changes.

### 5. Compose with global conventions
The shared umbrella principles already exist. The scaffold *references* conventional commits and focused commits; it does not re-define them. The contract points at the global engineering standards rather than duplicating them, so the repo stays DRY.

## Risks / Trade-offs

- **Contract rot** → the contract references live artifacts; CI enforces the cheap parts. A stale doc is caught by the reviewer-facing doc discipline (update on material change).
- **Process theater** → every item in the scaffold maps to a task/verification. Anything that doesn't pay rent gets pruned (explicitly stated in the contract).
- **Secret scan false positives** → scan is a gate, not a judge; triage is documented so legitimate non-secrets (test fixtures, docs) don't wedge CI.
- **CI drift on 3 OSes** → Linux is the floor; macOS included; Windows optional and added deliberately when native-Windows work begins. No unpaid matrix.
- **Demo goal leaking into scope** → explicitly non-goal: no demo-only tooling. The workflow doc is the single sanctioned outward-facing asset, and it is grounded.

## TODO (subsequently captured as its own change)

- (This is that change — `dev-workflow`. The agentic scaffold is now an OpenSpec change, per the exploration that produced it.)