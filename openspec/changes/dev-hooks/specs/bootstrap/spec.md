# bootstrap (delta) — modified by dev-hooks

## MODIFIED Requirements

### Requirement: Bootstrap script fetches and runs the tool
The system SHALL provide a thin bootstrap launcher that gets the setmeup binary onto a fresh machine and hands off to the first-run wizard. **When bootstrapping in a development context (a clone of the repository), the bootstrap SHALL also ensure the repo-pinned git hooks are installed via `scripts/install-hooks.sh`, so a fresh development environment starts with the local gates active.**

#### Scenario: Bootstrap on a bare Ubuntu/WSL2 machine
- **WHEN** a user runs `curl <repo>/bootstrap.sh | sh` on a fresh Ubuntu (WSL2) install with no prior tooling
- **THEN** the script detects the OS, installs rustup/cargo if absent, builds setmeup from source, and launches the first-run wizard

#### Scenario: Bootstrap on macOS
- **WHEN** a user runs the bootstrap script on a fresh macOS install
- **THEN** the script detects macOS, installs the Rust toolchain if absent, builds setmeup, and launches the first-run wizard

#### Scenario: Bootstrap in a repo clone
- **WHEN** bootstrap runs inside a development clone of the repository
- **THEN** `scripts/install-hooks.sh` is invoked and the repo-pinned hooks are active

### Requirement: Bootstrap script contains no provisioning logic
The bootstrap script SHALL contain no provisioning intelligence; it only delivers the binary and launches the wizard.

#### Scenario: Provisioning decisions live in the binary
- **WHEN** a user inspects the bootstrap script
- **THEN** the script does not install tools, write configuration, or manage credentials beyond acquiring the tool itself
- **THEN** any development-only hook installation is wired (when a clone is detected) without adding product provisioning logic