# bootstrap (delta) — modified by dev-hooks

## MODIFIED Requirements

### Requirement: Bootstrap script fetches and runs the tool
The system SHALL provide a thin bootstrap launcher that gets the setmeup binary onto a fresh machine and hands off to the first-run wizard. **When bootstrapping in a setmeup development clone (positively identified, not merely "some git worktree"), the bootstrap SHALL also ensure the repo-pinned git hooks are installed via `scripts/install-hooks.sh`, so a fresh development environment starts with the local gates active.**

#### Scenario: Bootstrap on a bare Ubuntu/WSL2 machine
- **WHEN** a user runs `curl <repo>/bootstrap.sh | sh` on a fresh Ubuntu (WSL2) install with no prior tooling
- **THEN** the script detects the OS, installs rustup/cargo if absent, builds setmeup from source, and launches the first-run wizard

#### Scenario: Bootstrap on macOS
- **WHEN** a user runs the bootstrap script on a fresh macOS install
- **THEN** the script detects macOS, installs the Rust toolchain if absent, builds setmeup, and launches the first-run wizard

#### Scenario: Bootstrap in a setmeup repo clone
- **WHEN** bootstrap runs inside a working directory that is positively identified as the setmeup repository (git worktree AND `git remote get-url origin` matches `github.com/mdecurtins/setmeup` AND a setmeup marker file exists)
- **THEN** `scripts/install-hooks.sh` is invoked and the repo-pinned hooks are active

#### Scenario: Bootstrap inside an unrelated git worktree
- **WHEN** the `curl | sh` bootstrap runs in a directory that is a git worktree but is NOT the setmeup repository (remote URL or marker mismatch)
- **THEN** the script does NOT run `install-hooks.sh`
- **THEN** the unrelated worktree's hooks are untouched

### Requirement: Bootstrap script contains no provisioning logic
The bootstrap script SHALL contain no provisioning intelligence; it only delivers the binary and launches the wizard. **The dev-clone hook installation is explicitly carved out of this requirement: when a clone is detected, the script SHALL run `scripts/install-hooks.sh`, which is development-environment setup, not product provisioning.**

#### Scenario: Provisioning decisions live in the binary
- **WHEN** a user inspects the bootstrap script
- **THEN** the script does not install tools, write configuration, or manage credentials beyond acquiring the tool itself
- **THEN** the only exception is the explicitly carved-out setmeup-repo hook installation, which runs `scripts/install-hooks.sh` only when the working directory is positively identified as the setmeup repository
