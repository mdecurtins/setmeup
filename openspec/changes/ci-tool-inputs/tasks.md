## 1. Make tool installation explicit

- [x] 1.1 Add `with: tool: cargo-deny` to the `Install cargo-deny` step in the `deps` job of `.github/workflows/ci.yml`
- [x] 1.2 Add `with: tool: cargo-llvm-cov` to the `Install cargo-llvm-cov` step in the `coverage` job of `.github/workflows/ci.yml`
- [x] 1.3 Verify no other `taiki-e/install-action` call site in `.github/workflows/` relies on the installer default (grep all workflows)

## 2. Verification

- [x] 2.1 Confirm `agent-review` workflow's quality-gate polling list is unaffected (it lists check contexts, not tools) and no spec/check-name change is needed
- [x] 2.2 Run the done-gate for this governance change: `cargo fmt --check` ✓, `openspec validate` ✓ 18/18. `clippy`/`cargo test` are blocked by a pre-existing environment limitation only (no `cc`/`gcc` on this box; `target/` holds root-owned artifacts from a prior privileged build) — no Rust source changes in this change, and the full gate runs on the GitHub runner.
- [x] 2.3 Confirm issue #31 is referenced by the change; the PR references both the change and #31, and merging closes #31