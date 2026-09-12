## 1. Manifest schema V2

- [x] 1.1 Define new structs in `manifest.rs`: `PackagesConfig`, `NvmConfig`,
      `RustupConfig`, `ReposConfig`, `AliasesConfig`, `ConfigsConfig`,
      `ShellConfig` (expanded from current `ShellConfig`).
      Per-section `validate()` methods on each struct.
- [x] 1.2 Add new fields to `Manifest` struct with `#[serde(default)]`,
      `#[serde(rename_all = "kebab-case")]`. Verify YAML key names match
      kebab-case convention (e.g. `node-version` not `node_version`).
- [x] 1.3 Add backward-compat shim: if `tools` is present, desugar to
      `packages` (emit deprecation warning).
      Spike: parse V1 test sample → produce `Manifest` with `packages`
      filled, verify YAML round-trip test passes.
- [x] 1.4 Add validation for each new section: empty alias names, unknown
      `from` methods, missing repo URLs, **reject `overwrite-managed`
      strategy for JSON/XML/binary config formats**.
- [x] 1.5 Add `Os::current()` resolution for `from` field defaults (apt on
      Ubuntu, brew on macOS, winget on Windows)
- [x] 1.6 Unit-test: parse V2 sample, empty sections, backward-compat shim,
      validation failures (including overwrite-managed rejected for JSON)

## 2. ProvisionHandler trait + dispatch

- [x] 2.1 Define `ProvisionHandler` trait and `CheckResult` / `ProvisionContext`
      in `provisioning.rs`
- [x] 2.2 Implement `capability_handlers()` factory function returning
      `Vec<Box<dyn ProvisionHandler>>` in declared ordering
- [x] 2.3 Refactor `provision()` to iterate handlers with check-before-provision
      loop
- [x] 2.4 Implement mock handler for testing dispatch logic independently
- [x] 2.5 Unit-test: dispatch order, check-skip on satisfied, collect-errors
      without halting, dry-run skips provision.
      **Include regression test** mirroring existing
      `provision_does_not_halt_on_failure` test with new dispatch loop.
- [x] 2.6 Update `cmd_apply` in `main.rs` to match the new `provision()`
      signature: remove `rc_path`, wire `dry_run` through `ProvisionContext`.
      Also update `cmd_status` and `cmd_diff` for V2 section awareness
      (even if deferred, the call site must compile).

## 3. Shell + aliases + configs + rustup handlers (trivial)

- [x] 3.1 Implement `ShellHandler`: writes `~/.config/setmeup/shell-functions.sh`
      with declared functions, ensures source line in `.bashrc`, adds PATH guards
      and env var exports
- [x] 3.2 Implement `AliasesHandler`: writes `~/.config/setmeup/shell-aliases.sh`,
      ensures source line in `.bashrc`, idempotent (file content match → skip)
- [x] 3.3 Implement `ConfigsHandler`: write config file per `strategy` field,
      `create-only` as default (skip if exists), `overwrite-managed` for files
      with setmeup marker
- [x] 3.4 Implement `RustupHandler`: install rustup via verified download when
      absent, install declared toolchain, ensure `~/.cargo/env` sourcing.
      **Testability note:** RustupHandler full provisioning calls external
      commands (curl, rustup). Unit tests cover the already-installed check
      path only. Full provisioning is manual integration test.
- [x] 3.5 Unit-test: all four handlers with check/provision, existing vs absent
      files, divergent content detection, config format validation
      (reject overwrite-managed for JSON/XML)

## 4. Linear crossterm wizard

- [x] 4.1 Refactor `tui.rs` into `tui/` module directory: `tui/mod.rs`
      declares `pub mod wizard; pub mod steps; pub mod dashboard;` and
      **re-exports all previously-public items** (`run_wizard`,
      `prompt_masked`, `render_apply_dashboard`, `DashboardEvent`,
      `DashboardSummary`, `spin_until`, `write_manifest`, `interactive`)
      so `main.rs` compiles unchanged
- [x] 4.2 Implement wizard orchestrator in `wizard.rs`: full-screen clear,
      progress bar rendering, step navigation (forward/back/quit)
- [x] 4.3 Implement step rendering in `steps.rs`: each step as a function
      receiving current state and returning next state or navigation action
- [x] 4.4 Implement Welcome, Credentials, Packages, Repos, Shell, Review steps
      with defaults-first pre-fill from existing manifest
- [x] 4.5 Implement spinner utility (reuse `spin_until` pattern) for async
      step operations
- [x] 4.6 Wire `cmd_configure` in main.rs to launch the new wizard
- [x] 4.7 Remove `ratatui` dependency from Cargo.toml
- [x] 4.8 Verify: `configure` with existing manifest pre-fills steps, non-TTY
      errors cleanly

## 5. Cleanup

- [x] 5.1 Remove `anyhow` dependency from Cargo.toml
- [x] 5.2 Remove dead-code stubs from `main.rs` (`_manifest_backend_to_secrets`,
      `_secret_acquire_to_hint`, `_is_tty`, `_flush`)
- [x] 5.3 Verify: no `#[allow(dead_code)]` annotations remain (or document
      each remaining one with a TODO reference)

## 6. Agent skills

- [x] 6.1 Write `.opencode/skills/manifest-packages/SKILL.md`
- [x] 6.2 Write `.opencode/skills/manifest-nvm/SKILL.md`
- [x] 6.3 Write `.opencode/skills/manifest-rustup/SKILL.md`
- [x] 6.4 Write `.opencode/skills/manifest-repos/SKILL.md`
- [x] 6.5 Write `.opencode/skills/manifest-aliases/SKILL.md`
- [x] 6.6 Write `.opencode/skills/manifest-configs/SKILL.md`
- [x] 6.7 Write `.opencode/skills/manifest-shell/SKILL.md`
- [x] 6.8 Write `docs/ADDING-A-CAPABILITY.md`

## 7. Spec sync

- [x] 7.1 Update `openspec/specs/manifest/spec.md` with V2 schema
- [x] 7.2 Update `openspec/specs/provisioning/spec.md` with handler dispatch
- [x] 7.3 Update `openspec/specs/tui/spec.md` with wizard spec
- [x] 7.4 Write `openspec/specs/agent-skills/spec.md` (new)

## 8. Done-gate

- [x] 8.1 `cargo fmt --check` clean
- [x] 8.2 `cargo clippy --all-targets --all-features -- -D warnings` clean
- [x] 8.3 `cargo test` green
- [x] 8.4 `openspec validate` passes
