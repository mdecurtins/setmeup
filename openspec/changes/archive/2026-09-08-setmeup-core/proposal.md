## Why

Every fresh OS install (Ubuntu on WSL2, macOS, native Windows, other *nix) currently requires hours of manual re-setup: installing tools, recreating dotfiles, re-establishing credentials. `setmeup` is a personal, declarative provisioning tool that converges any new machine to a defined desired state — automatically, idempotently, and with minimal human intervention. This change establishes the core tool foundation.

## What Changes

- Introduce `setmeup`, a Rust CLI + TUI application that provisions a fresh OS environment to a declared configuration.
- **Bootstrap:** a thin cross-platform launcher script that gets the `setmeup` binary onto a bare machine, then hands off to the wizard. Default delivery path is source-build via `cargo install --git`; prebuilt releases and native package manager paths are escape hatches, not defaults.
- **Declarative manifest:** the desired state (tools, dotfiles, shell config, OS preferences, secret policy) lives in a `manifest.yml` that is portable and versioned. The TUI edits the manifest; it does not run steps.
- **Apply/status/diff semantics:** `setmeup apply` reconciles the machine toward the manifest; re-running is a no-op when state converges.
- **Credential vault with a migration ladder:** secrets start in an `age`-encrypted file (works on a headless fresh WSL2) and migrate up to the OS keyring/keychain/credential manager as the machine grows. `migrate` is a first-class command.
- **Identity lifecycle:** SSH keys and GPG signing keys are fresh per machine, registered once against GitHub/GitLab via a stored token. On WSL2, SSH keys are age-encrypted and backed up to the Windows host so they survive distro wipes on the same physical machine.
- **Per-OS backends:** package installation via apt (Ubuntu/Debian), Homebrew (macOS), winget/choco (Windows), plus dotfile and shell-config application.
- **Self-update:** after bootstrap, `setmeup update` keeps the tool itself current.
- **TUI:** a wizard + dashboard combo for authoring the manifest and running/observing provisioning.

## Capabilities

### New Capabilities
- `bootstrap`: get the `setmeup` binary onto a fresh machine with zero prior tooling, then hand off to first-run wizard
- `cli`: command surface — `configure` (TUI), `apply`, `status`, `diff`, `update`, `secrets`, `keys`, `migrate` — with consistent machine-readable output
- `manifest`: the declarative desired-state document (`manifest.yml`): schema, validation, defaults, per-OS resolution
- `tui`: wizard + dashboard for authoring the manifest and observing apply runs
- `provisioning`: reconcile engine and per-OS backends (apt, Homebrew, winget, dotfiles, shell config) that converge the machine toward the manifest idempotently
- `secrets`: credential store with migration ladder — age-encrypted file first, OS keyring/keychain/Credential Manager second, optional external vault (1Password/Bitwarden) later
- `identity`: SSH + GPG key lifecycle — fresh keys per physical machine, GitHub/GitLab registration, and WSL2 cross-distro backup/restore via the Windows host
- `self-update`: keep the installed binary current after bootstrap, safe because apply is declarative and idempotent

### Modified Capabilities
- (none — greenfield)

## Impact

- **New Rust project** in this repository (single binary, `cargo`; `.gitignore` already prepared for Cargo artifacts).
- **Cross-platform behavior:** Ubuntu/WSL2 is the default and best-tested path; macOS and native Windows supported (Windows steers users toward WSL2 if absent, and points away from Git Bash).
- **Public repo constraint:** the manifest and dotfiles are public; secrets and private key material must never enter git — enforced by the vault/backup design, not by convention.
- **Security-sensitive paths:** secret storage ladder, SSH key backup on the Windows host, token acquisition (paste vs OAuth device flow, chosen per-secret).
- **Deliberate non-goals of this personal tool:** no plugin system, no community schema discovery, no multi-user config, no telemetry, no distribution pipeline beyond what bootstrap needs.
- **Agentic development scaffold (AGENTS.md, skills, repo layout):** explicitly out of scope for this change; deferred and tracked as a TODO in `design.md`.