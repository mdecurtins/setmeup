## ADDED Requirements

### Requirement: CLI command surface
The setmeup CLI SHALL provide the commands `configure`, `apply`, `status`, `diff`, `update`, `secrets`, and `keys`, each with help output.

#### Scenario: Help is available
- **WHEN** a user runs `setmeup --help` or `setmeup <command> --help`
- **THEN** the CLI prints usage and exits successfully

#### Scenario: Unknown command errors clearly
- **WHEN** a user runs an unknown subcommand
- **THEN** the CLI prints an error identifying the unknown command and exits non-zero

### Requirement: configure launches the TUI
The `configure` command SHALL launch the manifest-authoring wizard.

#### Scenario: configure without arguments
- **WHEN** a user runs `setmeup configure`
- **THEN** the TUI wizard opens on the current manifest (or a blank one if none exists)

### Requirement: apply reconciles the machine
The `apply` command SHALL converge the machine toward the declared manifest and be safe to re-run.

#### Scenario: apply on a fresh machine
- **WHEN** a user runs `setmeup apply` on a machine that does not match the manifest
- **THEN** the machine converges toward the manifest and the command exits successfully

#### Scenario: apply on an already-converged machine
- **WHEN** a user runs `setmeup apply` twice on a converged machine
- **THEN** the second run makes no changes and exits successfully

### Requirement: status reports convergence
The `status` command SHALL report which manifest items are satisfied, pending, or failed.

#### Scenario: status on a partial machine
- **WHEN** a user runs `setmeup status` on a partially provisioned machine
- **THEN** each manifest item is reported as satisfied, pending, or failed

### Requirement: diff shows pending changes
The `diff` command SHALL show the difference between the current machine state and the manifest without applying changes.

#### Scenario: diff on a non-converged machine
- **WHEN** a user runs `setmeup diff`
- **THEN** only the divergent items are printed and nothing is modified

### Requirement: machine-readable output
Subcommands SHALL support a machine-readable output mode (e.g. `--json`) for scripting.

#### Scenario: status as JSON
- **WHEN** a user runs `setmeup status --json`
- **THEN** the report is emitted as valid structured JSON on stdout

### Requirement: dry-run before apply
The `apply` command SHALL support a dry-run mode that reports intended actions without performing them.

#### Scenario: apply --dry-run
- **WHEN** a user runs `setmeup apply --dry-run`
- **THEN** the intended actions are printed and no system changes are made