## Context

`setmeup` is a personal, declarative provisioning tool. It runs first on a fresh OS install — primarily Ubuntu on WSL2 — and converges the machine to a defined desired state, idempotently, with minimal human intervention. It is hosted in a public GitHub repo but is built for one user; this constraint shapes every decision (no plugin system, no community schema, no telemetry, no distribution pipeline beyond what bootstrap needs).

The repo is currently greenfield: an OpenSpec scaffold, opsx skills, and a Cargo-prepared `.gitignore`. Nothing is implemented yet.

## Goals / Non-Goals

**Goals:**
- Bootstrap onto a bare machine (no prior tooling) with a thin launcher script
- Declarative desired-state manifest; the machine converges toward it
- True idempotency: `apply` twice converges; re-running never duplicates identity material
- Per-OS provisioning backends (apt, Homebrew, winget), with Ubuntu/WSL2 the best-tested default
- Credential storage that starts as an `age`-encrypted file and migrates up to the OS keyring/keychain/Credential Manager
- Fresh-per-physical-machine SSH and GPG identity, registered once, with WSL2 cross-distro survival via Windows-host backup
- A Rust TUI (wizard + dashboard) as the manifest authoring and apply-observation surface

**Non-Goals:**
- Productization: no community manifest schema, no plugin SDK, no multi-user config, no telemetry, no release-engineering CI matrix as a default
- Proving long-lived identity provenance (no persistent signing key)
- Cross-device credential portability: if the laptop dies, keys are gone; that is acceptable
- Agentic development scaffold (AGENTS.md, skills, agent-oriented repo layout) — explicit TODO, deferred

## Decisions

### 1. Declarative desired state over imperative step scripts
The manifest (`manifest.yml`) *is* the spec; the engine reconciles the machine toward it. Re-runs, diffs, dry-runs, and `status` fall out naturally: apply twice = no-op when converged. A step-script model would bury the "what" inside guarded commands and make idempotency a manual discipline.

### 2. Public repo, three-way artifact split
- Git repo (public): manifest + dotfiles + scripts. Nothing secret ever lives here.
- Encrypted vault (never in git): tokens, registry auth, cloud creds.
- Windows-host backup: age-encrypted SSH key archive for WSL2 distro survival.

### 3. Bootstrap: thin launcher, not the tool
`curl <repo>/bootstrap.sh` → detects OS, installs rustup + cargo if absent, builds the tool from source (`cargo install --git`), execs the first-run wizard. The script is ~100 lines and contains zero provisioning intelligence. Prebuilt binaries are the escape hatch if compile time ever bothers the user (NOT a default, avoiding the Gatekeeper/SmartScreen unsigned-download friction and a 6-target CI matrix). The Windows branch of the bootstrap mostly reads "install WSL2 if absent, then run the Unix path"; Git Bash is explicitly not steered toward.

### 4. Trust follow-main for the tool itself
Bootstrap pins nothing; `setmeup update` = git pull + rebuild (`apply` is declarative + idempotent, so pulling a newer binary is a safe re-run of the same manifest, not a surprise mutation). Optionally pin to tags later only if reproducibility becomes a real need.

### 5. Secret Backend as a trait; migration as a first-class verb
The ladder is literal architecture:
- **Step 0** (fresh headless WSL2): `age`-encrypted file at `~/.config/setmeup/secrets.age`, keyed by a passphrase entered in the TUI. No daemons, no D-Bus, no keyring dependency.
- **Step 1**: OS vault guards the encryption key — Secret Service (gnome-keyring/libsecret) on Linux, Keychain on macOS, Credential Manager on Windows. The data file stays encrypted at rest; the vault guards the key.
- **Step 2** (later, optional): external vault CLI (1Password/Bitwarden) as the vault of record; setmeup pulls from it.

`setmeup secrets migrate --to <backend>` moves the user up the ladder. Detection = "does the keyring exist and can we reach it?" (Rust: `age` crate for the file, `keyring` crate for cross-platform vault access.)

