## 1. Project Scaffold

- [ ] 1.1 Initialize Cargo project (`cargo init`) with binary name `setmeup`, workspace layout for future crates
- [ ] 1.2 Add core dependencies: `clap` (CLI), `ratatui` + `crossterm` (TUI), `serde` + `serde_yaml` (manifest), `age` (encryption), `keyring` (OS vaults), `anyhow`/`thiserror` (errors)
- [ ] 1.3 Add `--version`, `--help`, and subcommand skeleton (`configure`, `apply`, `status`, `diff`, `update`, `secrets`, `keys`) with clap
- [ ] 1.4 Set up CI (GitHub Actions): fmt, clippy, test on ubuntu-latest, macos-latest, windows-latest
- [ ] 1.5 Capture remaining project conventions (Rust edition, lint config) in `Cargo.toml` / `rustfmt.toml`

## 2. Manifest Model

- [ ] 2.1 Define `Manifest` types (tools, dotfiles, shell config, OS preferences, secret policy) with serde derive
- [ ] 2.2 Implement `manifest.yml` parse + validate, with schema errors named by section
- [ ] 2.3 Implement per-OS resolution: target → apt/brew/winget conventions, defaulting for absent sections
- [ ] 2.4 Load/write manifest from `~/.config/setmeup/manifest.yml`; support explicit path override
- [ ] 2.5 Unit-test parse/validate, defaults, and per-OS resolution across fake Ubuntu/macOS/Windows targets

## 3. CLI Core

- [ ] 3.1 Implement `apply` with dry-run flag reporting intended per-item actions
- [ ] 3.2 Implement `status` reporting satisfied/pending/failed per manifest item
- [ ] 3.3 Implement `diff` producing a no-mutation report of divergent items
- [ ] 3.4 Add `--json` machine-readable output to status/diff/apply summary
- [ ] 3.5 Add exit codes: 0 clean, non-zero on item failure, distinct non-zero on invalid manifest

## 4. Secret Backend Abstraction

- [ ] 4.1 Define `SecretBackend` trait: set/get/delete/list, with encrypted-file and keyring implementations
- [ ] 4.2 Implement age-encrypted-file backend (`~/.config/setmeup/secrets.age`), passphrase unlocked
- [ ] 4.3 Implement OS keyring backends via `keyring` crate (Secret Service / Keychain / Credential Manager)
- [ ] 4.4 Implement backend auto-selection (strongest reachable backend) at secret-write time
- [ ] 4.5 Implement `secrets list` (names only, no values) and `secrets migrate --to` across backends
- [ ] 4.6 Test: file encryption/decryption round-trip, no-plaintext guarantee, migration preserves all entries

## 5. Credential Acquisition

- [ ] 5.1 Implement `SecretAcquire` dispatch on manifest policy: `paste`, `device-flow`, `env`
- [ ] 5.2 Implement masked paste prompt (TUI)
- [ ] 5.3 Implement GitHub OAuth device-flow acquisition (PKCE-less device grant, URL + code, polling)
- [ ] 5.4 Store acquired secrets via active backend; never via git-tracked paths (enforcement test)

## 6. TUI

- [ ] 6.1 Implement wizard screens: identity, credential policy, tool selection, dotfiles, shell, review
- [ ] 6.2 Implement defaults-first prompting (accept on Enter) for every configurable setting
- [ ] 6.3 Implement credential-policy-rendered prompts (paste vs device-flow)
- [ ] 6.4 Implement live apply dashboard: per-item running/succeeded/failed, final summary
- [ ] 6.5 Wire `configure` → wizard writes manifest.yml; wire `apply` → optional dashboard

## 7. Identity Lifecycle

- [ ] 7.1 Implement SSH ed25519 generation to `~/.ssh` with 600 perms; detect existing keys
- [ ] 7.2 Implement GPG signing-key generation; register fingerprint lazily
- [ ] 7.3 Implement provider registration (GitHub/GitLab API) using stored token; skip if pubkey/fingerprint already present
- [ ] 7.4 Implement restore-before-generate: check `~/.ssh`, then Windows-host backup, before generating
- [ ] 7.5 Implement WSL2 Windows-host backup: age-encrypted `ssh-keys.age` under `/mnt/c/Users/<you>/.setmeup/backups/`
- [ ] 7.6 Implement restore from backup (decrypt, chmod 600, verify registration, re-register only if missing)
- [ ] 7.7 Idempotency test: re-run creates no new keys or duplicate registrations

## 8. Provisioning Backends

- [ ] 8.1 Implement backend trait: presence check + install per platform
- [ ] 8.2 apt backend (Ubuntu/Debian): `dpkg -s` check, `apt-get install`
- [ ] 8.3 Homebrew backend (macOS): `brew list` check, `brew install`
- [ ] 8.4 winget backend (native Windows): `winget list` check, `winget install`
- [ ] 8.5 Implement dotfile symlinking (`ln -sf` semantics, conflict detection without silent overwrite)
- [ ] 8.6 Implement shell rc idempotent merge (line-presence check before append)
- [ ] 8.7 Implement failed-item-continue: apply proceeds past failures, reports at end

## 9. Bootstrap

- [ ] 9.1 Write bootstrap script: OS detection, branch per platform (ubuntu/mac/windows)
- [ ] 9.2 WSL2 bootstrap: install rustup if absent, `cargo install --git`, launch wizard
- [ ] 9.3 Windows-native branch: install WSL2 if absent, then run Unix path inside it
- [ ] 9.4 Verify bootstrap on a bare Ubuntu/WSL2 container (no prior tooling) end-to-end

## 10. Self-Update

- [ ] 10.1 Implement `setmeup update`: fetch source, rebuild, replace running binary path
- [ ] 10.2 Verify update-then-apply stays converged (idempotent apply after binary change)

## 11. End-to-End Verification

- [ ] 11.1 Run `configure` → `apply` → `status`/`diff` on a fresh WSL2 distro
- [ ] 11.2 Re-run `apply` twice and confirm second run is a no-op
- [ ] 11.3 Blow away distro, re-run identity, confirm SSH keys restored from Windows-host backup
- [ ] 11.4 Confirm nothing secret is written into git-tracked paths during full flow
- [ ] 11.5 Products CI green on all three OS runners