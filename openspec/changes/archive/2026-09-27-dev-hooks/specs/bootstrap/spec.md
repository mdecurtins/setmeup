# bootstrap (delta) — modified by dev-hooks

## MODIFIED Requirements

### Requirement: Bootstrap script fetches and runs the tool
The system SHALL provide a thin bootstrap launcher that gets the setmeup binary onto a fresh machine and hands off to the first-run wizard. **The delivery bootstrap (`curl <repo>/bootstrap.sh | sh`) SHALL NOT install, execute, or otherwise touch anything in the caller's working directory — specifically it SHALL NOT run `scripts/install-hooks.sh` or any local repository file. The git-hooks change's hook installation is an explicit developer action in their own clone, never a side effect of the product-delivery bootstrap.**

#### Scenario: Bootstrap on a bare Ubuntu/WSL2 machine
- **WHEN** a user runs `curl <repo>/bootstrap.sh | sh` on a fresh Ubuntu (WSL2) install with no prior tooling
- **THEN** the script detects the OS, installs rustup/cargo if absent, builds setmeup from source, and launches the first-run wizard

#### Scenario: Bootstrap on macOS
- **WHEN** a user runs the bootstrap script on a fresh macOS install
- **THEN** the script detects macOS, installs the Rust toolchain if absent, builds setmeup, and launches the first-run wizard

#### Scenario: Delivery bootstrap never executes local files
- **WHEN** a user runs the `curl | sh` bootstrap in any directory (including a setmeup clone or a malicious/unrelated worktree)
- **THEN** the bootstrap does not invoke `install-hooks.sh` and does not execute any file from the working directory
- **THEN** hooks are installed only when the developer explicitly runs the install step themselves

### Requirement: Bootstrap script contains no provisioning logic
The bootstrap script SHALL contain no provisioning intelligence; it only delivers the binary and launches the wizard.

#### Scenario: Provisioning decisions live in the binary
- **WHEN** a user inspects the bootstrap script
- **THEN** the script does not install tools, write configuration, or manage credentials beyond acquiring the tool itself
- **THEN** the script does not scan for or detect a local clone, and does not execute repository files from the current directory