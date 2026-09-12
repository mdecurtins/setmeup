## ADDED Requirements

### Requirement: Capability dispatch

The `provision()` function SHALL iterate over declared capabilities in a fixed
order, dispatching each to its `ProvisionHandler`.

#### Scenario: Capabilities provisioned in order

- **WHEN** a manifest declares shell + packages + repos
- **THEN** packages are provisioned first, then repos, then shell

#### Scenario: Missing sections are skipped

- **WHEN** a manifest has no `nvm` section
- **THEN** the nvm handler is not called

#### Scenario: Handler error does not halt the loop

- **WHEN** one handler returns an error
- **THEN** the error is collected and reported, but subsequent handlers still
  execute (matching existing failed-item-continue behavior)

### Requirement: Check before provision

Every handler SHALL implement `check()` and `provision()`. The engine SHALL
call `check()` first and only call `provision()` when the result is
`NeedsProvision`.

#### Scenario: Satisfied capability is skipped

- **WHEN** `check()` returns `Satisfied` for a capability
- **THEN** `provision()` is not called; the capability is reported as satisfied

#### Scenario: Divergent state is reported, not overwritten

- **WHEN** `check()` returns `Divergent` (exists but differs from declared)
- **THEN** the handler reports the divergence and skips provisioning; no
  mutation occurs

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

- **WHEN** a handler's `check()` and `provision()` are called directly in a
  test with a mock `ProvisionContext`
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
8. identity      (SSH/GPG — existing, unchanged ordering)
9. secrets       (credential backend — existing, unchanged ordering)
```

#### Scenario: Dotfiles applied during shell step

- **WHEN** a manifest declares dotfiles
- **THEN** they are linked during the shell step (after PATH/env config)
