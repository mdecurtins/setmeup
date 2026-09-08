# manifest Specification

## Purpose
TBD - created by archiving change setmeup-core. Update Purpose after archive.
## Requirements
### Requirement: Declarative manifest document
The system SHALL define machine desired state in a `manifest.yml` document covering tools, dotfiles, shell preferences, OS preferences, and secret-acquisition policy.

#### Scenario: Manifest parses and validates
- **WHEN** a user provides a well-formed manifest.yml
- **THEN** the manifest parses and validates against the schema

#### Scenario: Invalid manifest rejected
- **WHEN** a user provides a malformed or schema-violating manifest
- **THEN** setmeup reports an error naming the offending section and refuses to apply

### Requirement: Manifest is portable across OSes
The manifest SHALL express targets in an OS-independent way, with per-OS resolution applied at provisioning time.

#### Scenario: Same manifest on Ubuntu and macOS
- **WHEN** the same manifest is applied on both Ubuntu and macOS
- **THEN** each machine resolves the tool/path to its platform-specific equivalent

### Requirement: Secrets policy declared, not stored
The manifest SHALL declare how each credential is to be acquired (`paste`, `device-flow`, or `env`) and SHALL NOT contain secret material itself.

#### Scenario: Manifest contains a secret-acquisition policy
- **WHEN** a user views a manifest declaring a credential
- **THEN** it specifies the acquisition method and target backend, and contains no token or key material

### Requirement: Secret material excluded from git
The system SHALL architecturally prevent secret material from being written into git-tracked paths.

#### Scenario: Secret write path
- **WHEN** setmeup stores or reads a credential
- **THEN** it only writes to vault or backup paths outside the git repository

### Requirement: Defaults and sections
The manifest SHALL support defaulting so that absent optional sections resolve to sensible per-OS defaults.

#### Scenario: Empty manifest applies cleanly
- **WHEN** a user applies a minimal manifest with no optional sections
- **THEN** the resolve step produces a valid per-OS effective manifest using defaults