### 6. Trust boundary: physical host, not WSL distro
WSL2 is disposable by construction. Identity must survive `wsl --unregister` on the same laptop. SSH keys are age-encrypted (passphrase = vault passphrase, because Windows user accounts are not a security boundary against themselves) and stored at `\\wsl$\...\mnt/c/Users/<you>/.setmeup/backups/ssh-keys.age`.

First-run flow on a fresh distro:
1. `~/.ssh` has keys? → no-op, done.
2. Backup exists on `/mnt/c`? → decrypt → `~/.ssh` → chmod 600 → verify pubkey registered, re-register only if missing.
3. Else → generate ed25519 → register pubkey via token → age-encrypt backup to `/mnt/c`.

This is restore-before-generate: it closes the orphan-key hole that plain "no key → generate" would create on every distro wipe. "Idempotent" here means *heals*, not *same every time*.

### 7. Two identity classes, asymmetric by design
- **Recurring (SSH keypair):** survives distro wipes, backed up age-encrypted, bound to physical host.
- **Fresh (GPG commit-signing):** dies with the distro, regenerated per machine, fingerprint registered lazily. No long-lived signing key — provenance isn't worth it for this tool.

### 8. Token acquisition is per-secret policy
The manifest declares how each credential is acquired (`via: paste | device-flow | env`):
- `paste` for simple cases
- OAuth device-flow for tokens where the UX justifies it (CLI polls, browser authorizes)
- `env` for credentials the user already exports

One abstraction — `SecretAcquire` — dispatches to the right prompt type; the TUI renders whatever policy the manifest declares rather than being a fixed form.

### 9. CLI surface
`setmeup configure` (TUI wizard) · `apply` · `status` · `diff` · `update` · `secrets` (store/list/migrate) · `keys` (list/revoke). Machine-readable output where it aids scripting.

### 10. Idempotency guarantee for apply
- Package provisioning: backends check presence before install (apt `dpkg -s`, brew list, `winget list`).
- Dotfiles: symlink-managed, `ln -sf` semantics, no content churn.
- Identity: never generates a second key when one exists; never re-registers a known pubkey.

## Risks / Trade-offs

- **Windows-host encryption is only as strong as a passphrase in a memory of the host session** → backups are age-encrypted with the vault passphrase, never plaintext; threat model is "casual access to the Windows user profile", not determined attacker.
- **Secret Service absent on fresh WSL2 → accidental falloff to plaintext** → the design's invariant is: if no keyring is reachable, the *encrypted file* backend is the floor; there is no plaintext fallback.
- **Git Bash being janky → users reaching for it** → the tool only supports PowerShell or plain Unix paths; WSL2 is the blessed route.
- **fetch-main builds can pull a breaking change** → mitigated by declarative + idempotent apply; a bad update converges to the same state a moment later.
- **Orphan pubkeys from abandoned distros** → `setmeup keys list/revoke` addresses the cruft; acceptable for a personal tool.
- **Public repo + secret hygiene is a correctness boundary, not a convention** → enforced by architecture: secret material only ever written by the `secrets`/`identity` modules into non-git paths, never by the manifest/git path.

## Migration Plan

Greenfield — no existing code to migrate. Seeds the initial repo structure: Cargo workspace, manifest schema, backend trait scaffolding, and OpenSpec specs.

## Open Questions

- Whether `cargo install --git` vs prebuilt release matters in practice (measure first-run compile time before building a release pipeline).
- Specific tool/dotfile catalog the user wants as the default manifest (the TUI's initial wizard content) — for a later change.
- External vault backend (1Password/Bitwarden) shape — deferred, Step 2 of the secret ladder.

## TODO (deferred)

- **Agentic development scaffold** — AGENTS.md, project skills, and a repo layout optimized for agent-driven development of this project. **RESOLVED:** captured as the separate `dev-workflow` change (`openspec/changes/dev-workflow/`), which owns the operator contract, curated project skills, CI enforcement, and the reviewer-facing `docs/workflow.md` narrative.