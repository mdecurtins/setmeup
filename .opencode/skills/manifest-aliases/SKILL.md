---
name: manifest-aliases
description: Document and implement the `aliases` section of the setmeup manifest — declarative shell alias definitions written to a managed file and sourced from shell rc files.
license: MIT
metadata:
  author: setmeup
  version: "1.0"
---

# Overview

The `aliases` section declares shell aliases for interactive use. Aliases are written to `~/.config/setmeup/shell-aliases.sh` — a managed file that is sourced from the user's `.bashrc` (or equivalent) via a single sourcing line. This keeps alias definitions separate from the user's rc file, making them easy to manage, regenerate, and remove without touching the user's personal rc customizations.

---

# Research

## Bash alias syntax

- **Docs:** https://www.gnu.org/software/bash/manual/bash.html#Aliases
- **Syntax:** `alias name='value'` — single-quoted value prevents expansion at definition time.
- **Escaping rules:**
  - Single quotes inside the alias value must use `'\''` (end quote, escaped quote, reopen).
  - Double quotes inside the value are safe: `alias greet="echo 'hello'"` is fine.
  - Variables in the aliased text are expanded at **invocation time** (with single quotes) or **definition time** (with double quotes). Always prefer single quotes for the alias value body to defer expansion.
  - `!` (history expansion) does not need escaping inside single-quoted alias values in non-interactive contexts, but in interactive bash you may need `set +H` or use `\` escaping.
- **Multi-line aliases:** Use `\` continuation inside the quotes: `alias foo='echo line1 \` followed by newline and `> echo line2'`.

## Alias vs function

- **Alias:** Simple text substitution at the start of a command. Cannot handle arguments positionally (use shell functions from `manifest-shell` for that).
- **Function:** Full shell function, can accept `$1`, `$2`, etc. Functions belong in `manifest-shell` (shell functions), not in aliases.
- **Rule of thumb:** If it needs arguments or is more than one line of text, make it a function, not an alias.

## Sourcing mechanism

- Aliases are only expanded in interactive shells by default. To use aliases in scripts, `shopt -s expand_aliases` must be set.
- The sourcing line in `~/.bashrc` must be placed AFTER any `shopt` settings but ideally at the end (before `[[ $- != *i* ]] && return` if there's a return-early guard).
- setmeup writes: `. ~/.config/setmeup/shell-aliases.sh` (POSIX dot, not Bash `source` — works in all POSIX shells).

## Platform availability

| Platform | rc file | Status |
|----------|---------|--------|
| Ubuntu (bash) | `~/.bashrc` | ✅ Primary target |
| macOS (zsh)   | `~/.zshrc` | ✅ Works; zsh alias syntax is identical |
| Ubuntu (zsh)  | `~/.zshrc` | ✅ Works |
| macOS (bash)  | `~/.bashrc` | ✅ Works |
| Windows (WSL2) | `~/.bashrc` | ✅ Works |

---

# Manifest schema

## Block structure

```yaml
aliases:
  - name: <alias-name>      # required, unique
    value: <alias-command>   # required, shell command string
```

## Required vs optional fields

| Field   | Required | Description |
|---------|----------|-------------|
| `name`  | ✅       | Alias name (the word typed at the shell prompt). Must be a valid shell identifier. |
| `value` | ✅       | The command string the alias expands to. Use single-quote-safe syntax. |

## Validation rules

1. `name` must be non-empty, unique, and a valid shell identifier (`^[a-zA-Z_][a-zA-Z0-9_-]*$`).
2. `name` must not shadow an existing shell builtin (`cd`, `ls`, `echo`, `exit`, etc.) unless the manifest explicitly intends to override it — a warning is raised but not a hard error.
3. `value` must be non-empty.
4. `value` must not contain newlines (multi-line aliases are not supported; use `manifest-shell` functions instead).
5. Circular references (alias A pointing to alias B pointing to alias A) are not detectable statically; the shell will detect them at runtime.
6. Aliases can reference paths from declared repos (e.g., `my-deploy` → `~/repos/my-project/deploy.sh`). Dependencies on `repos` are implicit — provisioning order satisfies this.

## Example YAML

```yaml
aliases:
  - name: gs
    value: git status
  - name: gc
    value: git commit
  - name: gp
    value: git push
  - name: reload
    value: source ~/.bashrc
  - name: ll
    value: ls -la --color=auto
  - name: tmux-session
    value: tmux new-session -A -s main
```

---

# Provisioning

## Install command template

The aliases file is generated from the manifest at provisioning time:

```bash
# Write to ~/.config/setmeup/shell-aliases.sh
cat > ~/.config/setmeup/shell-aliases.sh << 'ALIASES_EOF'
# setmeup aliases — managed file, do not edit manually
# Generated from manifest.yml. Edit the manifest and re-apply to change.

