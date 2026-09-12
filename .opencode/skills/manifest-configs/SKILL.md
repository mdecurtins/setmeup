---
name: manifest-configs
description: Document and implement the `configs` section of the setmeup manifest — declarative tool/dotfile configuration files with three idempotent strategies: create-only, overwrite-managed, and merge.
license: MIT
metadata:
  author: setmeup
  version: "1.0"
---

# Overview

The `configs` section manages tool configuration files (dotfiles beyond the simple source→dest symlink pattern in `dotfiles`). It supports three strategies for applying config content to a target path, chosen based on the file format and how aggressively setmeup should manage the content.

This is more capable than the simple `dotfiles` symlink section because it supports strategy choice, idempotency based on content matching, and a marker-comment system for distinguishing setmeup-managed content from user additions.

---

# Research

## Tool-specific config file locations

A reference for common tool config files. Not exhaustive — agents should confirm the correct path for the specific tool.

| Tool      | Config path | Format | Strategy |
|-----------|------------|--------|----------|
| git       | `~/.gitconfig` | INI-like | overwrite-managed (with `# setmeup` markers) |
| ripgrep   | `~/.ripgreprc` | line-per-flag | overwrite-managed |
| fd        | `~/.config/fd/fdignore` | gitignore-like | overwrite-managed |
| bat       | `~/.config/bat/config` | line-per-flag | overwrite-managed |
| alacritty | `~/.config/alacritty/alacritty.toml` | TOML | create-only (not safe to overwrite-managed) |
| starship  | `~/.config/starship.toml` | TOML | create-only |
| nvim      | `~/.config/nvim/init.lua` | Lua | create-only or overwrite-managed |
| tmux      | `~/.tmux.conf` | line-per-option | overwrite-managed (with `# setmeup` markers) |
| ssh       | `~/.ssh/config` | INI-like | create-only (ownership/permission sensitive) |
| vscode    | `~/.config/Code/User/settings.json` | JSON | ❌ create-only (JSON merge not supported) |
| zsh       | `~/.zshrc` | shell | handled by `shell` and `shell-functions.sh` |

## Format compatibility matrix for overwrite-managed

The `overwrite-managed` strategy works ONLY for comment-supporting formats. "Comment-supporting" means the format has a line-comment syntax that setmeup can use for its managed-file markers.

| Format | Comment syntax | `overwrite-managed` supportable? |
|--------|---------------|----------------------------------|
| Shell script | `#` | ✅ Yes |
| INI / `.gitconfig` | `#` or `;` | ✅ Yes (use `#` marker) |
| Python/TOML | `#` | ✅ Yes (TOML officially supports `#`) |
| YAML | `#` | ✅ Yes |
| Lua | `--` | ✅ Yes |
| C-style (`//`) | `//` | ✅ Yes |
| Line-per-flag | `#` | ✅ Yes |
| gitignore | `#` | ✅ Yes |
| **JSON** | **none** | **❌ No** — must use `create-only` |
| **XML** | `<!-- -->` | **❌ No** — block comments only; use `create-only` |
| **Binary** | none | **❌ No** — must use `create-only` |
| **TOML** | `#` | ❌ No by policy — see risk notes |

**Policy rule for TOML:** Despite having `#` comments, TOML is excluded from `overwrite-managed` because:
1. TOML is commonly machine-edited; users may have tables/keys added by other tools that would be lost on overwrite.
2. Key ordering is semantically meaningful in TOML; overwriting can change behavior if the tool expects a specific section order.
3. Arbitrary overwrite of `~/.config/starship.toml` or `~/.cargo/config.toml` could break the user's tool setup.

---

# Manifest schema

## Block structure

```yaml
configs:
  - path: <absolute-file-path>    # required
    content: <multiline-string>    # required (inline or literal block)
    strategy: <strategy>           # optional, default: create-only
    if-absent: <path>              # optional, only apply if this path exists
```

## Required vs optional fields

| Field      | Required | Description |
|------------|----------|-------------|
| `path`     | ✅       | Absolute file path for the config file (e.g., `~/.gitconfig`, `~/.config/starship.toml`). Tilde expansion happens at provisioning time. |
| `content`  | ✅       | File content as a YAML literal block. This is the complete file content (for create-only and overwrite-managed) or the managed-segment content (for merge, future). |
| `strategy` |          | One of: `create-only`, `overwrite-managed`. Default: `create-only`. |
| `if-absent`|          | A path to check. If the specified file or directory does not exist, this config entry is skipped (useful for tool-specific configs that should only apply when the tool is installed). |

