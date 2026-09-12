# provisioning Specification

## Purpose

Provision the machine toward the declared manifest state. Each capability is handled by a `ProvisionHandler` that checks before acting. The provisioning loop is ordered, idempotent, and never silently clobbers existing state.

## Requirements

### Requirement: Capability dispatch
The `provision()` function SHALL iterate over declared capabilities in a fixed order, dispatching each to its `ProvisionHandler`.

#### Scenario: Capabilities provisioned in order
- **WHEN** a manifest declares shell + packages + repos
- **THEN** packages are provisioned first, then repos, then shell

#### Scenario: Missing sections are skipped
- **WHEN** a manifest has no `nvm` section
- **THEN** the nvm handler is not called

#### Scenario: Handler error does not halt the loop
- **WHEN** one handler returns an error
- **THEN** the error is collected and reported, but subsequent handlers still execute

#### Scenario: Deferred section surfaces as visible failure
- **WHEN** a declared section (e.g. packages) has no implemented handler yet
- **THEN** the item is reported as failed with a "not yet implemented" detail rather than silently skipped

### Requirement: Check before provision
Every handler SHALL implement `check()` and `provision()`. The engine SHALL call `check()` first and only call `provision()` when the result is `NeedsProvision`.

#### Scenario: Satisfied capability is skipped
- **WHEN** `check()` returns `Satisfied` for a capability
- **THEN** `provision()` is not called; the capability is reported as satisfied

#### Scenario: Divergent state is reported, not overwritten
- **WHEN** `check()` returns `Divergent` (exists but differs from declared)
- **THEN** the handler reports the divergence and skips provisioning; no mutation occurs

#### Scenario: User-managed state is reported, not touched
- **WHEN** `check()` returns `ManagedByUser` (exists without setmeup marker)
- **THEN** the handler reports "user-managed, skipped"; no mutation occurs

### Requirement: ProvisionHandler trait
The trait SHALL be defined as:

```rust
pub enum CheckResult {
    Satisfied,
    NeedsProvision,
    Divergent(String),
    ManagedByUser,
}

pub struct ProvisionContext {
    pub repo_root: PathBuf,
    pub home: PathBuf,
    pub dry_run: bool,
}

pub trait ProvisionHandler: Debug {
    fn section_name(&self) -> &'static str;
    fn check(&self, os: Os, ctx: &ProvisionContext) -> Result<CheckResult>;
    fn provision(&self, os: Os, ctx: &ProvisionContext) -> Result<Vec<ProvisionResult>>;
}
```

#### Scenario: Handler is unit-testable independently
- **WHEN** a handler's `check()` and `provision()` are called directly in a test with a mock `ProvisionContext`
- **THEN** they behave identically to when called through the dispatch loop

### Requirement: Provisioning order
The fixed provisioning order SHALL be:

```
1. packages      (system packages and tool installations)
2. nvm           (node version manager)
3. rustup        (rust toolchain)
4. repos         (git repository cloning)
5. aliases       (shell aliases, may reference repo paths)
6. configs       (per-tool configuration files)
7. shell         (functions, prompt, PATH, env, sources)
8. identity      (SSH/GPG)
9. secrets       (credential backend)
```

Cross-capability dependency graphs (e.g., "clone this repo before installing this package") are NOT supported; the `depends` field within `packages` works only for ordering within that section.

#### Scenario: Sections provisioned in declared order
- **WHEN** a manifest declares multiple sections
- **THEN** results appear in the fixed order above

### Requirement: Per-OS package backends
The system SHALL support package installation via apt (Ubuntu/Debian), Homebrew (macOS), and winget (native Windows), selecting the backend by platform.

#### Scenario: Install on Ubuntu
- **WHEN** apply runs on Ubuntu
- **THEN** declared tools are installed via apt

#### Scenario: Install on macOS
- **WHEN** apply runs on macOS
- **THEN** declared tools are installed via Homebrew

#### Scenario: Install on native Windows
- **WHEN** apply runs on native Windows
- **THEN** declared tools are installed via winget

### Requirement: Presence-checked installation
Backends SHALL check whether a tool is already installed before installing it, so apply is a no-op for satisfied items.

#### Scenario: Tool already installed
- **WHEN** apply encounters a tool already present via the backend's query
- **THEN** installation is skipped and the item reports satisfied

#### Scenario: Tool missing
- **WHEN** apply encounters a tool absent via the backend's query
- **THEN** the tool is installed and the item reports satisfied on success

### Requirement: Dotfile application
The system SHALL apply dotfiles from the repo into the user home directory using symlinks with overwrite-safe semantics.

#### Scenario: Symlink created
- **WHEN** apply runs and a managed dotfile is absent from the home directory
- **THEN** a symlink to the repo copy is created

#### Scenario: Existing symlink preserved on re-run
- **WHEN** apply runs again with the same manifest
- **THEN** the existing symlink is left unchanged

#### Scenario: Conflicting regular file detected
- **WHEN** a regular file blocks a managed dotfile location
- **THEN** apply reports the conflict and does not overwrite it silently

### Requirement: Shell configuration application
The system SHALL append or merge declared shell configuration (aliases, exports, plugins) into the user's shell rc file idempotently.

#### Scenario: Shell line added once
- **WHEN** apply runs and a declared shell configuration line is absent
- **THEN** the line is added to the shell rc file

#### Scenario: Shell line not duplicated
- **WHEN** apply runs again and the declared line already exists
- **THEN** no duplicate line is added

### Requirement: Failed items do not halt apply
The system SHALL continue applying remaining manifest items when one item fails, and report the failures at the end.

#### Scenario: One item fails among many
- **WHEN** a single item fails during apply
- **THEN** remaining items still run and the final report lists the failure

### Requirement: Idempotent convergence
Repeated application of the same manifest SHALL produce no additional system changes once the machine has converged.

#### Scenario: Second apply is a no-op
- **WHEN** a converged machine runs apply a second time
- **THEN** no installation, symlink, or config action executes