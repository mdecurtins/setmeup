---
name: manifest-repos
description: Document and implement the `repos` section of the setmeup manifest — declarative git repository clones under `~/repos/` for personal projects, dotfile repositories, and other working copies.
license: MIT
metadata:
  author: setmeup
  version: "1.0"
---

# Overview

The `repos` section declares git repositories to clone into `~/repos/<name>/`. This is a pure clone operation — no fetch, pull, or update happens on re-apply. The repos section is designed for personal project repos that the user wants available on every provisioned machine. Cloning is done via authenticated HTTPS (with credential helpers) or SSH (with configured keys).

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
repos:
  - name: <local-dir-name>      # required, unique
    url: <git-clone-url>         # required
    path: <custom-path>          # optional, overrides ~/repos/<name>
    branch: <branch-name>        # optional, default: default branch or current HEAD
    depth: <integer>             # optional, shallow clone depth (use sparingly)
```

## Required vs optional fields

| Field    | Required | Description |
|----------|----------|-------------|
| `name`   | ✅       | Local directory name under the repos root; used as the slug for `depends` references from aliases. |
| `url`    | ✅       | Remote git URL. Accepts HTTPS and SSH formats. Must not contain embedded credentials. |
| `path`   |          | Full path override (e.g., `~/src/my-project`). When absent, defaults to `~/repos/<name>`. |
| `branch` |          | Branch to check out after clone. When absent, uses the remote's default branch (`HEAD`). |
| `depth`  |          | Shallow clone depth (e.g., `1`). Use only for very large repos where full history is unnecessary. Default: no depth limit. |

## Validation rules

1. `name` must be non-empty and unique across the `repos` list.
2. `url` must be non-empty and parse as a valid git remote URL (`https://...`, `git@...`, `ssh://...`).
3. `url` must not contain embedded credentials (reject `https://user:pass@...`). The `SecretAcquire` policy is the correct path for credentials.
4. `path` must be an absolute path when provided (starting with `/` or `~`).
5. `depth` must be a positive integer when provided.
6. `branch` must be non-empty when provided.
7. Self-referencing `depends` in the broader manifest context: aliases may reference `repos.<name>` paths — those repos must be provisioned before the `aliases` section.

## Example YAML

```yaml
repos:
  - name: dotfiles
    url: git@github.com:user/dotfiles.git

  - name: project-alpha
    url: https://github.com/user/project-alpha.git
    branch: main

  - name: big-monorepo
    url: https://github.com/user/big-monorepo.git
    depth: 1
```

---

# Provisioning

## Install command template

```bash
# Default path resolution
DEST="${path:-$HOME/repos/$name}"

# Clone only if destination does not exist
if [ ! -d "$DEST/.git" ]; then
  git clone "$url" "$DEST"
fi

# Checkout specific branch if declared (safe after clone or if already cloned)
if [ -n "$branch" ]; then
  git -C "$DEST" checkout "$branch" 2>/dev/null || true
fi
```

## Config file location and format

- **No additional config beyond the clone.** git stores remote references, config, and state inside `~/.gitconfig` (global) and `<repo>/.git/config` (local).
- **Destination layout:** every repo gets its own directory under `~/repos/` (by default) or at the custom `path`.
- **No setmeup-managed marker file.** Simply checking `.git` directory existence at the destination is the idempotency check. A `.git` directory that exists but points to a different remote is a divergent state (see below).

## Verification command

```bash
# Repo exists and is a git working copy
test -d "$DEST/.git"

# Optional: verify remote URL matches (for divergence detection)
git -C "$DEST" remote get-url origin | grep -F "$declared_url"

# Optional: verify branch (if declared)
git -C "$DEST" rev-parse --abbrev-ref HEAD | grep -x "$branch"
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
- **Branch mismatch:** `.git` exists, URL matches, but current branch differs from the declared `branch` field. → Run `git checkout <declared-branch>` (non-destructive — fails if the branch does not exist locally).

## What state is "user-managed" (no setmeup marker)

- Any existing repo at the destination that was NOT cloned by setmeup is left alone (setmeup does not write marker files for repos — `.git` directory presence is the check).
- Uncommitted changes, stashes, and dirty working trees are never touched.
- Additional branches, remotes, or tags the user has added are left untouched.
- Removing a repo from the manifest does NOT delete the directory.
- The `~/repos/` parent directory itself is not managed (user may have other repos there).

---

# Wizard

## Input type

| Field    | Type | Widget |
|----------|------|--------|
| `name`   | free text | Text input for directory name |
| `url`    | free text | Text input for git URL; show examples: HTTPS, SSH |
| `path`   | free text | Optional; auto-filled from `~/repos/<name>`; user can override |
| `branch` | free text | Optional; leave empty for default branch |
| `depth`  | number | Optional; spinner or text input; leave empty for full clone |

## Default values

- `path`: `~/repos/<name>` (computed, not stored explicitly in the manifest unless overridden).
- `branch`: empty (default branch).
- `depth`: unset (full clone).

## Validation rules (wizard-specific)

- `name` must be non-empty, unique in the current session, and match `^[a-zA-Z0-9_-]+$` (safe directory name).
- `url` must be non-empty and start with `https://`, `git@`, or `ssh://`.
- `url` must not contain `@` before the host (reject embedded credentials) — unless it's a valid SSH URL like `git@github.com:user/repo.git`.
- `path` when provided must start with `/` or `~` (absolute path).
- `depth` must be a positive integer when provided.