## Strategy semantics

| Strategy | Description | Marker required? | Overwrites user changes? |
|----------|-------------|------------------|--------------------------|
| `create-only` | Only writes if `path` does not exist. If the file exists (regardless of content), it is left untouched. | No | No |
| `overwrite-managed` | Writes only if `path` exists AND the file content matches a setmeup marker pattern. Regenerates the entire file from `content`. If no marker is detected, it refuses and reports `Failed`. | Yes | Yes (only managed files) |

### create-only (default)

The safest strategy. Writes the file only when the target path does not exist. Once the file exists — whether created by setmeup or by the user — it is never modified. This is correct for:
- Binary format configs (JSON, XML, compiled).
- Configs that other tools also manage.
- Configs that, once created, the user customizes heavily.

### overwrite-managed (requires comment markers)

Only valid for formats that support line comments (`#`, `//`, `;`, `--`). The config file content must include a marker comment near the top:

```
# >>> setmeup managed — edits may be overwritten
```

When the file does not contain this marker, setmeup refuses to overwrite it (safety guard). When the marker is present, setmeup may regenerate the entire file.

### merge (future, not yet implemented)

A future strategy that allows surgically inserting setmeup-managed blocks into a larger file while preserving user-maintained sections around them. Not yet implemented. When designing merge, the block-marker pattern would be:

```
# === setmeup: section-name begin ===
...managed content...
# === setmeup: section-name end ===
```

## Validation rules

1. `path` must be non-empty and absolute (starting with `/` or `~`).
2. `content` must be non-empty.
3. `strategy` must be one of: `create-only`, `overwrite-managed`.
4. `overwrite-managed` strategy is rejected for JSON, XML, and any format whose filename extension does not line-comment-supporting. Reference list at time of validation:
   - Allowed: `.sh`, `.bash`, `.zsh`, `.conf`, `.cfg`, `.ini`, `.gitconfig`, `.tmux.conf`, `.lua`, `.py`, `.yaml`, `.yml`, `.gitignore`, `.ripgreprc`, `.editorconfig`, `.mermaid`, `.env`, `.tool-versions`.
   - Rejected: `.json`, `.xml`, `.toml`, `.bin`, `.exe`, `.png`, `.jpg`, `.pdf`, `.lock`, `.mod`, `.sum`.
5. `if-absent` when provided must be an absolute path (file or directory).
6. Content for overwrite-managed MUST include the marker line `# >>> setmeup managed — edits may be overwritten` (or the appropriate comment-syntax variant) as the first or second line.
7. `path` values must be unique across the `configs` list.

## Example YAML

```yaml
configs:
  - path: ~/.gitconfig
    strategy: overwrite-managed
    content: |
      # >>> setmeup managed — edits may be overwritten
      [user]
        name = Jane Doe
        email = jane@example.com
      [core]
        editor = nvim
        pager = delta
      [init]
        defaultBranch = main

  - path: ~/.config/starship.toml
    strategy: create-only
    content: |
      format = "$all"
      add_newline = true
      [character]
        success_symbol = "[➜](bold.green)"
      [nodejs]
        format = "via [⬢ $version](bold.green) "

  - path: ~/.tmux.conf
    strategy: overwrite-managed
    if-absent: /usr/bin/tmux
    content: |
      # >>> setmeup managed — edits may be overwritten
      set -g default-terminal "tmux-256color"
      set -ga terminal-overrides ",*256col*:Tc"
      set -g mouse on
      bind r source-file ~/.tmux.conf
```

---

# Provisioning

## Install command template

