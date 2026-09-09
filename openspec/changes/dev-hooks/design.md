## Context

The repo and machine have zero git hooks: `.git/hooks` contains only samples, no `core.hooksPath`, no hook tooling. Conventional commits are documented but unenforced, and gitleaks only runs at push-time in CI (by then a secret is already in history). This change adds a repo-pinned local hook set so prevents the wrong moves before they leave the machine.

The done-gate is defined in `AGENTS.md`: `fmt --check` clean, `clippy -D warnings` clean, `cargo test` green, spec scenarios covered. The local hooks encode the cheap part of that gate at commit/push time.

## Goals / Non-Goals

**Goals:**
- Repo-pinned hooks via committed `git-hooks/` + `core.hooksPath` (set by the install script), so a fresh clone activates the committed hooks with a single install command instead of per-machine setup
- `commit-msg`: enforce Conventional Commits (type + optional scope), reject non-conforming messages
- `pre-commit`: reject secret-like staged content before it enters history (gitleaks scan, fail closed when gitleaks is absent — the local prevention loop must be real, not advisory)
- `pre-push`: done-gate preflight (fmt, clippy `-D warnings`, tests when a crate is present)
- `scripts/install-hooks.sh` wired into a dev-time install/verify path; the delivery bootstrap never executes local files (decision D2)
- Document hooks in `AGENTS.md` + `docs/workflow.md`

**Non-Goals:**
- NOT a full pre-commit framework (lefthook/husky/pre-commit) — minimal native hooks, no new dependency
- NOT enforcing spec-scenario coverage locally (that stays in CI; local hooks cover the cheap mechanical gates)
- NOT auto-installing hooks from the product-delivery bootstrap (see decision D2 — that would execute a user-controlled local file under `curl | sh`)

## Decisions

### D1. Native git hooks; no framework
**Decision:** Implement hooks as plain shell scripts in `git-hooks/`, wired via `core.hooksPath`.
**Why:** Zero dependencies, trivially auditable, and the "minimal dependencies" principle. The hooks are small and shell-native; a framework adds abstraction without benefit.
**Alternative considered:** lefthook (nice DX, config-driven) — adds a dependency and a bootstrap step; rejected for minimalism.

### D2. `core.hooksPath` set by an explicit developer action — the delivery bootstrap NEVER runs local scripts
**Decision:** The hooks live in committed `git-hooks/`; `scripts/install-hooks.sh` sets `core.hooksPath=git-hooks/` via the local git config (it cannot be committed — git never reads repo-committed config). Hook installation is an **explicit developer action** run in their own clone (`scripts/install-hooks.sh`, or a documented dev-time install/verify step). The `curl | sh` product-delivery bootstrap **never** invokes `install-hooks.sh` and, more broadly, never executes any file from the caller's working directory.
**Why:** A `curl | sh` bootstrap that runs a script from the user's CWD is a trust-boundary violation: an attacker who controls that directory (a malicious clone or worktree with a fabricated `origin` and marker file) would get their code executed by the "official" install. There is no unforgeable local identity check that survives CWD execution, so the only safe design is to not execute local files from the delivery path at all. Hook install lives on the developer's explicit action — where the developer's own trust decision applies to their own clone.
**Trade-off:** A fresh dev clone needs one explicit command (`scripts/install-hooks.sh`) instead of automatic wiring — acceptable, documented in AGENTS.md, and probed (adversary scenario: a malicious worktree with spoofed origin/marker, proving the delivery bootstrap executes nothing from it).

### D3. `pre-commit` secret scan via gitleaks (not a hand-rolled regex); fails closed when gitleaks is missing
**Decision:** `pre-commit` runs `gitleaks` on staged content (or a `gitleaks protect`-style scan) using the existing `.gitleaks/setmeup.toml` config, rather than a custom grep. **The gate fails closed: if gitleaks is not installed, the hook blocks the commit and prints install instructions, because the change's stated motivation is that push-time CI detection is already too late — a secret reaching history-clean CI is not the prevention goal.** `install-hooks.sh` verifies gitleaks availability and surfaces install guidance at setup time.
**Why:** Reuse the exact same allowlist/pattern source as CI for a consistent secret gate. Failing open on a missing gitleaks would defeat the change's own premise (a secret is in history long before push-time CI sees it); a hand-rolled regex is worse than a clear "install gitleaks" error. Fail-closed is the honest gate.
**Trade-off:** Commits are blocked on machines without gitleaks until it is installed — the hook message shows the one-line install, and `git commit --no-verify` remains an explicit override (documented, not silent).

### D4. `pre-push` done-gate preflight, cargo-gated
**Decision:** `pre-push` runs `cargo fmt --check` and `clippy --all-targets --all-features -- -D warnings` and `cargo test` only when `Cargo.toml` exists (mirrors CI's `hashFiles` guard), so scaffold-only changes are not blocked.
**Why:** Prevent pushing a branch that would red CI for the mechanical gates. Tests add latency on push but the cost is worth the preflight signal.

### D5. Hook install is idempotent and self-verifying
**Decision:** `scripts/install-hooks.sh` sets `core.hooksPath` (and prints the active state), verifies the hooks are executable + wired, and exits non-zero if a hook is missing so the caller knows setup failed.
**Why:** Idempotency (re-runnable on re-clone), and probe-friendly verification matches the repo's `probe-verify` discipline.

## Risks / Trade-offs

- **[gitleaks not installed locally] → Mitigation:** the pre-commit gate fails closed with install instructions; `install-hooks.sh --verify` checks gitleaks presence and prints the one-line install, so the blocking state is diagnosable. Documented in AGENTS.md.
- **[pre-push adds latency] → Mitigation:** cargo-gated (only when crate present); scope is the four done-gate checks, no redundant work.
- **[core.hooksPath surprise for contributors] → Mitigation:** hooks are fast/local, behaviors documented; `install-hooks.sh --verify` prints state.
- **[Hook bypass is trivial (skip hooks)] → Mitigation:** hooks are friction-reduction + prevention, not security; the hard gates are branch protection + CI required checks (dev-protection). Document this framing so nobody over-trusts the local hooks.
- **[Malicious local clone hijacks the delivery bootstrap] → Mitigation:** the `curl | sh` bootstrap never executes any file from the caller's working directory and never runs `install-hooks.sh`; hook installation is an explicit developer action. Adversary probe (task 3.4) proves a spoofed-origin worktree gets nothing executed.

## Migration Plan

1. Add `git-hooks/` with `commit-msg`, `pre-commit`, `pre-push`, and a `git-hooks/README.md`
2. Add `scripts/install-hooks.sh` + `scripts/verify-hooks.sh`
3. Document the explicit dev-time install step in `AGENTS.md` + a `make`-style convenience (no bootstrap wiring)
4. Register `core.hooksPath` in the local config via the install script (per-clone; not committed)
5. Update `AGENTS.md` + `docs/workflow.md`
6. Verify: fresh-clone simulation, non-conventional commit rejected, secret-staged commit rejected, failing-gate push blocked, passing push accepted, delivery bootstrap executes nothing from CWD

Rollback: unset `core.hooksPath`; delete the hooks dir + script; remove the dev-time install step from docs.

## Open Questions

- None blocking. (If the repo later adds more hooks/coverage, revisit whether a framework earns its keep; not now.)