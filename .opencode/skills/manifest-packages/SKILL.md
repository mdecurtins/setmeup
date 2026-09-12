---
name: manifest-packages
description: Document and implement the `packages` section of the setmeup manifest — OS-native tool installation via apt, brew, winget, pip, and verified download methods.
license: MIT
metadata:
  author: setmeup
  version: "1.0"
---

# Overview

The `packages` section declares OS-native packages to install. Each entry specifies a canonical name and optional per-OS overrides. The provisioning backend resolves the correct package manager command at apply time.

---

# Research

## Official docs and package sources

| Backend | Registry URL | Query command | Install command |
|---------|-------------|---------------|-----------------|
| apt (Debian/Ubuntu) | https://packages.ubuntu.com/ | `apt-cache search <name>` | `sudo apt-get install -y <name>` |
| brew (macOS) | https://formulae.brew.sh/ | `brew info <name>` | `brew install <name>` |
| winget (Windows) | https://winstall.app/ or `winget search <name>` | `winget search --id <name>` | `winget install --id <name>` |
| pip (Python, cross-platform) | https://pypi.org/ | `pip show <name>` | `pip install <name>` |

## Latest version / release sources

- **apt:** snapshots at https://packages.ubuntu.com/; version depends on distro release channel.
- **brew:** latest stable from formula; upstream tracks the tool's own release tags.
- **winget:** community-curated manifests at https://github.com/microsoft/winget-pkgs.
- **pip:** latest release on PyPI.

## Installation methods and prerequisites

