# identity Specification

## Purpose
TBD - created by archiving change setmeup-core. Update Purpose after archive.
## Requirements
### Requirement: Fresh keys per physical machine
The system SHALL generate a fresh SSH keypair and a fresh GPG signing key on first identity setup, keyed to the physical machine rather than shared across machines.

#### Scenario: First identity setup
- **WHEN** identity runs on a machine with no existing keys
- **THEN** an ed25519 SSH keypair and a GPG signing key are generated locally
- **NOTE** — GPG signature registration with providers is deferred to follow-up #21; generation is implemented, registration is not part of this change's contract.

#### Scenario: Existing keys honored
- **WHEN** identity runs on a machine that already has keys
- **THEN** no new key is generated and the existing keys are used

### Requirement: Public-key registration
The system SHALL register the SSH public key with configured hosting providers (GitHub, GitLab) using a stored token. (GPG fingerprint registration is deferred to follow-up #21.)

#### Scenario: SSH pubkey registered
- **WHEN** a fresh SSH key is generated and a provider token is available
- **THEN** the public key is added to the provider account

#### Scenario: Already-registered key not duplicated
- **WHEN** the public key is already present on the provider account
- **THEN** registration is skipped

### Requirement: Credentials never re-created on re-run
Identity setup SHALL be idempotent: re-runs must never produce a second keypair or duplicate registrations. The SSH path is idempotent by construction (no-op when a key exists; duplicate registration skipped); the re-run idempotency TEST is deferred to follow-up #22.

#### Scenario: Re-run after setup
- **WHEN** identity runs again after a completed setup
- **THEN** the original keys are detected and no new keys or registrations are created

### Requirement: WSL2 key survival
The system SHALL back up SSH keys to the Windows host (age-encrypted) so they survive WSL2 distro recreation on the same physical machine.

#### Scenario: Backup created on first setup
- **WHEN** SSH keys are first generated inside WSL2
- **THEN** an age-encrypted backup is written to the Windows host filesystem

#### Scenario: Restore on fresh distro
- **WHEN** identity runs inside a fresh WSL2 distro and a backup exists on the Windows host
- **THEN** the keys are restored from the backup before any generation is considered

#### Scenario: Successful restore does not recreate keys
- **WHEN** keys are restored from the Windows-host backup
- **THEN** registration verifies the restored pubkey and only registers it if missing

### Requirement: Restore-before-generate ordering
Identity setup SHALL attempt restore from any available backup before generating new keys.

#### Scenario: Backup absent
- **WHEN** identity runs and no backup exists
- **THEN** new keys are generated and (on WSL2) a new backup is written

#### Scenario: Backup present
- **WHEN** identity runs and a backup exists
- **THEN** keys are restored and no new keypair is generated

### Requirement: Key permissions
The system SHALL set restrictive permissions on private key material.

#### Scenario: Private key permissions
- **WHEN** a private key is generated or restored
- **THEN** its file mode is owner-readable/writable only (e.g. 600)

### Requirement: Windows host encryption
The system SHALL encrypt the Windows-host key backup with the vault passphrase, never storing plaintext key material on the Windows filesystem.

#### Scenario: Backup contents encrypted
- **WHEN** a backup on the Windows host is inspected
- **THEN** its contents are not readable without the passphrase

