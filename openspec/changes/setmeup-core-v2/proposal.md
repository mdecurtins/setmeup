## Why

The setmeup-core MVP shipped a working engine (manifest parse, secret backend,
provisioning, identity, CLI surface) with several deferred items. Production
experience with the MVP reveals gaps:

1. **The manifest schema is too narrow** — it only models packages, dotfiles,
   shell rc lines, and OS preferences. Real machines need nvm, repo cloning,
   managed aliases, PS1 customization, per-tool config files, shell functions,
   and PATH management.

2. **The TUI wizard is a stub** — `configure` validates + writes the manifest
   but offers no interactive UX (deferred to #20). The live apply dashboard
   works but needs the wizard counterpart.

3. **The provisioning engine has no capability extension mechanism** — adding a
   new "thing to install/configure" requires editing 7 files with no recipe.
   There's no trait-based handler dispatch, no skill-driven research pattern.

4. **No agent skills for tool ecosystems** — when an agent needs to add or
   update a provisioning capability, it has no canonical reference for that
   tool's install methods, config format, or verification steps.

5. **ratatui dependency carries cost without benefit** — declared in Cargo.toml
   but unused. The linear wizard can be built with crossterm (already a dep)
   and minimal ANSI rendering.

## What Changes

This change does NOT implement new capabilities. It establishes the
**architectural foundation** for adding them:

### 1. Manifest schema V2 (flat sections, strongly typed)

The manifest grows new top-level sections, each backed by a Rust struct:

- `packages` — replaces the current `tools` section, adds `from:` (apt/brew/
  winget/pip/download) and `depends:` for ordering
- `nvm` — node version manager configuration, with `global_packages`
- `rustup` — Rust toolchain manager configuration
- `repos` — named repo entries for cloning to `~/repos/`
- `aliases` — key/value alias definitions referencing repo names
- `configs` — per-tool managed config file content with merge strategy
- `shell` — expanded: prompt, functions, source files, PATH extends, env vars

Each section is optional. The manifest remains OS-portable with per-OS
resolution. Current `tools` / `dotfiles` / `os` sections are retained but may
be deprecated in favor of the new `packages` / `configs` sections (decided per
capability at implementation time).

### 2. Trait-based provisioning engine

A `ProvisionHandler` trait:

```rust
pub trait ProvisionHandler {
    /// The manifest section this handler owns (e.g. "packages", "nvm").
    fn section_name(&self) -> &'static str;

    /// Check whether this capability is already satisfied.
    fn check(&self, os: Os, ctx: &ProvisionContext) -> Result<CheckResult>;

    /// Converge the machine toward the declared state.
    fn provision(&self, os: Os, ctx: &ProvisionContext) -> Result<Vec<ProvisionResult>>;
}
```

`provision()` iterates declared capabilities in order, dispatching each to its
handler. Adding a capability = new struct + new handler impl + new skill. The
dispatch loop and check-before-act contract is in the engine, not repeated per
handler.

### 3. No-clobber contract (idempotency as architecture)

Every handler must implement `check()` before `provision()`. The engine
provides a unified rule set:

- **File exists with matching content → satisfied, skip**
- **File exists with different content, setmeup-managed → warn, preserve user
  changes, report "divergent"**
- **File exists with different content, unmanaged (no marker) → skip, report
  "user-managed"**
- **File absent → create from declared content**

Config merge (per strategy):
- `create-only` — never touch an existing file
- `overwrite-managed` — overwrite files with the setmeup marker comment
- `merge` — blend sections line-by-line, flag conflicts for user resolution

Default strategy for all new capabilities: **check first, skip on presence,
never silently clobber.**

### 4. Linear crossterm wizard (replaces ratatui stub)

- Full-screen steps with progress bar
- 6 wizard steps: Welcome → Credentials → Packages → Repos → Shell → Review
- Each step renders as: progress bar (top) + instruction text + input area
- Defaults-first: pre-fill from existing manifest, accept-on-Enter
- Spinner for async operations (credential validation, repo visibility check)
- ratatui dependency removed; crossterm (already declared) + minimal ANSI

### 5. Agent skill architecture for capability extension

Each manifest capability gets a matching skill under `.opencode/skills/`:

```
manifest-<capability>/SKILL.md
  ├── Research (how to fetch docs, version info)
  ├── Install (methods, verification)
  ├── Config (format, location, merge rules)
  ├── Idempotency (check-before-act logic)
  └── Wizard (what the TUI screen collects)
```

The "add a capability" workflow becomes:
1. Create a GitHub issue describing the tool/behavior
2. Write `manifest-<tool>/SKILL.md` (research the ecosystem)
3. Define the YAML schema block in `manifest.rs`
4. Implement `ProvisionHandler` in `provisioning/`
5. Add wizard step in `tui/wizard/`
6. Wire into manifest + provisioning dispatch
7. Verify with the done-gate

This workflow is documented as `docs/ADDING-A-CAPABILITY.md` — the recipe.

## Capabilities

### New
- Manifest schema V2 (packages, nvm, rustup, repos, aliases, configs, shell sections)
- `ProvisionHandler` trait + capability dispatch engine
- No-clobber contract with `check()` before `provision()`
- Linear crossterm wizard (full-screen, progress bar, 6 steps)
- Agent skill architecture for capability research + implementation
- `docs/ADDING-A-CAPABILITY.md` recipe
- `manifest-<name>/*` skill templates for the first wave of capabilities
  (packages, nvm, rustup, repos, aliases, configs, shell)

### Modified
- `cli` — `configure` command launches the new linear wizard
- `tui` — `run_wizard()` becomes the real 6-step interactive flow;
  `render_apply_dashboard()` stays as-is
- `manifest` — grows new sections; `Manifest` struct adds `ProvisionHandler`
  dispatch
- `provisioning` — `provision()` delegates to handler list instead of inline
  match; per-capability handler modules in `provisioning/` directory
- `.opencode/skills/` — new `manifest-*` skills
- `Cargo.toml` — remove `ratatui` dependency

### Removed
- `anyhow` dependency (dead, never imported)
- `ratatui` dependency (unused, replaced by crossterm wizard)
- Dead-code stubs in `main.rs` (`_manifest_backend_to_secrets`,
  `_secret_acquire_to_hint`, `_is_tty`, `_flush`) — replaced by wiring or
  properly removed

## Impact

- `src/manifest.rs` — new structs + `ProvisionHandler` integration (≈+200 loc)
- `src/provisioning.rs` — refactored to handler dispatch (≈+150 loc)
- `src/provisioning/packages.rs` — new handler module
- `src/provisioning/nvm.rs` — new handler module
- `src/provisioning/rustup.rs` — new handler module
- `src/provisioning/repos.rs` — new handler module
- `src/provisioning/aliases.rs` — new handler module
- `src/provisioning/configs.rs` — new handler module
- `src/provisioning/shell.rs` — new handler module
- `src/tui.rs` — refactored to wizard module + steps (≈+400 loc)
- `src/tui/wizard.rs` — wizard orchestrator
- `src/tui/steps.rs` — per-step rendering
- `src/tui/dashboard.rs` — existing dashboard (unchanged)
- `src/main.rs` — remove dead-code stubs
- `Cargo.toml` — remove ratatui, remove anyhow
- `.opencode/skills/manifest-*/SKILL.md` — new skills (7 files)
- `docs/ADDING-A-CAPABILITY.md` — new recipe
- `openspec/specs/manifest/spec.md` — updated schema spec
- `openspec/specs/provisioning/spec.md` — updated capability dispatch spec
- `openspec/specs/tui/spec.md` — updated wizard spec
- `openspec/specs/agent-skills/spec.md` — new spec for skill architecture

## Dependencies

- No new crate dependencies. Crossterm is already declared. All wizard
  rendering uses crossterm + std library I/O.

## Risks

- **Over-engineering the trait dispatch** → Mitigation: `ProvisionHandler` is
  simple (check + provision), the dispatch loop is a for-loop over a vec. No
  plugin system, no dynamic registration, no reflection.
- **Wizard complexity exceeds value** → Mitigation: linear, not tree-based.
  Each step is one screen with one or two inputs. No nested sub-wizards.
- **Handler proliferation creates too many small files** → Not a risk for a
  personal tool. If it becomes one, handlers can merge into
  `provisioning/capabilities.rs`.
- **Breaking existing manifests when "tools" → "packages"** → Mitigation:
  backward-compat shim: if `tools` is present, desugar to `packages`. Removed
  after one release cycle.
