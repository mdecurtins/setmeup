---
name: manifest-configs
description: Document and implement the `configs` section of the setmeup manifest — declarative tool configuration files with the implemented merge strategies (create-only, overwrite-managed, merge-declared).
license: MIT
metadata:
  author: setmeup
  version: "1.0"
---

# Overview

The `configs` section manages tool configuration files. It supports strategy-based application of declared content to a target path: `create-only` (never touch an existing file), `overwrite-managed` (regenerate files bearing a setmeup marker comment), and `merge` (declared enum variant; not yet implemented). Content matching drives idempotency; a marker-comment system distinguishes setmeup-managed content from user additions.

---

# Research

## Tool-specific config file locations

A reference for common tool config files. Not exhaustive — agents should confirm the correct path for the specific tool.

| Tool      | Config path | Format | Strategy |
|-----------|------------|--------|----------|
| git       | `~/.gitconfig` | INI-like | overwrite-managed (with marker comment) |
| ripgrep   | `~/.ripgreprc` | line-per-flag | overwrite-managed |
| yt-dlp    | `~/.config/yt-dlp/config` | line-per-flag | create-only |
| alacritty | `~/.config/alacritty/alacritty.toml` | TOML | create-only |
| nvim      | `~/.config/nvim/init.lua` | Lua | create-only or overwrite-managed |
| tmux      | `~/.tmux.conf` | line-per-option | overwrite-managed |
| ssh       | `~/.ssh/config` | INI-like | create-only (ownership/permission sensitive) |
| vscode    | `~/.config/Code/User/settings.json` | JSON | create-only only (rejected for overwrite-managed) |
| zsh       | `~/.zshrc` | shell | handled by `manifest-shell` |

## Format compatibility for overwrite-managed

The `overwrite-managed` strategy works ONLY for comment-supporting formats. The implemented `validate()` rejects `overwrite-managed` for paths ending in `.json` or `.xml` (comment-less formats). For other comment-less or binary formats, use `create-only`:

| Format | Comment syntax | `overwrite-managed`? |
|--------|---------------|----------------------|
| Shell script | `#` | Yes |
| INI / `.gitconfig` | `#` or `;` | Yes (use `#` marker) |
| YAML | `#` | Yes |
| Lua | `--` | Yes |
| C-style (`//`) | `//` | Yes |
| Line-per-flag | `#` | Yes |
| gitignore | `#` | Yes |
| **JSON** | **none** | **No — rejected at validation** |
| **XML** | **none (block only)** | **No — rejected at validation** |
| **Binary** | none | No — use `create-only` |

Marker variants by format: `# managed by setmeup` (shell/YAML/Python/INI), `// managed by setmeup` (C-family/CSS/JS/TS), `; managed by setmeup` (INI-style). The exact marker line used by the implementation is emitted by the `ConfigsHandler`.

---

# Manifest schema

## Block structure

```yaml
configs:                        # map tool-name -> ConfigFile
  <tool-name>:                  # arbitrary label (e.g. yt-dlp, git)
    path: <absolute-file-path>  # required; absolute or ~-relative
    content: <multiline-string> # required; complete file content
    strategy: <strategy>        # optional; default create-only
```

## Required vs optional fields

| Field      | Required | Description |
|------------|----------|-------------|
| `path`     | ✅       | File path for the config file (e.g., `~/.config/yt-dlp/config`, `~/.gitconfig`). Tilde expansion happens at provisioning time. |
| `content`  | ✅       | File content as a YAML literal block. The complete file content. |
| `strategy` |          | One of: `create-only`, `overwrite-managed`, `merge`. Default: `create-only`. |

There is no `if-absent` field and no per-entry nested sub-options in the implemented schema.

## Strategy semantics

| Strategy | Description | Marker required? | Overwrites user changes? |
|----------|-------------|------------------|--------------------------|
| `create-only` | Writes only if `path` does not exist. If the file exists (regardless of content), it is left untouched and reported user-managed. | No | No |
| `overwrite-managed` | Writes if `path` does not exist OR the existing file contains the setmeup marker. Regenerates the full file. Refuses (reports divergence) if the file exists without the marker. Only valid for comment-capable formats. | Yes | Yes (only files bearing the marker) |
| `merge` | Declared enum variant; line-blend semantics with conflict flagging are not yet implemented. | Yes (planned) | Planned |

