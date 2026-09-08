# secrets Specification

## Purpose
TBD - created by archiving change setmeup-core. Update Purpose after archive.
## Requirements
### Requirement: Backend abstraction
The system SHALL expose secret storage behind a backend abstraction, so the active backend can change without changing secret consumers.

#### Scenario: Backend swapped without consumer change
- **WHEN** secrets migrate from the encrypted-file backend to a keyring backend
- **THEN** secret-reading operations continue to work unchanged

### Requirement: Encrypted-file backend as the floor
The system SHALL provide an age-encrypted file backend that works on a headless fresh machine with no OS keyring available.

#### Scenario: No keyring reachable
- **WHEN** setmeup runs where no OS keyring or secret service is reachable
- **THEN** secrets are stored in an age-encrypted file unlocked by a passphrase

#### Scenario: File is encrypted at rest
- **WHEN** the encrypted-file backend is active
- **THEN** the file contents are not readable without the passphrase

### Requirement: No plaintext fallback
The system SHALL NOT store secrets in plaintext when no keyring is available.

#### Scenario: Keyring absent
- **WHEN** the OS provides no keyring and the user stores a secret
- **THEN** the secret is only ever persisted in encrypted form

### Requirement: OS keyring backends
The system SHALL support the Secret Service on Linux, Keychain on macOS, and Credential Manager on native Windows as secret backends.

#### Scenario: Linux keyring
- **WHEN** a Linux Secret Service is reachable
- **THEN** secrets can be stored and retrieved through it

#### Scenario: macOS Keychain
- **WHEN** setmeup runs on macOS
- **THEN** secrets can be stored and retrieved through the Keychain

### Requirement: Migration between backends
The system SHALL migrate stored secrets between backends via `setmeup secrets migrate --to <backend>`.

#### Scenario: Migrate file to keyring
- **WHEN** a user migrates secrets from the encrypted file to the OS keyring
- **THEN** every stored secret is readable from the new backend and the old artifact no longer holds the plaintext-equivalent state used by setmeup

#### Scenario: No secrets to migrate
- **WHEN** a user runs migrate with an empty store
- **THEN** the command reports nothing to migrate and succeeds

### Requirement: Backend auto-selection
The system SHALL choose the strongest backend available on the current machine at secret-write time.

#### Scenario: Keyring appears after file backend
- **WHEN** a keyring becomes reachable on a machine that used the encrypted file
- **THEN** subsequent secret writes use the keyring backend

### Requirement: Secret listing without plaintext
The system SHALL list stored secret names without revealing their values.

#### Scenario: secrets list
- **WHEN** a user runs `setmeup secrets list`
- **THEN** secret names are shown and values are not

