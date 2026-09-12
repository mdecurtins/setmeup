---
name: manifest-nvm
description: Document and implement the `nvm` section of the setmeup manifest — git-clone-based Node Version Manager install, Node.js version pinning, and global npm package declarations.
license: MIT
metadata:
  author: setmeup
  version: "1.0"
---

# Overview

The `nvm` section installs the Node Version Manager via a verified git clone and configures a default Node.js version and global npm packages. This is distinct from `packages` because nvm is installed by cloning a git repository (never via package manager), and its state is managed through `~/.nvm/` rather than `dpkg`/`brew`.

**Implementation status:** the `nvm` handler is declared but `provision()` is not yet implemented — it surfaces a visible "not yet implemented" failure.

---

# Research

## Official docs URL

- **nvm GitHub:** https://github.com/nvm-sh/nvm
- **nvm install guide:** https://github.com/nvm-sh/nvm#installing-and-updating
- **Node.js releases:** https://nodejs.org/en/about/previous-releases
- **Node.js LTS schedule:** https://github.com/nodejs/release#release-schedule

## Latest version / release source

- nvm uses tags: `v0.40.1` (latest as of writing). Check https://github.com/nvm-sh/nvm/tags for the current latest.
- Changelog: https://github.com/nvm-sh/nvm/releases

## Installation methods and prerequisites

- **Required method (setmeup):** git clone into `~/.nvm/` — **never** `curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/.../install.sh | bash` (curl-to-shell violates the trust boundary).
- **Prerequisites:** `git`, `curl`, `bash`, standard POSIX utilities.
- **Alternative (not used):** Homebrew installs nvm but creates a different directory layout (`$(brew --prefix nvm)`) with inconsistent sourcing behavior. setmeup uses the canonical git-clone layout for reliable sourcing.
- The official install.sh scripts do a git clone internally (checking out the tag), then add sourcing lines to bashrc/zshrc. setmeup replicates this with explicit control.

## Platform availability

| Platform | Status | Notes |
|----------|--------|-------|
| Ubuntu   | ✅     | Requires bash 4+ |
| macOS    | ✅     | Tested with bash (default zsh also works) |
| Windows  | ❌     | nvm Windows is a separate project; use via WSL2 |

---

# Manifest schema

## Block structure

```yaml
nvm:
  node-version: lts/*         # optional, string, default "lts/*"
  global-packages: [pnpm]     # optional, list of strings, default []
```

All fields are optional. An absent `nvm` section means "do not manage nvm"; a present `nvm: {}` means "install nvm with defaults" (latest tag, lts/* node, no global packages).

## Required vs optional fields

| Field              | Required | Description |
|--------------------|----------|-------------|
| `node-version`     |          | Node.js version to install and set as default. Accepts: `lts/*`, `lts/iron`, `22`, `20`, `18`, etc. Default: `lts/*`. |
| `global-packages`  |          | List of global npm packages to install after Node is set. Each entry is a package name, optionally with `@<version>`. Default: `[]`. |

## Validation rules

1. `node-version` must be non-empty when present (default `lts/*` is always valid).
2. `node-version` must match a valid Node.js version specifier that `nvm install` accepts.
3. `global-packages` entries must be non-empty.

## Example YAML

```yaml
nvm:
  node-version: "22"
  global-packages:
    - prettier
    - typescript@5.6
```

---

# Provisioning

## Install command template

```bash
# 1. Clone nvm repo at the declared version/tag
git clone --branch <version> https://github.com/nvm-sh/nvm.git ~/.nvm

# 2. Source nvm for the current shell session
export NVM_DIR="$HOME/.nvm"
[ -s "$NVM_DIR/nvm.sh" ] && \. "$NVM_DIR/nvm.sh"

# 3. Install the declared Node.js version
nvm install <node-version>

# 4. Set as default (writes to ~/.nvm/alias/default)
nvm alias default <node-version>

# 5. Install global npm packages
nvm use default
npm install -g <package1> <package2> ...
```

## Config file location and format

- **nvm source:** `~/.nvm/nvm.sh` (clone root)
- **nvm data dir:** `~/.nvm/` managed by nvm itself
- **Default Node alias:** `~/.nvm/alias/default` — single line containing the version string
- **Sourcing:** setmeup must ensure `~/.nvm/nvm.sh` is sourced in shell rc files. This is handled by the `shell` section (see `manifest-shell` skill), which adds:
  ```bash
  export NVM_DIR="$HOME/.nvm"
  [ -s "$NVM_DIR/nvm.sh" ] && \. "$NVM_DIR/nvm.sh"
  ```
  The nvm sourcing block must be idempotent (guard against double-source).

## Verification command

```bash
# nvm installed
test -f ~/.nvm/nvm.sh && echo "nvm script present"

# nvm is loadable
bash -c 'source ~/.nvm/nvm.sh && nvm --version'

# Node version matches declared
bash -c 'source ~/.nvm/nvm.sh && nvm ls default | grep <node-version>'

# Global packages installed
bash -c 'source ~/.nvm/nvm.sh && npm list -g --depth=0'
```

## Dependencies on other capabilities

- **Required before nvm provisions:** `packages` must include `git` (to clone the nvm repo).
- **Required after nvm provisions:** `shell` section must include the nvm sourcing lines so nvm is available in interactive shells.
- **Ordering:** `packages` → `nvm` → `shell` (nvm sourcing lines depend on `~/.nvm/nvm.sh` existing).

---

# Idempotency

## How to check if already satisfied

1. **nvm installed:** `~/.nvm/nvm.sh` exists and is a regular file (not a broken symlink).
2. **nvm version matches:** `source ~/.nvm/nvm.sh && nvm --version` output matches the declared `version` tag (trimming leading `v`).
3. **Node version matches:** `source ~/.nvm/nvm.sh && nvm ls default` reports the declared Node version as default and as installed.
4. **Global packages present:** `source ~/.nvm/nvm.sh && npm list -g --depth=0` includes each declared package at the declared version.

If ALL four are satisfied, the entire `nvm` block reports `Satisfied` and no operations run.

## What constitutes "divergent" state

- **nvm version mismatch:** `~/.nvm/nvm.sh` exists but `nvm --version` does not match the declared tag. → Re-clone at the declared tag.
- **Node version mismatch:** declared version is not in `nvm ls` or not set as default. → Run `nvm install <ver>` then `nvm alias default <ver>`.
- **Package missing:** a declared global package is absent. → `npm install -g <pkg>` for missing packages only.
- **Git clone target exists but is not a git repo:** `~/.nvm/` exists as a regular file or empty directory. → Remove and re-clone.

## What state is "user-managed" (no setmeup marker)

- Additional Node.js versions installed by the user (beyond the declared default) are left untouched.
- Additional global npm packages installed by the user are left untouched.
- Custom nvm aliases (`nvm alias <name> <ver>`) are left untouched.
- The ~/.nvm/ directory itself is entirely managed by setmeup for the declared version; user modifications to `~/.nvm/nvm.sh` or the checked-out git ref will be reverted on apply.

---

# Wizard

## Input type

| Field              | Type | Widget |
|--------------------|------|--------|
| `node-version`     | list toggle | Pre-populated: `lts/*`, `22`, `20`, `18`, `16`; option to type custom |
| `global-packages`  | multi free text | Add/remove list of npm package names |

## Default values

- `node-version`: `lts/*` (current LTS).
- `global-packages`: empty.

## Validation rules (wizard-specific)

- `node-version` must be a valid Node.js version: `lts/*`, `lts/<name>`, or a semver-like string (e.g. `22`, `22.11.0`).
- `global-packages` entries must be non-empty and cannot duplicate.