### create-only (default)

The safest strategy. Writes only when the target does not exist. Once the file exists — whether created by setmeup or the user — it is never modified. Correct for:
- Comment-less formats (JSON, XML, binary).
- Configs that other tools also manage.
- Configs the user customizes heavily after first creation.

### overwrite-managed

Valid only for comment-supporting formats. The written content includes a marker comment near the top so re-runs can recognize setmeup ownership. If a file exists without the marker, setmeup refuses to overwrite it (safety guard).

### merge

Declared enum variant (`ConfigStrategy::Merge`) but NOT implemented — provisioning treats it as not-yet-supported. Do not emit `merge` in manifests expecting behavior; it is an explicit marker for future block-merge semantics.

## Validation rules (implemented)

1. `path` must be non-empty.
2. `content` must be non-empty.
3. `strategy` must be one of `create-only`, `overwrite-managed`, `merge`.
4. `overwrite-managed` for `.json` or `.xml` paths is rejected at manifest validation: only `create-only` is valid for comment-less formats.

## Example YAML

```yaml
configs:
  yt-dlp:
    path: ~/.config/yt-dlp/config
    strategy: create-only
    content: |
      -o ~/downloads/%(title)s.%(ext)s

  git:
    path: ~/.gitconfig
    strategy: overwrite-managed
    content: |
      # managed by setmeup
      [user]
        name = Jane Doe
      [core]
        editor = nvim
```

---

# Provisioning

## Handler behavior

The `ConfigsHandler` (in `src/provisioning.rs`) walks the declared `configs` map in fixed order and per entry:

1. `check_config`: 
   - File absent → `NeedsProvision`
   - File present, content matches → `Satisfied`
   - File present, differs → for `create-only`: `ManagedByUser` (never write); for `overwrite-managed`: marker present → `NeedsProvision`, marker absent → `ManagedByUser`
2. `provision_config`: only invoked on `NeedsProvision`; ensures parent dirs, applies quoting if needed, writes the file (adding the marker header for `overwrite-managed`).

## Verification

```bash
# File exists at target path
test -f ~/.config/yt-dlp/config

# Marker present (for overwrite-managed)
grep -q 'managed by setmeup' ~/.gitconfig
```

## Dependencies on other capabilities

- **Recommended ordering:** `packages` → `configs` so tool-specific configs are written after the tool is installed (fixed order in `provision()`).
- No runtime dependency on other sections.

---

# Idempotency

## How to check if already satisfied

**create-only:** target `path` exists → satisfied permanently (setmeup never re-touches it).

**overwrite-managed:** target exists AND marker line present AND full content matches declared content byte-for-byte.

## What constitutes "divergent" state

**create-only:** no divergence possible — files are only created once.

**overwrite-managed:**
- File missing → create it.
- Marker present, content differs → regenerate (divergent, will rewrite).
- Marker absent → user-managed; report and do NOT overwrite.

## What state is "user-managed"

- Any file without the setmeup marker, under any strategy, is user-managed.
- For `create-only`, every existing file is treated as user-managed forever.
- Removal of a config from the manifest does NOT delete the file on disk (convergence-only model).

---

# Wizard

## Input type

| Field      | Type | Widget |
|------------|------|--------|
| `(name)`   | free text | Label for the config entry (map key) |
| `path`     | free text | Target path |
| `content`  | free text | Multi-line text area |
| `strategy` | list toggle | `create-only`, `overwrite-managed`, `merge` (merge marked not-yet-implemented) |

## Default values

- `strategy`: `create-only` (safest default).
- `content`: empty (user must provide).

## Validation rules (wizard-specific)

- `path` must start with `/` or `~` and be a file path.
- `overwrite-managed` is only offered when the path does not end in `.json`/`.xml`.
- `content` must be non-empty.