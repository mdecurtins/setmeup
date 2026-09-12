---
name: manifest-packages
description: Document and implement the `packages` section of the setmeup manifest — OS-native tool installation via apt, brew, winget, pip, and verified download methods.
license: MIT
metadata:
  author: setmeup
  version: "1.0"
---

# Overview

The `packages` section declares OS-native packages to install. It is a map from package name to config. Each key is the canonical cross-OS identifier; the config object provides optional `from` (backend selector), `depends` (within-section ordering), and per-OS package-name overrides. The provisioning backend resolves the correct package manager command at apply time.

**Implementation status:** the `packages` handler is declared but `provision()` is not yet implemented — it surfaces a visible "not yet implemented" failure.

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
packages:                              # map name -> config
  <name>:                              # canonical cross-OS identifier (key)
    from: <backend>                    # optional: apt | brew | winget | pip | download
    depends: [<other-package-name>]    # optional, within-section ordering only
    apt: <apt-package-name>            # optional, overrides key on Ubuntu
    brew: <brew-formula-name>          # optional, overrides key on macOS
    winget: <winget-package-id>        # optional, overrides key on Windows
```

The map key (`<name>`) IS the package name. Per-OS overrides are only needed when the package name differs across platforms (e.g. `docker.io` on Ubuntu vs `docker` on macOS).

## Required vs optional fields

| Field      | Required | Description |
|------------|----------|-------------|
| `from`     |          | Backend selector: `apt`, `brew`, `winget`, `pip`, `download`. `None` = per-OS default (apt on Ubuntu, brew on macOS, winget on Windows). |
| `depends`  |          | List of other package **keys** that must install first (within-section ordering only). |
| `apt`      |          | Override package name on Ubuntu/Debian when it differs from the key. |
| `brew`     |          | Override formula name on macOS when it differs from the key. |
| `winget`   |          | Override package ID on Windows when it differs from the key (e.g. `Git.Git`). |

## Validation rules

1. The map key (package name) must be non-empty.
2. `from` when present must be one of: `apt`, `brew`, `winget`, `pip`, `download`.
3. `depends` must reference keys that also appear in the `packages` map (self-references rejected).
4. `depends` ordering constraint: there must be no cycles in the dependency graph. If A depends on B and B depends on C, the provisioning order is C, B, A.
5. `from: download` means the package is a verified download (git clone or checksum-pinned binary) — see Trust boundary section below. No `url` or `checksum` fields exist in the manifest schema; the download URL and verification method are encoded in the handler, not the manifest.

## Example YAML

```yaml
packages:
  git: {}                       # empty = per-OS default from
  yt-dlp:
    from: pip
    depends: [ffmpeg]
  docker:
    apt: docker.io
    brew: docker
  starship:
    from: download

---

# Provisioning

## Install command template

```rust
// OS-native backends
fn install_package(package_name: &str, from: Option<&str>) -> Result<ItemStatus> {
    let backend = from.unwrap_or_else(|| default_backend_for_os(os));
    match backend {
        "apt" => sudo apt-get install -y <package_name>,
        "brew" => brew install <package_name>,
        "winget" => winget install --id <package_name> ...,
        "pip" => pip3 install <package_name>,
        "download" => verified_download(package_name), // see trust boundary
    }
}

// download (verified, handler-encoded)
fn verified_download(package_name: &str) -> Result<ItemStatus> {
    let (url, checksum) = lookup_download_info(package_name)?; // encoded in handler
    let archive = download(url)?;        // fetch to temp
    verify_sha256(&archive, checksum)?;  // must match
    extract_to(archive, dest_dir)?;      // tar/gzipped or unzip
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
| (key)      | free text (required, canonical name) |
| `from`     | list toggle: `""` | `"apt"` | `"brew"` | `"winget"` | `"pip"` | `"download"` |
| `apt`      | free text (auto-filled from key) |
| `brew`     | free text (auto-filled from key) |
| `winget`   | free text (auto-filled from key) |
| `depends`  | multi-select from already-listed package keys |

## Default values

- `from` defaults to empty (per-OS default backend).
- `apt`, `brew`, `winget` default to the map key (package name) unless explicitly overridden.
- `depends` defaults to empty list.

## Validation rules (wizard-specific)

- The map key must be non-empty and not already used in the current session.
- `from` when set must be one of the recognized backends.
- `depends` selections cannot include the package itself.

---

# Trust boundary

**`from: download` means verified download only.** The manifest does not carry download URLs or checksums — those are encoded in the handler. The implementation MUST:

1. Fetch via secure transport (HTTPS, never HTTP).
2. Verify integrity (SHA-256 checksum or git tag verification) before extracting or executing.
3. Fail closed: if integrity check fails, remove the downloaded file and report `ItemStatus::Failed`.

No `from: download` entry may ever execute a `curl <url> | sh` pattern. Git clone for release tarballs is acceptable when the URL points to an official GitHub release asset. For git clone from arbitrary repos, see `manifest-repos` skill.