alias gs='git status'
alias gc='git commit'
alias gp='git push'
alias ll='ls -la --color=auto'
# ... one alias per line
ALIASES_EOF
```

The aliases file must start with a managed-file header comment:
```bash
# setmeup aliases — managed file, do not edit manually
# Generated from manifest.yml. Edit the manifest and re-apply to change.
```

## Config file location and format

- **Managed file:** `~/.config/setmeup/shell-aliases.sh`
- **Format:** POSIX shell script, one `alias` definition per line, with a header comment block. Each alias uses single quotes around the value (automatic escaping applied by setmeup for values containing single quotes).
- **Sourcing in rc file:** setmeup adds a single line to the user's rc file:
  ```bash
  . ~/.config/setmeup/shell-aliases.sh
  ```
  This is done through the `shell` section's idempotent line-append mechanism (see `manifest-shell` skill). The line is added only once; content matching prevents duplication.

## Verification command

```bash
# File exists
test -f ~/.config/setmeup/shell-aliases.sh

# File sourceable (dry-run)
bash -n ~/.config/setmeup/shell-aliases.sh

# Each alias is actually defined
bash -c '. ~/.config/setmeup/shell-aliases.sh && alias gs'

# Sourcing line present in rc file
grep -F 'shell-aliases.sh' ~/.bashrc
```

## Dependencies on other capabilities

- **Required before aliases provision:** `repos` may be needed if any alias value references a path like `~/repos/<name>/...`. Provisioning order: `repos` → `aliases`.
- **Required after aliases provision:** `shell` section must include the sourcing line `. ~/.config/setmeup/shell-aliases.sh` in the rc file.
- **Ordering:** `packages` → `repos` → `aliases` → `shell`.
- **Interaction with `manifest-shell`:** The shell functions file at `~/.config/setmeup/shell-functions.sh` is sourced **after** the aliases file, so functions can reference aliases (though this is unusual — functions typically replace aliases for complex use cases).

---

# Idempotency

## How to check if already satisfied

1. **File exists:** `~/.config/setmeup/shell-aliases.sh` exists and is a regular file.
2. **Content matches:** The file content, when parsed, produces exactly the same alias definitions as the manifest declares. Comparison is done by parsing the file's `alias` lines and comparing key-value pairs, ignoring whitespace and comment lines.
3. **Sourcing line in rc:** The user's rc file contains `. ~/.config/setmeup/shell-aliases.sh` exactly once.

If ALL three are satisfied, the `aliases` block reports `Satisfied` and no file is written.

## What constitutes "divergent" state

- **File exists but content differs:** The alias definitions in the file do not match the manifest. → Regenerate the file entirely (full overwrite — the file is managed).
- **File missing:** No aliases file. → Generate from manifest.
- **Sourcing line missing from rc:** The `. ~/.config/setmeup/shell-aliases.sh` line is absent from the rc file. → Add it via the `shell` section's line-append mechanism.
- **Sourcing line duplicated:** There are multiple sourcing lines for the aliases file. → Deduplication is handled by the `shell` section's content-based skip.
- **Header comment modified:** If the user edits the managed file, re-apply detects the content mismatch (via parsed alias comparison) and overwrites the file entirely. The user is warned, not just silently overwritten.

## What state is "user-managed" (no setmeup marker)

- Aliases defined directly in `~/.bashrc` or `~/.bash_aliases` (outside the managed file) are never touched.
- User modifications to `~/.config/setmeup/shell-aliases.sh` are overwritten on re-apply if they diverge from the manifest — this is a managed file.
- The `~/.config/setmeup/` parent directory is managed (created if absent, not removed on un-declare).

---

# Wizard

## Input type

| Field   | Type | Widget |
|---------|------|--------|
| `name`  | free text | Text input for alias name |
| `value` | free text | Text input for alias command; multi-line not supported |

## Default values

No default values for aliases — the wizard starts with an empty list. Common presets can be offered as an "add common aliases" toggle in the wizard UI (e.g., `gs`, `gc`, `gp`, `ll`).

## Validation rules (wizard-specific)

- `name` must match `^[a-zA-Z_][a-zA-Z0-9_-]*$` and be unique in the current session.
- `name` with a leading `sudo` prefix is rejected (alias 'sudo-apt' is invalid; 'sapt' is acceptable).
- `value` must be non-empty and not exceed 500 characters (prevent pathological alias definitions).
- If `value` contains a single quote, the wizard automatically escapes it using the `'\''` sequence.
- A warning is shown (but not a hard error) if `name` shadows a shell builtin or a command on `$PATH`.
