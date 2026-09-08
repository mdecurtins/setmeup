## Context

`dev-workflow` (already specced) gives the repo its agentic-development machinery: the root `AGENTS.md` operator contract, curated project skills, CI-as-enforcement, and `docs/workflow.md`. It defines *that* agents work under a contract and what "done" means.

`dev-governance` adds the *policy* the machinery enforces. It answers: how do issues become changes, how do changes become reviewed merges, and what quality gates protect a public repo that holds security-sensitive claims. It pairs with the already-seeded `openspec/config.yaml` context so every future artifact an agent generates respects these conventions.

## Goals / Non-Goals

**Goals:**
- Every code change leaves a traceable, reviewable trail: issue → change → PR → merge
- Quality gates run in CI as enforcement, with coverage applied where judgment says it matters
- Policies are codified in the OpenSpec change system so they evolve deliberately
- Keep the single-operator loop fast, not ceremonial

**Non-Goals:**
- Multi-repo or org-level governance (this is one project)
- Re-authoring the scaffold: `dev-workflow` owns the machinery; `dev-governance` owns policy
- A heavy process that a solo operator cannot sustain; each rule must pay rent
- Numerical coverage as a proxy for quality (explicitly avoided)

## Decisions

### 1. Issues seed changes; changes own the work
Every actionable GitHub issue seeds an OpenSpec change. The proposal references the issue number; the PR references the change and issue; merge closes the issue. Issues provide discoverability and the long-lived record; changes carry the spec/design/task discipline. The `github-issues` skill is the entry point for issue-driven work.

### 2. All changes via branch + PR
Even single-operator, every change ships as a branch + PR. This gives a durable place for the review gate, a traceable history for the public repo, and (as an emergent side effect) a strong demonstration of sound AI-driven workflow. Direct-to-main is not allowed; the branch isolates work in flight.

### 3. Formal review gate at the PR
Review is checklist-based, not LGTM, and can be performed by an agent or human:
- Done-checklist satisfied (fmt/clippy/test/spec scenarios, per `dev-workflow`)
- Change aligns with its spec/scenarios and the issue it resolves
- Security-sensitive paths inspected (secrets, git hygiene, identity, credentials) — this repo's trust boundary
- Risk claim is credible (what could go wrong, how it's mitigated)

### 4. Rust gates
- `rust-toolchain.toml` pins latest stable: reproducible build without going stale
- CI: `cargo fmt --check`, `clippy -D warnings`, `cargo test`
- Coverage: maintain a lib-coverage floor (via `cargo-llvm-cov`) on core logical modules (manifest, secrets, identity orchestration); OS backends are integration-tested where the runner permits (e.g., Homebrew on macOS, winget when Windows is added), judged by review rather than a blanket number
- Dependencies: `cargo deny` (advisories, licenses, sources) as the primary gate; `cargo audit` as a lightweight complement or fallback; exact version pinning; minimal-dependency bias per engineering principles

### 5. Shell gates
`bootstrap.sh` and any shell in the repo pass `shellcheck` and `shfmt --check` in CI. The bootstrap is a thin launcher, but it is the FIRST thing a user runs — its quality gate is cheap and warrants enforcement.

### 6. Policy-in-context
The seeded `openspec/config.yaml` context is the always-on carrier of policy: tech stack, trust boundary, issue→change flow, quality gates, and engineering principles. Agents generating artifacts inherit the rules by construction.

## Risks / Trade-offs

- **Formal review on every PR could slow a solo loop** → the gate is a checklist, not a second-person requirement; an agent can review, and trivial mechanical changes move fast through a light checklist invocation
- **Lib-coverage floor could be gamed** → it applies to core logic modules where behavior matters, not the whole crate; backends are exercised by integration tests on capable runners
- **`cargo deny` license/source scanning can be noisy** → allowlist policy is explicit and maintained; denials are deliberate, not accidental
- **Policy drift across three changes** → `dev-workflow` owns machinery, `dev-governance` owns policy, `openspec/config.yaml` carries the carrier; each knows its boundary and references the others
- **Coverage floor false-confidence on OS branches** → mitigated by executing integration tests where the platform is available rather than mocking everything