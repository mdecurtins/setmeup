---
name: manifest-rustup
description: Document and implement the `rustup` section of the setmeup manifest — Rust toolchain installation via rustup-init (verified download), default toolchain pinning, and target/component declarations.
license: MIT
metadata:
  author: setmeup
  version: "1.0"
---

# Overview

The `rustup` section installs the Rust toolchain manager via a verified `rustup-init` download and configures the default toolchain. This is distinct from `packages` because rustup is installed via an official installer binary (verified by checksum and GPG), not via the OS package manager. Package-manager versions of Rust (`apt install rustc`) are never used — they lag behind and conflict with rustup-managed installs.

**Implementation status:** the `rustup` handler has a real `check()` but `provision()` is a stub in this change. Only `toolchain` is declared in the schema; `profile`, `targets`, and `components` are NOT part of the implemented schema.

---

# Research

## Official docs URL

- **rustup.rs:** https://rustup.rs/
- **GitHub:** https://github.com/rust-lang/rustup
- **Rust toolchain channels:** https://rust-lang.github.io/rustup/concepts/channels.html
- **Rust release schedule:** https://forge.rust-lang.org/infra/release-process.html (6-week cadence)
- **Component availability:** https://rust-lang.github.io/rustup-components-history/

## Latest version / release source

- **rustup:** Check https://github.com/rust-lang/rustup/releases for the latest stable release.
- **Toolchains:** stable (default), beta, nightly — published every 6 weeks at https://forge.rust-lang.org/.
- **Rustup SHA-256 / GPG sig:** Official releases are signed; verify via `gpg --verify` with the Rust release signing key.

## Installation methods and prerequisites

- **Required method (setmeup):** Download `rustup-init` for the target architecture + OS, verify checksum, run it with `--no-modify-path` and `-y` to avoid interactive prompts. This is, by design, a binary download — **not** a `curl | sh` pipe.
- **Prerequisites:** `curl` (or `wget`), standard POSIX utils; C compiler toolchain (`build-essential` on Ubuntu, `xcode-select` on macOS) may be needed for local compilation but is not strictly required for rustup itself.
- **Alternative (not used):** `apt install rustc` — packages are old, don't interact with rustup, and fragment the ecosystem.
- **Windows:** rustup-init.exe is available; on WSL2 use the Linux binary.

## The verified download flow

The "curl | sh" pattern on the rustup.rs landing page is the most common installation method but it violates the setmeup trust boundary (pipes network content into a shell). Instead, setmeup implements the equivalent as a **verified binary download**:

1. Download `rustup-init` for the target triple.
2. Verify SHA-256 checksum (fetch checksums from the official source).
3. Run the verified binary with `-y --no-modify-path` (the user does NOT want rustup modifying their .bashrc — setmeup manages shell integration separately).
4. Source `$HOME/.cargo/env` in the managed shell files (see `manifest-shell` skill).

## Platform availability

| Platform | Status | Target triple example |
|----------|--------|----------------------|
| Ubuntu x86_64 | ✅ | `x86_64-unknown-linux-gnu` |
| macOS (Intel) | ✅ | `x86_64-apple-darwin` |
| macOS (Apple Silicon) | ✅ | `aarch64-apple-darwin` |
| Windows (WSL2) | ✅ | Use Linux binary |
| Windows (native) | ✅ | `x86_64-pc-windows-msvc` via `rustup-init.exe` |

---

# Manifest schema

## Block structure

```yaml
rustup:
  toolchain: stable         # optional, string, default "stable"
```

All fields are optional. An absent `rustup` section means "do not manage rustup"; a present `rustup: {}` means "install rustup with defaults" (stable toolchain).

## Required vs optional fields

| Field       | Required | Description |
|-------------|----------|-------------|
| `toolchain` |          | Rust channel or version string. Accepts: `stable`, `nightly`, `beta`, `1.82.0`, `nightly-2024-01-01`. Default: `stable`. |

## Validation rules

