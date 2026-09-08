## ADDED Requirements

### Requirement: Bootstrap script fetches and runs the tool
The system SHALL provide a thin bootstrap launcher that gets the setmeup binary onto a fresh machine and hands off to the first-run wizard.

#### Scenario: Bootstrap on a bare Ubuntu/WSL2 machine
- **WHEN** a user runs `curl <repo>/bootstrap.sh | sh` on a fresh Ubuntu (WSL2) install with no prior tooling
- **THEN** the script detects the OS, installs rustup/cargo if absent, builds setmeup from source, and launches the first-run wizard

#### Scenario: Bootstrap on macOS
- **WHEN** a user runs the bootstrap script on a fresh macOS install
- **THEN** the script detects macOS, installs the Rust toolchain if absent, builds setmeup, and launches the first-run wizard

#### Scenario: Bootstrap on native Windows without WSL2
- **WHEN** a user runs the bootstrap script on native Windows where WSL2 is not set up
- **THEN** the script installs WSL2, then runs the Unix bootstrap path inside it

#### Scenario: Bootstrap on native Windows with WSL2
- **WHEN** a user runs the bootstrap script on native Windows where WSL2 already exists
- **THEN** the script runs the Unix bootstrap path inside the existing WSL2 distro

### Requirement: Bootstrap script contains no provisioning logic
The bootstrap script SHALL contain no provisioning intelligence; it only delivers the binary and launches the wizard.

#### Scenario: Provisioning decisions live in the binary
- **WHEN** a user inspects the bootstrap script
- **THEN** the script does not install tools, write configuration, or manage credentials beyond acquiring the tool itself

### Requirement: Rust toolchain installed automatically
The bootstrap SHALL install the Rust toolchain when it is missing on the target machine.

#### Scenario: Toolchain already present
- **WHEN** the target machine already has cargo available
- **THEN** the bootstrap skips toolchain installation and proceeds to building

### Requirement: Prebuilt binary as escape hatch
The system SHALL support installing from prebuilt release binaries as an alternative delivery path when compiling from source is impractical, while keeping source-build as the default.

#### Scenario: User chooses prebuilt download
- **WHEN** a user explicitly selects a prebuilt binary delivery
- **THEN** setmeup installs from the release artifact instead of compiling, without changing the first-run wizard handoff