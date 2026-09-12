---
name: manifest-repos
description: Document and implement the `repos` section of the setmeup manifest — declarative git repository clones under `~/repos/` for personal projects, dotfile repositories, and other working copies.
license: MIT
metadata:
  author: setmeup
  version: "1.0"
---

# Overview

The `repos` section declares git repositories to clone into `~/repos/<name>/`. It is a map from repo name to clone URL. This is a pure clone operation — no fetch, pull, or update happens on re-apply. The repos section is designed for personal project repos that the user wants available on every provisioned machine. Cloning is done via authenticated HTTPS (with credential helpers) or SSH (with configured keys).

**Implementation status:** the `repos` handler is declared but `provision()` is not yet implemented — it surfaces a visible "not yet implemented" failure.

---

# Research

## Git clone mechanics

- **Official docs:** https://git-scm.com/docs/git-clone
- **Default depth:** full clone (not shallow), to allow pushing and full history.
- **Clone URL formats:**
  - `https://github.com/user/repo.git` — HTTPS (recommended for setmeup, works with credential helpers)
  - `git@github.com:user/repo.git` — SSH (requires SSH key configured; see the `identity` spec)
  - `https://github.com/user/repo` — shorthand (GitHub auto-redirects; `.git` suffix is optional)

## Authentication methods

| Method | URL format | Prerequisites | setmeup recommendation |
|--------|-----------|---------------|------------------------|
| HTTPS + token | `https://<token>@github.com/user/repo.git` | Personal access token | Use with credential helper, never embed tokens in manifest |
| HTTPS + credential helper | `https://github.com/user/repo.git` | `gh auth setup-git` or manual helper | ✅ Recommended — works with `git-credential-libsecret` or GH CLI |
| SSH key | `git@github.com:user/repo.git` | SSH key in `~/.ssh/` and configured for host | ✅ Recommended — zero-auth after key setup |
| SSH + agent | `git@github.com:user/repo.git` | `ssh-agent` with loaded key | ✅ Best for interactive use |

## Platform availability

| Platform | Status | Notes |
|----------|--------|-------|
| Ubuntu   | ✅     | git required (see `packages`). SSH pre-installed. |
| macOS    | ✅     | git shipped with Xcode CLI tools. |
| Windows  | ✅ (WSL2) | git + SSH via WSL2 Linux environment. |
| Windows (native) | ⚠️ | Needs `git.exe` on PATH; use different root dir than `~/repos/`. |

---

# Manifest schema

## Block structure

```yaml
repos:                              # map name -> URL string
  <name>: "<git-clone-url>"        # value is the full clone URL
```

## Required vs optional fields

| Field    | Required | Description |
|----------|----------|-------------|
| (key)    | ✅       | Local directory name under `~/repos/`; used as the slug for references from aliases. |
| (value)  | ✅       | Remote git URL string. Accepts HTTPS and SSH formats. Must not contain embedded credentials. |

The map key is the repo name; the map value is the URL. There is no per-repo `path`, `branch`, or `depth` configuration — repos always clone to `~/repos/<name>/` at the remote's default branch, full depth.

## Validation rules

1. The map key (repo name) must be non-empty and unique (map keys are inherently unique).
2. The URL value must be non-empty and parse as a valid git remote URL (`https://...`, `git@...`, `ssh://...`).
3. The URL must not contain embedded credentials (reject `https://user:pass@...`). The `secrets` section handles credentials via acquisition policy.
4. Aliases may reference `repos.<name>` paths — those repos must be provisioned before the `aliases` section.

## Example YAML

```yaml
repos:
  dotfiles: "[EMAIL]:user/dotfiles.git"
  project-alpha: "https://github.com/user/project-alpha.git"
  hoodhunter: "github.com/user/hoodhunter"
```

---

# Provisioning

## Install command template

```bash
# Destination is always ~/repos/<name>
DEST="$HOME/repos/$name"

# Clone only if destination does not exist
if [ ! -d "$DEST/.git" ]; then
  git clone "$url" "$DEST"
fi
```

## Config file location and format

- **No additional config beyond the clone.** git stores remote references, config, and state inside `~/.gitconfig` (global) and `<repo>/.git/config` (local).
- **Destination layout:** every repo gets its own directory under `~/repos/<name>/`.
- **No setmeup-managed marker file.** Simply checking `.git` directory existence at the destination is the idempotency check. A `.git` directory that exists but points to a different remote is a divergent state (see below).

## Verification command

```bash
# Repo exists and is a git working copy
test -d "$DEST/.git"

# Verify remote URL matches (for divergence detection)
git -C "$DEST" remote get-url origin | grep -F "$declared_url"
```

## Dependencies on other capabilities

- **Required before repos provision:** `packages` must include `git`. setmeup enforces this even if the manifest does not declare git explicitly.
- **For SSH URLs:** the `identity` section must provide SSH key configuration before `repos` runs. setmeup checks for SSH URL patterns and reports a warning if no SSH key is configured.
- **Aliases may reference repo paths:** if an alias uses a path like `~/repos/my-project/scripts/deploy.sh`, the repo must be cloned first. The provisioning order enforces `repos` before `aliases`.
- **Ordering:** `packages` (git) → `identity` (SSH keys) → `repos` → `aliases` (path references).

---

# Idempotency

## How to check if already satisfied

1. **Repo cloned:** `~DEST/.git/HEAD` exists and is a regular file (detached HEAD or branch head).
2. **Remote URL matches:** `git -C "$DEST" remote get-url origin` output matches the declared URL (exact string comparison, with `.git` suffix normalization: both `https://github.com/user/repo.git` and `https://github.com/user/repo` are accepted for the same repo).

If BOTH are satisfied, the repo reports `Satisfied`. No `git fetch` or `git pull` is performed — setmeup is convergence for clone state, not a sync tool.

## What constitutes "divergent" state

- **URL mismatch:** `.git` exists but the remote URL differs from the declared URL. → Report `ItemStatus::Failed` with a clear message: "Destination already exists with a different remote URL. Remove manually or update manifest." setmeup does NOT overwrite or `git remote set-url` — this is a user-managed area.
- **Directory exists but is not a git repo:** `$DEST` exists as a regular file or empty directory. → Report `ItemStatus::Failed`: "Path exists but is not a git repository. Remove it manually to allow clone."

## What state is "user-managed" (no setmeup marker)

- Any existing repo at the destination that was NOT cloned by setmeup is left alone (setmeup does not write marker files for repos — `.git` directory presence is the check).
- Uncommitted changes, stashes, and dirty working trees are never touched.
- Additional branches, remotes, or tags the user has added are left untouched.
- Removing a repo from the manifest does NOT delete the directory.
- The `~/repos/` parent directory itself is not managed (user may have other repos there).

---

# Wizard

## Input type

| Field   | Type | Widget |
|---------|------|--------|
| (key)   | free text | Text input for directory name |
| (value) | free text | Text input for git URL; show examples: HTTPS, SSH |

## Default values

No defaults — the wizard starts with an empty list. The destination is always `~/repos/<name>`.

## Validation rules (wizard-specific)

- The map key must be non-empty, unique in the current session, and match `^[a-zA-Z0-9_-]+$` (safe directory name).
- The URL value must be non-empty and start with `https://`, `git@`, or `ssh://`.
- The URL must not contain `@` before the host (reject embedded credentials) — unless it's a valid SSH URL like `[EMAIL]:user/repo.git`.
