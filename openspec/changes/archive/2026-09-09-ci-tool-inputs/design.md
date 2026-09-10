## Context

`.github/workflows/ci.yml` has two `taiki-e/install-action` call sites. Both omit an explicit `with: tool:` input, so each relies on the action's default `tool` value. The default changed between pinned SHAs (old `f1dd293…` → `cargo-deny`; new `4005ba9…` → `cargo-llvm-cov`), which is exactly how the pending Dependabot bump (PR #29) broke the `deps` job: the action installed cargo-llvm-cov where cargo-deny was required, `cargo deny check` failed, and the `agent-review` gate failed closed downstream (issue #31).

## Goals / Non-Goals

**Goals:**
- Make the installed tool explicit at both `install-action` call sites so behavior is independent of the action's changing default.
- Keep the change minimal and mergeable ahead of the upcoming Dependabot bump.

**Non-Goals:**
- Changing any action SHA. The Dependabot PR handles its own versions.
- Reordering or refactoring CI jobs.
- Changing which tools are installed (still cargo-deny and cargo-llvm-cov).

## Decisions

- **Add `with: tool:` to both call sites** rather than pinning the SHA's default. Rationale: the default is an implicit contract that shifts between releases; an explicit input is stable across any future SHA. Announced in the trailing comment too (`# cargo-deny`), which guides readers.
- **Keep the trailing SHA comments** (`# v4`, `# cargo-deny`) — they document provenance and pin intent; no reason to drop them.
- **No spec change to required checks.** Branch protection and check names are untouched; only installation determinism is fixed.

## Risks / Trade-offs

- **The bug only manifests on a SHA whose default differs from the needed tool; current `main` is coincidentally correct.** → The explicit input eliminates the coincidence. Verification is by inspection of the workflow plus the Dependabot PR #29 turning green after this merges.
- **Config drift if one call site is edited and the other missed.** → Both sites are adjacent/similar; the tasks.md checklist references both `deps` and `coverage` jobs explicitly.