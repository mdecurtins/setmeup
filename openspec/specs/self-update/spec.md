# self-update Specification

## Purpose
TBD - created by archiving change setmeup-core. Update Purpose after archive.
## Requirements
### Requirement: Self-update command
The system SHALL provide `setmeup update` that pulls the latest tool source and rebuilds itself.

#### Scenario: Update to latest
- **WHEN** a user runs `setmeup update`
- **THEN** the tool fetches the latest source and rebuilds the binary

#### Scenario: Update is safe to re-run
- **WHEN** `setmeup update` runs twice in a row
- **THEN** the second run completes without error

### Requirement: Newer binary converges safely
Applying with a newly updated binary SHALL converge toward the same declared manifest rather than mutating state unexpectedly.

#### Scenario: Update then apply
- **WHEN** a user updates setmeup and runs apply on a converged machine
- **THEN** apply remains a no-op for satisfied items

### Requirement: Version reporting
The system SHALL report its installed version.

#### Scenario: version flag
- **WHEN** a user runs `setmeup --version`
- **THEN** the current version string is printed and the command exits successfully