```rust
fn apply_config(path: &Path, content: &str, strategy: &Strategy) -> Result<ItemStatus> {
    let expanded = path.to_str().unwrap().replace('~', &home_dir());
    let target = Path::new(&expanded);

    match strategy {
        Strategy::CreateOnly => {
            if target.exists() {
                return Ok(ItemStatus::Satisfied); // leave untouched
            }
            // Create parent dirs, write file
            fs::create_dir_all(target.parent().unwrap())?;
            fs::write(target, content)?;
            Ok(ItemStatus::Satisfied)
        }

        Strategy::OverwriteManaged => {
            if !target.exists() {
                return Ok(ItemStatus::Satisfied); // nothing to manage yet
            }
            let existing = fs::read_to_string(target)?;
            if !existing.contains(MANAGED_MARKER_LINE) {
                return Ok(ItemStatus::Failed(
                    "Refusing to overwrite: file exists but no setmeup marker found".into()
                ));
            }
            if existing.trim() == content.trim() {
                return Ok(ItemStatus::Satisfied); // content matches, skip
            }
            fs::write(target, content)?;
            Ok(ItemStatus::Satisfied)
        }
    }
}
```

## Config file location and format

- `path` determines the destination; setmeup creates parent directories as needed.
- No central managed-file directory — each config lives at the tool's expected path.
- The marker comment `# >>> setmeup managed — edits may be overwritten` is used for language-appropriate comment syntax:
  - `#` for shell, YAML, Python, INI, .gitignore, etc.
  - `//` for C-style languages (`.lua` uses `--`, which setmeup respects).
  - `;` for some INI variants.

## Verification command

```bash
# File exists at target path
test -f ~/.gitconfig

# File content matches (for overwrite-managed)
diff -q <(echo "$declared_content") ~/.gitconfig

# Marker is present (for overwrite-managed)
grep -q 'setmeup managed' ~/.gitconfig

# If using create-only: file was not overwritten if it existed
test "$(stat -c %Y ~/.gitconfig)" -le "$(stat -c %Y ~/.config/setmeup/.last-apply)"
```

## Dependencies on other capabilities

- **No strict ordering dependencies.** Configs can be applied before or after package installation.
- **`if-absent` dependencies:** If a config uses `if-absent: /usr/bin/tmux`, the `packages` section must install tmux before this config runs.
- **Recommended ordering:** `packages` → `configs` so tool-specific configs are written after the tool is installed.

---

# Idempotency

## How to check if already satisfied

**create-only:**
1. Target path exists (file or symlink).
2. Content is not compared; any content at the target is accepted.

**overwrite-managed:**
1. Target path exists.
2. The `setmeup managed` marker line is present in the first 3 lines of the file.
3. The full file content matches the declared content (byte-for-byte).

If ALL conditions for the strategy are satisfied, the config entry reports `Satisfied` and no write occurs.

## What constitutes "divergent" state

**create-only:**
- No divergence possible — once the file exists, setmeup never touches it again.

**overwrite-managed:**
- **File missing:** the marker cannot be found → create the file (it's managed, so no need to be cautious about overwriting).
- **Marker present, content differs:** the file has a setmeup marker but the content doesn't match. → Overwrite with the declared content.
- **Marker absent, file was created by user:** setmeup refuses to overwrite and reports `ItemStatus::Failed`. The user must either add the marker manually or remove the file.
- **Marker absent, file was created by setmeup in a previous version:** setmeup recognizes it as a managed legacy file only if a companion marker file exists (future: `~/.config/setmeup/managed-files/<hash>`).

## What state is "user-managed" (no setmeup marker)

- Any config file that does not contain the setmeup marker comment is considered user-managed. The `overwrite-managed` strategy refuses to touch it.
- For `create-only` strategy, all existing files are treated as user-managed (setmeup never overwrites them).
- The distinction is purely by marker absence, not by content comparison.

---

# Wizard

## Input type

| Field      | Type | Widget |
|------------|------|--------|
| `path`     | free text | Text input; show autocomplete for common config paths |
| `content`  | free text | Multi-line text area; syntax highlighting by file extension |
| `strategy` | list toggle | Options: `create-only`, `overwrite-managed` |
| `if-absent`| free text | Optional path input |

## Default values

- `strategy`: `create-only` (safest default).
- `content`: empty (user must provide).
- `if-absent`: empty (always apply).

## Validation rules (wizard-specific)

- `path` must start with `/` or `~`.
- `path` must be a file path, not just a directory (the wizard rejects paths ending in `/`).
- `strategy` `overwrite-managed` is only offered when the file extension is in the allowed list (see "Format compatibility matrix" above).
- `content` must be non-empty.
- If `strategy` is `overwrite-managed`, the wizard automatically prepends the marker line `# >>> setmeup managed — edits may be overwritten\n` to the content if absent.
