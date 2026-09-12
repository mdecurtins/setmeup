---
name: manifest-shell
description: Document and implement the `shell` section of the setmeup manifest — shell function definitions, PATH extensions, environment variables, rc-file sourcing, and PS1 prompt customization.
license: MIT
metadata:
  author: setmeup
  version: "1.0"
---

# Overview

The `shell` section manages interactive shell configuration beyond simple aliases: shell functions (complex multi-line logic), PATH extensions (idempotent directory additions), environment variable exports, and PS1 prompt customization with git-branch integration.

Managed files (in the implemented schema):
- `~/.config/setmeup/shell-functions.sh` — shell function definitions AND the prompt/PS1, PATH-extend guard blocks, and env exports (all written into this one managed file).
- `~/.config/setmeup/shell-aliases.sh` — aliases, owned by `manifest-aliases`.

Both are sourced from the user's rc file via idempotent line-append.

---

# Research

## Bashrc sourcing mechanics

- **Bash startup files:** https://www.gnu.org/software/bash/manual/bash.html#Bash-Startup-Files
- **Execution order (interactive login shells):** `/etc/profile` → `~/.bash_profile` (or `~/.bash_login` or `~/.profile`) → `~/.bashrc` (if the profile sources it).
- **Execution order (interactive non-login shells):** `~/.bashrc` directly.
- **Key insight:** On Ubuntu, `~/.bash_profile` does not exist by default; `~/.profile` sources `~/.bashrc`. So `~/.bashrc` is the primary config file for interactive shells.

## PS1 customization

- **Prompt escape sequences:** https://www.gnu.org/software/bash/manual/bash.html#Controlling-the-Prompt
- **PS1 variables:** `\u` (user), `\h` (hostname), `\w` (working directory), `\W` (basename), `\$` (`#` root / `$` normal).
- **Colors:** Wrap color codes in `\[\e[...m\]` so bash computes cursor position correctly.
- **Git branch in prompt:** Requires a `parse_git_branch` function that extracts the current branch from `HEAD`; the function body is declared in `shell.functions` and referenced from PS1.

## PATH management

- **Duplicate prevention:** check `case ":$PATH:" in *:"$dir":*) ;; *) PATH="$dir:$PATH" ;; esac` before prepending.
- **Prepend vs append:** prepend (`PATH="$dir:$PATH"`) gives priority; use it for tools that should shadow system versions.

## Platform availability

| Platform | rc file | Notes |
|----------|---------|-------|
| Ubuntu (bash) | `~/.bashrc` | Primary target |
| macOS (zsh)   | `~/.zshrc` | Works; zsh uses `PROMPT` vs `PS1` |
| Ubuntu (zsh)  | `~/.zshrc` | Works |
| Windows (WSL2) | `~/.bashrc` | Works |

---

# Manifest schema

## Block structure (implemented)

```yaml
shell:
  prompt: true                # BOOL — enable colored PS1 with git branch
  functions:                  # optional list of {name, body}
    - name: parse_git_branch
      body: |
        git branch 2> /dev/null | sed -e "/^[^*]/d;s/* \(.*\)/(\1)/"
  source:                     # optional list of additional rc paths to source
    - ~/.config/setmeup/shell-functions.sh
  path-extend:                # optional list of dirs to prepend to PATH
    - ~/.opencode/bin
  env:                        # optional map KEY -> VALUE
    BROWSER: /mnt/c/Program Files/Mozilla Firefox/firefox.exe
  # legacy (kept for V1 compat): rc-lines: [...]
```

## Required vs optional fields

| Field | Required | Description |
|-------|----------|-------------|
| `prompt` | | **Bool.** When `true`, setmeup writes a colored PS1 that includes the git branch. |
| `functions` | | List of `{name, body}` shell function definitions. |
| `functions[].name` | ✅ (per entry) | Function name (valid shell identifier). |
| `functions[].body` | ✅ (per entry) | Function body (code following the `name() {` opening). |
| `source` | | List of additional files to source from the rc file. |
| `path-extend` | | List of directories to prepend to PATH (idempotent guarded blocks). |
| `env` | | Map of environment variable key → value. Emitted as `export KEY="<value>"` (double-quoted, escaped). |

## Validation rules

1. `env` keys must match `^[A-Za-z_][A-Za-z0-9_]*$` (valid shell variable names).
2. `functions[].name` must be a valid shell function name.
3. `functions[].body` must be non-empty and the inner code only (no `function name {` wrapper).
4. `path-extend` entries are deduplicated.

## Example YAML

