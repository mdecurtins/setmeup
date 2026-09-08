## Why

`dev-workflow` provides the *machinery* for agentic development (contract, skills, CI scaffold, docs). `dev-governance` provides the *policy* that machinery enforces: how issues become changes, how changes become reviewed merges, and what quality gates protect the public repo. Policies keep evolving, so they get their own change and lifecycle rather than being baked into the scaffold.

## What Changes

- **Issue-driven development:** GitHub issues seed OpenSpec changes; every actionable issue has a templates and label taxonomy; proposals reference issues, PRs reference changes, merges close issues.
- **PR pipeline for all changes:** every change ships as a branch + PR (even single-operator), with the formal review gate living at the PR.
- **Formal review gate:** every PR receives checklist review by at least one agent and/or human — done-checklist, spec/issue alignment, security-sensitive paths, and risk claim — before merge.
- **Rust quality gates:** latest-stable toolchain pinned via `rust-toolchain.toml`; CI enforces `fmt --check`, `clippy -D warnings`, `cargo test`; a lib-coverage floor via `cargo-llvm-cov` on core modules with judgment applied to OS backends.
- **Dependency governance:** `cargo deny` (advisories + licenses) and/or `cargo audit` in CI; exact-version pinning; minimal-dependency bias.
- **Shell quality gates:** `shellcheck` + `shfmt --check` on `bootstrap.sh` and any shell in the repo.
- **`openspec/config.yaml` context seeded** with the repo's tech stack, conventions, and trust boundary so all future agent-generated artifacts respect them.

## Capabilities

### New Capabilities
- `issue-workflow`: GitHub issue → OpenSpec change flow, issue templates (bug/feature/change), label taxonomy, and linking conventions
- `pr-conventions`: branch + PR pipeline for all changes, PR description conventions, merge-closes-issue wiring
- `quality-gates`: Rust gates (toolchain pin, fmt/clippy/test, coverage floor) and shell gates (shellcheck/shfmt), plus dependency governance (deny/audit/pin)

### Modified Capabilities
- (none — the workflow changes `dev-workflow` and `setmeup-core` reference, but their spec-level requirements do not change)

## Impact

- **New files:** issue templates, PR template, label taxonomy, `rust-toolchain.toml`, CI additions for coverage/deny/shell gates, `openspec/config.yaml` context seed.
- **`dev-workflow` interplay:** `dev-workflow` defines *that* agents operate under contract; `dev-governance` defines *what* the contract enforces. The `AGENTS.md` from `dev-workflow` gains the policy content defined here.
- **`setmeup-core` interplay:** gates execute against the product code as it lands; no change to `setmeup-core` specs themselves.
- **CI cost:** the running matrix stays lean (Linux floor + macOS), with gates composed per-runner (shellcheck on ubuntu, coverage on ubuntu, deny/audit cross-platform).
- **Deliberate non-goals:** no metric-gaming coverage theater; no process that blocks the momentum the scaffold was built to protect; no human random review on trivial changes that aren't real changes.