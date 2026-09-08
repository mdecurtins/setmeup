## 1. Operator Contract

- [x] 1.1 Write root `AGENTS.md`: trust boundary (public repo, secrets never in git), edit discipline (OpenSpec first), done-definition (fmt/clippy/test/specs), explore-first default
- [x] 1.2 Reference the shared global engineering principles without duplicating them (composition, not copy)
- [x] 1.3 Verify the file is discoverable at repo root and free of process theater (every line maps to a task/verification)

## 2. Project Skills

- [x] 2.1 Create probe/verify skill under `.opencode/skills/` directing inspection of actual machine/repo state before asserting
- [x] 2.2 Create done-checklist skill (fmt/clippy/test/spec-scenarios gate before declaring complete)
- [x] 2.3 Confirm OpenSpec skills (propose/apply/explore) are present and referenced; do not re-author them
- [x] 2.4 Add a note in the skill docs that unused skills are removed (paying-rent rule)

## 3. CI Pipeline

- [x] 3.1 Create `.github/workflows/ci.yml` with `fmt --check`, `clippy -D warnings`, `cargo test` on ubuntu-latest
- [x] 3.2 Add macOS runner to the matrix
- [x] 3.3 Add secret scan (gitleaks or equivalent) as a required gate on push/PR
- [x] 3.4 Add a documented allowlist mechanism for legitimate false positives (test fixtures, docs)
- [x] 3.5 Verify the workflow is lean — each step maps to a stated enforcement purpose

## 4. Workflow Documentation

- [x] 4.1 Write `docs/workflow.md`: what is believed about AI-driven development, grounded in this repo's real artifacts
- [x] 4.2 Wire `README.md` to reference `docs/workflow.md` and the OpenSpec change workflows
- [x] 4.3 Update `docs/workflow.md` when skills or CI change (kept current by convention stated in the doc)

## 5. Verification

- [x] 5.1 Run a clean local gate: fmt, clippy `-D warnings`, cargo test all pass
- [x] 5.2 Simulate a secret leak and confirm the CI scan flags it (trust boundary is observable)
- [x] 5.3 Confirm `AGENTS.md`, skills, CI, and docs/workflow reference each other consistently (no orphaned claims)