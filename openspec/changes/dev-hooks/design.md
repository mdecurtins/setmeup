## Context

The repo and machine have zero git hooks: `.git/hooks` contains only samples, no `core.hooksPath`, no hook tooling. Conventional commits are documented but unenforced, and gitleaks only runs at push-time in CI (by then a secret is already in history). This change adds a repo-pinned local hook set so prevents the wrong moves before they leave the machine.

The done-gate is defined in `AGENTS.md`: `fmt --check` clean, `clippy -D warnings` clean, `cargo test` green, spec scenarios covered. The local hooks encode the cheap part of that gate at commit/push time.

## Goals / Non-Goals

**Goals:**
- Repo-pinned hooks via committed `git-hooks/` + `core.hooksPath` (set by the install script), so a fresh clone activates the committed hooks with a single install command instead of per-machine setup
- `commit-msg`: enforce Conventional Commits (type + optional scope), reject non-conforming messages
- `pre-commit`: reject secret-like staged content before it enters history (gitleaks scan when available; warn and continue when absent — CI is the hard gate)
- `pre-push`: done-gate preflight (fmt, clippy `-D warnings`, tests when a crate is present)
- `scripts/install-hooks.sh` wired into BOTH `bootstrap.sh` (development clone only) and a dev-time install/verify path
- Document hooks in `AGENTS.md` + `docs/workflow.md`

**Non-Goals:**
- NOT a full pre-commit framework (lefthook/husky/pre-commit) — minimal native hooks, no new dependency
- NOT enforcing spec-scenario coverage locally (that stays in CI; local hooks cover the cheap mechanical gates)
- NOT changing the runtime provisioning story of `bootstrap.sh` (see decision D2)

## Decisions

### D1. Native git hooks; no framework
**Decision:** Implement hooks as plain shell scripts in `git-hooks/`, wired via `core.hooksPath`.
**Why:** Zero dependencies, trivially auditable, and the "minimal dependencies" principle. The hooks are small and shell-native; a framework adds abstraction without benefit.
**Alternative considered:** lefthook (nice DX, config-driven) — adds a dependency and a bootstrap step; rejected for minimalism.

### D2. `core.hooksPath` = committed hooks dir; `install-hooks.sh` sets the config; `bootstrap.sh` installs only in a dev clone, never in the delivery path
**Decision:** The hooks live in committed `git-hooks/`; `scripts/install-hooks.sh` sets `core.hooksPath=git-hooks/` via the local git config (it cannot be committed — git never reads repo-committed config). `bootstrap.sh` runs `scripts/install-hooks.sh` ONLY when it detects a repo clone (development context); it never touches hooks in the `curl | sh` product-delivery path.
**Why:** The `bootstrap` spec requires the script to contain no provisioning logic (delivers binary + launches wizard). Detecting a clone and installing the dev hooks is a development-context concern, not product provisioning — an explicit carve-out — but it must be gated so the delivery path stays clean (D3 spec scenario reflects the gitleaks-unavailable warning).

### D3. `pre-commit` secret scan via gitleaks (not a hand-rolled regex)
**Decision:** `pre-commit` runs `gitleaks` on staged content (or a `gitleaks protect`-style scan) using the existing `.gitleaks/setmeup.toml` config, rather than a custom grep.
**Why:** Reuse the exact same allowlist/pattern source as CI for a consistent secret gate. If gitleaks isn't installed locally, the hook degrades gracefully to a warning (fail-open with a note to install), since CI is the hard gate; a false-positive from a hand-rolled regex is worse than a warning.
**Trade-off:** If gitleaks isn't present, local secret prevention is soft — acceptable because CI enforces hard. This fail-open is a declared design choice (not an accident): the spec's secret-prevention requirement and scenarios are written conditionally on gitleaks availability to match.

### D4. `pre-push` done-gate preflight, cargo-gated
**Decision:** `pre-push` runs `cargo fmt --check` and `clippy --all-targets --all-features -- -D warnings` and `cargo test` only when `Cargo.toml` exists (mirrors CI's `hashFiles` guard), so scaffold-only changes are not blocked.
**Why:** Prevent pushing a branch that would red CI for the mechanical gates. Tests add latency on push but the cost is worth the preflight signal.

### D5. Hook install is idempotent and self-verifying
**Decision:** `scripts/install-hooks.sh` sets `core.hooksPath` (and prints the active state), verifies the hooks are executable + wired, and exits non-zero if a hook is missing so the caller knows setup failed.
**Why:** Idempotency (re-runnable on re-clone), and probe-friendly verification matches the repo's `probe-verify` discipline.

## Risks / Trade-offs

- **[gitleaks not installed locally → soft gate] → Mitigation:** hook warns and continues; CI remains the hard gate. Documented in AGENTS.md.
- **[pre-push adds latency] → Mitigation:** cargo-gated (only when crate present); scope is the four done-gate checks, no redundant work.
- **[core.hooksPath surprise for contributors] → Mitigation:** hooks are fast/local, behaviors documented; `install-hooks.sh --verify` prints state.
- **[Hook bypass is trivial (skip hooks)] → Mitigation:** hooks are friction-reduction + prevention, not security; the hard gates are branch protection + CI required checks (dev-protection). Document this framing so nobody over-trusts the local hooks.

## Migration Plan

1. Add `git-hooks/` with `commit-msg`, `pre-commit`, `pre-push`, and a `git-hooks/README.md`
2. Add `scripts/install-hooks.sh` + `scripts/verify-hooks.sh`
3. Wire `bootstrap.sh` to run install-hooks only on clone detection
4. Register `core.hooksPath` in the local config via the install script (per-clone; not committed)
5. Update `AGENTS.md` + `docs/workflow.md`
6. Verify: fresh-clone simulation, non-conventional commit rejected, secret-staged commit rejected, failing-gate push blocked, passing push accepted

Rollback: unset `core.hooksPath`; delete the hooks dir + script; revert docs + bootstrap wiring.

## Open Questions

- None blocking. (If the repo later adds more hooks/coverage, revisit whether a framework earns its keep; not now.)