1. `toolchain` must be non-empty when present. It is passed through to `rustup` — no static channel validation is done; invalid channels are caught at provisioning time by rustup itself.

> Note: `profile`, `targets`, and `components` are NOT part of the implemented schema. Do not add them to the Rust struct without an OpenSpec change; the current `RustupConfig` has only `toolchain`.

## Example YAML

```yaml
rustup:
  toolchain: stable
```

---

# Provisioning

## Install command template

```bash
# 1. Detect target triple
TRIPLE="x86_64-unknown-linux-gnu"  # adjust per platform

# 2. Download rustup-init with checksum verification
RUSTUP_URL="https://static.rust-lang.org/rustup/dist/${TRIPLE}/rustup-init"
CHECKSUM_URL="${RUSTUP_URL}.sha256"
curl -sSLO "$RUSTUP_URL"
curl -sSLO "$CHECKSUM_URL"
sha256sum -c "$(basename ${RUSTUP_URL}).sha256"  # fails if checksum mismatch

# 3. Run the verified binary
chmod +x rustup-init
./rustup-init -y --no-modify-path --default-toolchain <toolchain>

# 4. Set default toolchain (redundant if --default-toolchain worked, but idempotent)
rustup default <toolchain>

# 5. Clean up
rm rustup-init rustup-init.sha256
```

The current `provision()` is a stub — this template is the target behavior.

## Config file location and format

- **rustup home:** `~/.rustup/` — managed entirely by rustup.
- **Cargo home:** `~/.cargo/` — contains `bin/` with `rustc`, `cargo`, etc.
- **Cargo env:** `~/.cargo/env` — shell-sourced file with PATH additions. setmeup handles sourcing this in the `shell` section (see `manifest-shell` skill).
- **Config file (future):** `~/.cargo/config.toml` — not managed by setmeup currently.

## Verification command

```bash
# rustup binary present
rustup --version

# toolchain installed and default set
rustup show | grep <toolchain>

# cargo on PATH (via source)
source ~/.cargo/env && cargo --version
```

## Dependencies on other capabilities

- **Required before rustup provisions:** `packages` must include `curl` (for the verified download).
- **Required after rustup provisions:** `shell` section must include the rustup/cargo PATH lines (see `manifest-shell` skill) so `rustc`, `cargo`, etc. are available in interactive shells.
- **Ordering:** `packages` (curl) → `rustup` → `shell` (cargo PATH).

---

# Idempotency

## How to check if already satisfied

1. **rustup installed:** `rustup --version` exits with code 0.
2. **Toolchain matches:** `rustup default` output matches the declared toolchain channel/version.

If BOTH are satisfied, the `rustup` block reports `Satisfied` and no operations run.

## What constitutes "divergent" state

- **rustup missing:** `rustup` command not found. → Run the verified download and binary install.
- **Toolchain mismatch:** `rustup default` shows a different toolchain. → Run `rustup default <declared>` to switch (does NOT remove the old toolchain).
- **Toolchain not installed:** the declared toolchain is not in `rustup show`. → Run `rustup toolchain install <declared>` then `rustup default <declared>`.
- **Download checksum mismatch:** the rustup-init binary fails the SHA-256 check. → Remove downloaded file, report `ItemStatus::Failed`, do NOT run the binary.

## What state is "user-managed" (no setmeup marker)

- Additional toolchains installed by the user (beyond the declared default) are left untouched.
- Non-default overrides (`rustup override set`) in specific directories are left untouched.
- `~/.cargo/config.toml` is not managed.
- `~/.rustup/settings.toml` is not managed.

---

# Wizard

## Input type

| Field        | Type | Widget |
|--------------|------|--------|
| `toolchain`  | list toggle | Options: `stable`, `beta`, `nightly`; option to type a specific version |

## Default values

- `toolchain`: `stable`

## Validation rules (wizard-specific)

- `toolchain` must match one of: `^stable$`, `^beta$`, `^nightly$`, `^\d+\.\d+\.\d+$`, or `^nightly-\d{4}-\d{2}-\d{2}$`.
