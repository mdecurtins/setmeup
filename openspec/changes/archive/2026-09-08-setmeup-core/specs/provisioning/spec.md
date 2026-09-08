## ADDED Requirements

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