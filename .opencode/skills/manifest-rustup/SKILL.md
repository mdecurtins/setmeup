---
name: manifest-rustup
description: Document and implement the `rustup` section of the setmeup manifest — Rust toolchain installation via rustup-init (verified download), default toolchain pinning, and target/component declarations.
license: MIT
metadata:
  author: setmeup
  version: "1.0"
---

# Overview

The `rustup` section installs the Rust toolchain manager via a verified `rustup-init` download and configures the default toolchain, targets, and components. This is distinct from `packages` because rustup is installed via an official installer binary (verified by checksum and GPG), not via the OS package manager. Package-manager versions of Rust (`apt install rustc`) are never used — they lag behind and conflict with rustup-managed installs.

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
  toolchain: <channel>        # required, e.g. "stable", "nightly", "1.78.0"
  profile: <profile>          # optional, minimal / default / complete
  targets:                    # optional list of cross-compilation targets
    - <target-triple>
  components:                 # optional list of additional components
    - <component-name>
```

## Required vs optional fields

| Field       | Required | Description |
|-------------|----------|-------------|
| `toolchain` | ✅       | Rust channel or version string. Accepts: `stable`, `nightly`, `beta`, `1.82.0`, `nightly-2024-01-01`. |
| `profile`   |          | Component selection profile: `minimal` (only rustc/cargo/std), `default` (adds rustdoc, clippy, etc.), `complete` (all components). Defaults to `default`. |
| `targets`   |          | Additional cross-compilation targets to install (e.g. `wasm32-unknown-unknown`, `aarch64-unknown-linux-gnu`). |
| `components`|          | Additional components not covered by the profile (e.g. `rust-analyzer`, `rust-src`, `miri`, `llvm-tools`). |

## Validation rules

1. `toolchain` must be non-empty and one of: a channel keyword (`stable`, `beta`, `nightly`), an exact version (`1.82.0`), or a dated nightly (`nightly-YYYY-MM-DD`).
2. `profile` must be one of: `minimal`, `default`, `complete`.
3. `targets` entries must be valid Rust target triples (e.g. `x86_64-unknown-linux-gnu`, `wasm32-unknown-unknown`). No validation beyond non-empty is done statically; invalid targets are caught by `rustup target add`.
4. `components` entries must be valid component names recognized by rustup (e.g. `clippy`, `rustfmt`). Invalid components are caught at provisioning time.
5. The special component `rust-analyzer` has a known name mismatch: the target directory is `rust-analyzer` but the component name is `rust-analyzer-preview` on some older toolchains. Check the installed toolchain's component list if `rust-analyzer` fails.

## Example YAML

```yaml
rustup:
  toolchain: stable
  profile: default
  targets:
    - wasm32-unknown-unknown
    - aarch64-unknown-linux-gnu
  components:
    - rust-analyzer
    - rust-src
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
./rustup-init -y --no-modify-path \
  --default-toolchain <toolchain> \
  --profile <profile>

# 4. Add targets
rustup target add <target1> <target2>

# 5. Add components
rustup component add <component1> <component2>

# 6. Set default toolchain (redundant if --default-toolchain worked, but idempotent)
rustup default <toolchain>

# 7. Clean up
rm rustup-init rustup-init.sha256
```

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

# targets installed
rustup target list --installed | grep <target>

# components installed
rustup component list --installed | grep <component>

# cargo on PATH (via source)
source ~/.cargo/env && cargo --version
```

## Dependencies on other capabilities

- **Required before rustup provisions:** `packages` must include `curl` (for the verified download).
- **Required after rustup provisions:** `shell` section must include `. ~/.cargo/env` sourcing so `rustc`, `cargo`, etc. are available in interactive shells.
- **Ordering:** `packages` (curl) → `rustup` → `shell` (cargo env sourcing).

---

# Idempotency

## How to check if already satisfied

1. **rustup installed:** `rustup --version` exits with code 0.
2. **Toolchain matches:** `rustup default` output matches the declared toolchain channel/version.
3. **Profile matches:** (indirect) if the toolchain is the same profile and the expected components are present, the profile is considered satisfied. Direct profile detection requires `rustup show` parsing.
4. **Targets present:** `rustup target list --installed` contains each declared target.
5. **Components present:** `rustup component list --installed` contains each declared component.

If ALL five are satisfied, the entire `rustup` block reports `Satisfied` and no operations run.

## What constitutes "divergent" state

- **rustup missing:** `rustup` command not found. → Run the verified download and binary install.
- **Toolchain mismatch:** `rustup default` shows a different toolchain. → Run `rustup default <declared>` to switch (does NOT remove the old toolchain).
- **Toolchain not installed:** the declared toolchain is not in `rustup show`. → Run `rustup toolchain install <declared>` then `rustup default <declared>`.
- **Missing targets:** at least one declared target is not installed. → Run `rustup target add <target>` for each missing target.
- **Missing components:** at least one declared component not installed for the default toolchain. → Run `rustup component add <component>` for each missing component.
- **Download checksum mismatch:** the rustup-init binary fails the SHA-256 check. → Remove downloaded file, report `ItemStatus::Failed`, do NOT run the binary.

## What state is "user-managed" (no setmeup marker)

- Additional toolchains installed by the user (beyond the declared default) are left untouched.
- Additional targets and components installed by the user are left untouched.
- Non-default overrides (`rustup override set`) in specific directories are left untouched.
- `~/.cargo/config.toml` is not managed.
- `~/.rustup/settings.toml` is not managed.

---

# Wizard

## Input type

| Field        | Type | Widget |
|--------------|------|--------|
| `toolchain`  | list toggle | Options: `stable`, `beta`, `nightly`; option to type a specific version |
| `profile`    | list toggle | Options: `minimal`, `default`, `complete` |
| `targets`    | multi free text | Add known targets via toggle list + free text for custom triples |
| `components` | multi-select | Checkboxes: `rust-analyzer`, `rust-src`, `clippy`, `rustfmt`, `miri`, `llvm-tools`, `rust-docs`; option to type custom |

## Default values

- `toolchain`: `stable`
- `profile`: `default`
- `targets`: empty
- `components`: empty

## Validation rules (wizard-specific)

- `toolchain` must match one of: `^stable$`, `^beta$`, `^nightly$`, `^\d+\.\d+\.\d+$`, or `^nightly-\d{4}-\d{2}-\d{2}$`.
- `profile` must be one of the three recognized values.
- `targets` entries must look like valid triples (at least two hyphens: `arch-vendor-os`).
- Duplicate targets or components are filtered.