```yaml
shell:
  prompt: true
  functions:
    - name: parse_git_branch
      body: |
        git branch 2> /dev/null | sed -e "/^[^*]/d;s/* \(.*\)/(\1)/"
  source:
    - ~/.cargo/env
    - ~/.local/bin/env
  path-extend:
    - ~/.opencode/bin
    - ~/.local/share/pnpm/bin
  env:
    BROWSER: /mnt/c/Program Files/Mozilla Firefox/firefox.exe
    OPENCODE_EXPERIMENTAL_BACKGROUND_SUBAGENTS: "true"
```

---

# Provisioning

## Handler behavior

The `ShellHandler` (in `src/provisioning.rs`) checks first, then provisions:

1. `check_shell`:
   - If the shell section declares NOTHING (no prompt, no functions, no source, no path-extend, no env) → `Satisfied`.
   - If functions declared: satisfied iff `~/.config/setmeup/shell-functions.sh` exists with matching function names AND the `. "$HOME/.config/setmeup/shell-functions.sh"` source line exists in the rc file. Differs → `Divergent`.
   - If prompt true: satisfied iff `parse_git_branch` appears in rc (PS1 references it).
   - If source declared: satisfied iff each source line exists in rc.
   - If path-extend declared: satisfied iff each dir appears in rc.
   - If env declared: satisfied iff each `export KEY="..."` line present in rc.
2. `provision_shell` (only on `NeedsProvision`): ensure `~/.config/setmeup/` exists, write functions file (with `# managed by setmeup` header), write prompt/PS1 if enabled, ensure each source line / PATH guard / env export appended idempotently. Env values are double-quoted with escaping; never emitted unquoted.

## Managed files

| File | Purpose | Managed? |
|------|---------|----------|
| `~/.config/setmeup/shell-functions.sh` | Functions, prompt, PATH guards, env exports | Fully managed |
| `~/.config/setmeup/shell-aliases.sh` | Aliases (owned by `manifest-aliases`) | Fully managed |
| `~/.bashrc` (or `~/.zshrc`) | User rc file; setmeup only appends sourcing lines | Line-appended only |

There is NO `shell-env.sh` in the implemented schema — env vars and PATH extensions live in the functions file.

## Sourcing in rc file

setmeup ensures the following lines are present in the user's rc file (content-based idempotent line-append):

```bash
. "$HOME/.config/setmeup/shell-functions.sh"
. "$HOME/.config/setmeup/shell-aliases.sh"
```

The `source` list adds additional `. <path>` lines.

## Verification

```bash
# Functions file exists and is syntactically valid
test -f ~/.config/setmeup/shell-functions.sh && bash -n ~/.config/setmeup/shell-functions.sh

# Sourcing line present in rc exactly once
grep -c 'shell-functions.sh' ~/.bashrc

# Function defined
bash -c '. ~/.config/setmeup/shell-functions.sh && type parse_git_branch'

# Env var set (double-quoted)
bash -c '. ~/.config/setmeup/shell-functions.sh && echo "$BROWSER"'
```

## Dependencies on other capabilities

- **Ordering:** `packages` → `repos` → `aliases` → `configs` → `shell`. Shell runs last so it can reference everything declared earlier.
- Sourcing lines for both managed files are owned by the shell handler.

---

# Idempotency

## How to check if already satisfied

1. Functions file exists with the declared function set (+ prompt/other content) and the rc source line present.
2. Each declared source line, PATH guard, and env export is present in the rc file exactly once.

## What constitutes "divergent" state

- Managed file content differs from declared → `Divergent`; reported, NOT silently overwritten (per the check-before-provision review fix).
- File missing → `NeedsProvision`.
- rc sourcing line missing → `NeedsProvision`.
- PATH entry / env export / function missing → `NeedsProvision`.

## What state is "user-managed"

- User's own `.bashrc` content outside the setmeup sourcing lines is never touched.
- Any existing file WITHOUT the setmeup marker is preserved (`Divergent`/`ManagedByUser` path, never overwritten).

---

# Wizard

## Input type

| Field | Type | Widget |
|-------|------|--------|
| `prompt` | yes/no | Toggle: enable colored PS1 with git branch |
| `functions` | multi-block | Named function with multi-line body |
| `source` | multi free text | Add/remove additional rc source paths |
| `path-extend` | multi free text | Add/remove PATH directories |
| `env` | key-value list | Dynamic add/remove of `KEY: VALUE` |

## Default values

- `prompt`: `true`
- `functions`: `[{ name: parse_git_branch, body: <standard git branch function> }]` offered when prompt is enabled
- `source`, `path-extend`, `env`: empty

## Validation rules (wizard-specific)

- `path-extend` entries must be absolute (start with `/` or `~`) and not duplicate.
- `env` keys must be valid shell identifiers.
- Function names must be unique and not conflict with alias names.
- Custom function bodies should be syntactically valid (`bash -n` on a constructed snippet).