- **apt:** requires `sudo` access; available on all Debian-family distros.
- **brew:** must be installed first (https://brew.sh/); pre-installed on macOS in many org setups.
- **winget:** ships with Windows 10 1809+ / Windows 11; app installer optional on older builds.
- **pip:** `python3 -m pip` bundled with modern Python; standalone on some distros.

## Platform availability

| Backend | Ubuntu | macOS | Windows |
|---------|--------|-------|---------|
| apt     | ✅ primary | ❌  | ❌  |
| brew    | ❌ (usable but not default) | ✅ primary | ❌  |
| winget  | ❌  | ❌  | ✅ primary |
| pip     | ✅ optional | ✅ optional | ✅ optional |
| download| ✅ manual | ✅ manual | ✅ manual |

---

# Manifest schema

## Block structure

```yaml
packages:
  - name: <canonical-name>           # required, unique
    apt: <apt-package-name>          # optional, overrides name on Ubuntu
    brew: <brew-formula-name>        # optional, overrides name on macOS
    winget: <winget-package-id>      # optional, overrides name on Windows
    pip: <pypi-package-name>         # optional, installs via pip3
    from: <source-type>              # optional, "download" for verified download
    url: <download-url>              # required if from: download
    checksum: <sha256-hex>           # required if from: download
    depends: [<other-package-name>]  # optional, ordering constraint within section
```

## Required vs optional fields

| Field      | Required | Description |
|------------|----------|-------------|
| `name`     | ✅       | Canonical cross-OS identifier; used as fallback package name |
| `apt`      |          | Override package name on Ubuntu/Debian |
| `brew`     |          | Override formula name on macOS |
| `winget`   |          | Override package ID on Windows (e.g. `Git.Git`) |
| `pip`      |          | PyPI package name; adds pip as implicit dependency |
| `from`     |          | Source type: currently only `download` is defined |
| `url`      | ⚠️      | Required when `from: download` — direct download URL |
| `checksum` | ⚠️      | Required when `from: download` — SHA-256 hex digest |
| `depends`  |          | List of other package names that must install first |

## Validation rules

1. `name` must be non-empty and unique across the `packages` list.
2. At least one backend must be resolvable: either a per-OS override OR `name` is a valid package name for at least one OS.
3. `from: download` requires both `url` and `checksum`; rejects any per-OS package overrides (mutually exclusive with apt/brew/winget/pip).
4. `depends` must reference names that also appear in `packages` (self-references rejected).
5. `checksum` must be a valid 64-character hex string (`^[0-9a-f]{64}$`).
6. `depends` ordering constraint: there must be no cycles in the dependency graph. If A depends on B and B depends on C, the provisioning order is C, B, A.

## Example YAML

```yaml
packages:
  - name: git
    apt: git
    brew: git
    winget: Git.Git

  - name: neovim
    apt: neovim
    brew: neovim

  - name: docker
    apt: docker.io
    brew: docker

  - name: gh
    apt: gh
    brew: gh
    winget: GitHub.cli

  - name: ruff
    pip: ruff

  - name: starship
    from: download
    url: https://github.com/starship/starship/releases/latest/download/starship-x86_64-unknown-linux-gnu.tar.gz
    checksum: a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4a5b6c7d8e9f0a1b
    depends: [git, curl]
```

---

# Provisioning

## Install command template

```rust
// OS-native backends
fn install_package(os: Os, package_name: &str) -> Result<ItemStatus> {
    match os {
        Os::Ubuntu => sudo apt-get install -y <package_name>,
        Os::Macos  => brew install <package_name>,
        Os::Windows => winget install --id <package_name> --accept-package-agreements --accept-source-agreements,
    }
}

// pip (cross-platform)
pip3 install <pypi-package-name>

// download (verified, cross-platform)
fn install_download(url: &str, checksum: &str, dest_dir: &Path) -> Result<ItemStatus> {
    let archive = download(url)?;       // fetch to temp
    verify_sha256(&archive, checksum)?; // must match
    extract_to(archive, dest_dir)?;     // tar/gzipped or unzip
    Ok(ItemStatus::Satisfied)
}
```

## Config file location and format

- No config files per package; package state is managed by the OS package manager.
- `from: download` artifacts live under `~/.local/bin/` (symlinked) or `~/.local/share/<name>/`.
- Download cache may live in `~/.cache/setmeup/downloads/` for re-use on re-apply.

## Verification command

```bash
# apt: dpkg presence check
dpkg -s <package>          # exit 0 = installed

# brew
brew list <formula>        # exit 0 = installed

# winget
winget list --id <package> # exit 0 = installed

# pip
pip3 show <package>        # exit 0 = installed

# download
test -f <dest-path>/<binary>  # binary exists in expected location
```

## Dependencies on other capabilities

- `from: download` depends on `curl` or `wget` (implicit: adds it if not declared).
- `pip` depends on `python3` and `pip3` being available on the system.
- No hard ordering across sections; packages install before `aliases` and `shell` so aliases can reference installed tools.

---

# Idempotency

## How to check if already satisfied

Presence-check per backend (see Verification command above) — each tool is checked before install. Presence means "the package manager reports it as installed," **not** "the binary exists on PATH". This avoids false-positives from stale symlinks or manually-placed binaries.

## What constitutes "divergent" state

- A tool is installed via a different method than declared (e.g., manually compiled vs. apt). This is **not** reported as an error — setmeup treats the package as "present" and skips it.
- `from: download`: divergent = the binary at the expected path has a different SHA-256 than the declared checksum. This triggers a re-download and re-extract.
- A package installed at the wrong version is **not** detected unless a version field is added in the future (v2). Currently presence-check only.

## What state is "user-managed" (no setmeup marker)

- Any tool installed outside the manifest (user ran `brew install xyz` independently) is not tracked by setmeup. No marker file is written.
- `from: download` writes a marker file at `~/.local/share/setmeup/downloads/<name>.installed.sha256` to distinguish setmeup-managed downloads from user-placed ones.
- Removing a tool from the manifest does NOT uninstall it — setmeup is convergence-only, not garbage-collection.

---

# Wizard

## Input type

| Field      | Type |
|------------|------|
| `name`     | free text (required) |
| `apt`      | free text (auto-filled from `name`) |
| `brew`     | free text (auto-filled from `name`) |
| `winget`   | free text (auto-filled from `name`) |
| `pip`      | free text |
| `from`     | list toggle: `""` | `"download"` |
| `url`      | free text (shown only if `from: download`) |
| `checksum` | free text (shown only if `from: download`) |
| `depends`  | multi-select from already-listed package names |

## Default values

- `apt`, `brew`, `winget` default to `name` unless explicitly overridden.
- `from` defaults to empty (OS-native package manager).
- `depends` defaults to empty list.

## Validation rules (wizard-specific)

- `name` must be non-empty and not already used in the current session.
- URL must parse as a valid HTTPS URL when `from: download`.
- Checksum must be exactly 64 hex characters when `from: download`.
- `depends` selections cannot include the package itself.

---

# Trust boundary

**`from: download` means verified download only.** The implementation MUST:

1. Fetch the URL via a secure transport (HTTPS, never HTTP).
2. Verify the SHA-256 checksum before extracting or executing.
3. Fail closed: if checksum does not match, remove the downloaded file and report `ItemStatus::Failed`.

No `from: download` entry may ever execute a `curl <url> | sh` pattern. Git clone for release tarballs is acceptable when the URL points to an official GitHub release asset. For git clone from arbitrary repos, see `manifest-repos` skill.
