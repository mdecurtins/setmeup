## 1. Repo-pinned hooks directory

- [ ] 1.1 Add `git-hooks/` with `commit-msg`, `pre-commit`, `pre-push` as plain shell scripts, plus `git-hooks/README.md` describing each hook
- [ ] 1.2 `commit-msg`: validate Conventional Commits (allowed types `feat|fix|chore|docs|refactor|test`, optional scope, reject non-conforming messages with a hint)
- [ ] 1.3 `pre-commit`: scan staged content for secret-like patterns using gitleaks with `.gitleaks/setmeup.toml`; fail closed when gitleaks is available, warn+continue if it isn't (CI remains the hard gate)
- [ ] 1.4 `pre-push`: run done-gate preflight — `cargo fmt --check`, `clippy --all-targets --all-features -- -D warnings`, `cargo test` — only when `Cargo.toml` exists (mirror CI's `hashFiles` guard)
- [ ] 1.5 Make hooks executable and register `core.hooksPath=git-hooks/` in the repo config

## 2. Install + verify script

- [ ] 2.1 Add `scripts/install-hooks.sh`: idempotent install of `core.hooksPath`, prints active state, exits non-zero if a hook is missing
- [ ] 2.2 Add `scripts/verify-hooks.sh`: checks hooks exist, are executable, and `core.hooksPath` is set correctly
- [ ] 2.3 Verify idempotency: run install twice, confirm no mutation and correct state both times

## 3. Bootstrap + dev-time wiring

- [ ] 3.1 Wire `bootstrap.sh` to run `scripts/install-hooks.sh` only when a repo clone (development context) is detected — never in the `curl | sh` delivery path
- [ ] 3.2 Add a dev-time install/verify check (e.g. in `AGENTS.md` workflow / a `make`-style convenience) so development always keeps hooks active
- [ ] 3.3 Probe-verify: fresh-clone simulation installs hooks with zero manual steps; delivery-path bootstrap does NOT install hooks

## 4. Behavior verification (negative tests)

- [ ] 4.1 Attempt a non-conventional commit message → rejected at `commit-msg` with a format hint
- [ ] 4.2 Stage a secret-like string (e.g. a test `sk-…` token) → rejected at `pre-commit`
- [ ] 4.3 Push a branch with a deliberate fmt/clippy/test failure → blocked at `pre-push` with the failing gate reported
- [ ] 4.4 Valid conventional commit + clean gate → accepted end-to-end

## 5. Documentation + spec sync

- [ ] 5.1 Update `AGENTS.md` with the local hooks (what they enforce, how to install/verify, framing as friction-reduction not security)
- [ ] 5.2 Update `docs/workflow.md` describing the hooks + install wiring
- [ ] 5.3 Run the done-gate: `cargo fmt --check`, `clippy -D warnings`, `cargo test`, `openspec validate`
- [ ] 5.4 Confirm `openspec validate` passes and the branch is ready for PR + review gate (per `pr-review` skill)