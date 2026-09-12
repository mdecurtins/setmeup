---
name: manifest-shell
description: Document and implement the `shell` section of the setmeup manifest — shell function definitions, PATH extensions, environment variable declarations, RC file sourcing management, and PS1 prompt customization.
license: MIT
metadata:
  author: setmeup
  version: "1.0"
---

# Overview

The `shell` section manages interactive shell configuration beyond simple aliases. It covers shell functions (complex multi-line shell logic), PATH extensions (idempotent directory additions), environment variable declarations, and PS1 prompt customization with git branch integration.

Shell configuration is written into two managed files:
- `~/.config/setmeup/shell-functions.sh` — shell function definitions (sourced from rc file).
- `~/.config/setmeup/shell-env.sh` — environment variables and PATH extensions (sourced early in rc file).

These are sourced from the user's rc file via idempotent line-append, managed by setmeup alongside the aliases sourcing line (see `manifest-aliases`).

---

# Research

## Bashrc sourcing mechanics

- **Bash startup files:** https://www.gnu.org/software/bash/manual/bash.html#Bash-Startup-Files
- **Execution order for interactive login shells:** `/etc/profile` → `~/.bash_profile` (or `~/.bash_login` or `~/.profile`) → `~/.bashrc` (sourced by `~/.bashrc` if `~/.bash_profile` sources it) → done.
- **Execution order for interactive non-login shells:** `~/.bashrc` directly.
- **Key insight:** On Ubuntu, `~/.bash_profile` does not exist by default; `~/.profile` sources `~/.bashrc`. Most interactive shells are non-login (terminal emulators), so `~/.bashrc` is the primary config file.
- **Zsh:** Similar flow; `~/.zshrc` is the equivalent of `~/.bashrc`.

## Sourcing order within ~/.bashrc

The order matters. The recommended setmeup sourcing placement:

```bash
# (top of .bashrc — system settings, shopt)

# Setmeup environment (early — PATH, vars needed by everything else)
. ~/.config/setmeup/shell-env.sh

# Setmeup aliases (aliases are expanded first by bash)
. ~/.config/setmeup/shell-aliases.sh

# Setmeup functions (functions can use aliases)
. ~/.config/setmeup/shell-functions.sh

# (user's own customizations after)
```

## PS1 customization

- **Prompt escape sequences:** https://www.gnu.org/software/bash/manual/bash.html#Controlling-the-Prompt
- **PS1 variables:** `\u` (user), `\h` (hostname), `\w` (working directory), `\W` (basename of CWD), `\$` (`#` for root, `$` for normal).
- **Colors:** Use `\[\e[<code>m\]` to wrap color codes so bash can correctly calculate cursor position.
- **Git branch in prompt:** Requires a `parse_git_branch` function that extracts the current branch name from `HEAD`.
- **git-prompt.sh:** Git ships a `contrib/completion/git-prompt.sh` script that provides `__git_ps1`. The function approach is more portable and avoids sourcing an external script.

## PATH management

- **PATH format:** colon-delimited list of directories.
- **Duplicate prevention:** Always check if the directory is already in PATH before adding.
- **Prepend vs append:** Prepend (`PATH="$dir:$PATH"`) gives priority to the new directory; append (`PATH="$PATH:$dir"`) adds fallback. Use prepend for tools that should shadow system versions (e.g., `~/.cargo/bin` for rustup-managed rustc).
- **Guarded blocks pattern:**
  ```bash
  case ":$PATH:" in
    *:"$dir":*) ;;
    *) PATH="$dir:$PATH" ;;
  esac
  ```
  This pattern is idempotent and works in POSIX sh.

## Platform availability

| Platform | rc file | Notes |
|----------|---------|-------|
| Ubuntu (bash) | `~/.bashrc` | Primary target |
| macOS (zsh)   | `~/.zshrc` | Works; minor syntax differences (`setopt`, `PROMPT` instead of `PS1`) |
| Ubuntu (zsh)  | `~/.zshrc` | Works |
| Windows (WSL2) | `~/.bashrc` | Works |

---

# Manifest schema

## Block structure

```yaml
shell:
  prompt: <prompt-config>        # optional, PS1 customization
    style: <style>               # "minimal" | "full" | "custom"
    custom_ps1: <string>         # used when style: "custom"
    git_branch: <bool>           # show git branch in prompt
    color: <bool>                # enable colors in prompt

  path_extend:                   # optional list of PATH additions
    - <absolute-directory-path>

  env:                           # optional, key-value environment variables
    <KEY>: <VALUE>

  functions:                     # optional list of shell function definitions
    - name: <function-name>      # required
      body: <multiline-string>   # required, the function code (without wrapper)

  source_files:                  # optional list of additional files to source
    - <absolute-file-path>
```

## Required vs optional fields

| Field          | Required | Description |
|----------------|----------|-------------|
| `prompt`       |          | Prompt configuration object |
| `prompt.style` | ⚠️      | Required if `prompt` is present. One of: `minimal`, `full`, `custom`. |
| `prompt.custom_ps1` | ⚠️  | Required if `style: custom`. The literal PS1 value. |
| `prompt.git_branch` |      | Show git branch in prompt. Default: `true`. |
| `prompt.color` |          | Enable prompt colors. Default: `true`. |
| `path_extend`  |          | List of directories to add to PATH (prepended). |
| `env`          |          | Map of environment variable key-value pairs. |
| `functions`    |          | List of shell function definitions. |
| `functions[].name` | ✅   | Function name (valid shell identifier). |
| `functions[].body` | ✅   | Function body (the code that follows `name() {`). |
| `source_files` |          | Additional files to source from the rc file. |

## Validation rules

1. `prompt.style` must be one of: `minimal`, `full`, `custom`.
2. `prompt.custom_ps1` must be non-empty when `style: custom`.
3. `path_extend` entries must be absolute paths (starting with `/` or `~`).
4. `path_extend` entries are deduplicated (no two identical paths).
5. `env` keys must match `^[A-Za-z_][A-Za-z0-9_]*$` (valid shell variable names).
6. `functions[].name` must be a valid shell function name (`^[a-zA-Z_][a-zA-Z0-9_-]*$`).
7. `functions[].body` must be non-empty and must not contain the function wrapper (`function name {` / `}`) — only the inner code.
8. `source_files` entries must be absolute paths.
9. Function names must not conflict with alias names (if `aliases` is also present).

## Example YAML

```yaml
shell:
  prompt:
    style: full
    git_branch: true
    color: true

  path_extend:
    - ~/.cargo/bin
    - ~/.local/bin
    - ~/.npm-global/bin

  env:
    EDITOR: nvim
    VISUAL: nvim
    PAGER: less
    BROWSER: firefox
    DOTNET_CLI_TELEMETRY_OPTOUT: "1"

  functions:
    - name: mkcd
      body: |
        mkdir -p "$1" && cd "$1"
    - name: gcb
      body: |
        git checkout -b "$1"

  source_files:
    - ~/.config/setmeup/shell-aliases.sh
    - ~/.config/setmeup/shell-functions.sh
```

---

# Provisioning

## Install command templates

### Environment file (`~/.config/setmeup/shell-env.sh`)

```bash
cat > ~/.config/setmeup/shell-env.sh << 'ENV_EOF'
# setmeup environment — managed file, do not edit manually

# PATH extensions
case ":$PATH:" in
  *:"$HOME/.cargo/bin":*) ;;
  *) PATH="$HOME/.cargo/bin:$PATH" ;;
esac

case ":$PATH:" in
  *:"$HOME/.local/bin":*) ;;
  *) PATH="$HOME/.local/bin:$PATH" ;;
esac

# Env vars
export EDITOR=nvim
export VISUAL=nvim
export PAGER=less
ENV_EOF
```

### Functions file (`~/.config/setmeup/shell-functions.sh`)

```bash
cat > ~/.config/setmeup/shell-functions.sh << 'FUNCS_EOF'
# setmeup shell functions — managed file, do not edit manually

# Git branch for prompt
parse_git_branch() {
  local branch
  branch=$(git rev-parse --abbrev-ref HEAD 2>/dev/null)
  if [ -n "$branch" ] && [ "$branch" != "HEAD" ]; then
    echo " ($branch)"
  fi
}

mkcd() {
  mkdir -p "$1" && cd "$1"
}

gcb() {
  git checkout -b "$1"
}
FUNCS_EOF
```

### Prompt (written into the environment file or functions file)

```bash
# Colored PS1 with git branch (full style)
parse_git_branch() {
  git rev-parse --abbrev-ref HEAD 2>/dev/null
}

if [ "$color" = true ]; then
  PS1='\[\e[32m\]\u@\h\[\e[0m\]:\[\e[34m\]\w\[\e[33m\]$(parse_git_branch)\[\e[0m\]\$ '
else
  PS1='\u@\h:\w$(parse_git_branch)\$ '
fi
```

## Config file location and format

| File | Purpose | Managed? |
|------|---------|----------|
| `~/.config/setmeup/shell-env.sh` | PATH extensions, env vars, sourced early | ✅ Fully managed |
| `~/.config/setmeup/shell-functions.sh` | Shell functions, prompt, sourced after aliases | ✅ Fully managed |
| `~/.bashrc` (or `~/.zshrc`) | User rc file, setmeup adds sourcing lines | ⚠️ Line-appended only |

Both managed files get the standard header:
```bash
# setmeup <description> — managed file, do not edit manually
# Generated from manifest.yml. Edit the manifest and re-apply to change.
```

## Sourcing in RC file

setmeup ensures the following lines are present in the user's rc file (via content-based idempotent line-append, not regex):
```bash
. ~/.config/setmeup/shell-env.sh
. ~/.config/setmeup/shell-aliases.sh
. ~/.config/setmeup/shell-functions.sh
```

The `source_files` list in the manifest adds additional `. <path>` lines.

## Verification command

```bash
# Managed env file exists and is syntactically valid
test -f ~/.config/setmeup/shell-env.sh && bash -n ~/.config/setmeup/shell-env.sh

# Managed functions file exists and is syntactically valid
test -f ~/.config/setmeup/shell-functions.sh && bash -n ~/.config/setmeup/shell-functions.sh

# Sourcing lines present in rc
grep -c 'shell-env.sh' ~/.bashrc    # must be 1
grep -c 'shell-functions.sh' ~/.bashrc  # must be 1

# Functions are defined
bash -c '. ~/.config/setmeup/shell-functions.sh && type mkcd'

# PATH includes declared directories
bash -c '. ~/.config/setmeup/shell-env.sh && echo "$PATH"' | tr ':' '\n' | grep '.cargo/bin'

# Env vars are set
bash -c '. ~/.config/setmeup/shell-env.sh && echo "$EDITOR"'

# Prompt renders without errors (run in proper shell)
bash -i -c 'echo "$PS1"' 2>/dev/null || true
```

## Dependencies on other capabilities

- **Required after shell provisions:** The sourcing lines in the rc file. setmeup itself handles these.
- **Ordering concern with `aliases`:** `shell-aliases.sh` must be sourced before `shell-functions.sh` (functions can reference aliases). The sourcing order in the rc file is:
  1. `shell-env.sh` (PATH, env vars)
  2. `shell-aliases.sh` (aliases, handled by `manifest-aliases`)
  3. `shell-functions.sh` (functions, prompt, handled here)
- **In this context:** `manifest-shell` is responsible for the ENTIRE sourcing block, including the aliases sourcing line. It must be idempotent for all three lines together.

---

# Idempotency

## How to check if already satisfied

1. **Files exist:** Both `~/.config/setmeup/shell-env.sh` and `~/.config/setmeup/shell-functions.sh` exist.
2. **Content matches:** The content of each file, when compared after stripping the header comment, matches the parsed manifest declarations.
3. **Sourcing lines in rc:** The three sourcing lines exist in the rc file, each exactly once.
4. **PATH on env file:** Each declared `path_extend` entry appears in the file with a `case` guard block.
5. **Env vars present:** Each declared env var has an `export <KEY>=<VALUE>` line in the env file.
6. **Functions present:** Each declared function's body is present in the functions file.
7. **PS1 defined:** If prompt is declared, `PS1=` appears in the functions file (or env file).

If ALL conditions are satisfied, the `shell` block reports `Satisfied` and no files are written.

## What constitutes "divergent" state

- **File content differs:** Either managed file does not match the expected content. → Regenerate the entire file (both are fully managed).
- **File missing:** A managed file does not exist. → Create and write both managed files.
- **Sourcing line missing from rc file:** The `. ~/.config/setmeup/shell-env.sh` (or functions or aliases) line is absent. → Append it.
- **Sourcing line duplicated:** A sourcing line appears more than once. → The line-append mechanism prevents this (content-based check) but if a duplicate exists from manual editing, the first occurrence is kept and subsequent duplicates are removed on the next write.
- **PATH entry missing from env file:** A declared path_extend entry does not have a `case` guard in the env file. → Regenerate the env file.
- **Env var missing from env file:** A declared env var's `export` line is absent. → Regenerate the env file.
- **Function missing from functions file:** A declared function body is absent. → Regenerate the functions file.

## What state is "user-managed" (no setmeup marker)

- User's own additions to `~/.bashrc` above or below the setmeup sourcing block are never touched.
- Functions the user has defined directly in `~/.bashrc` (not in the managed functions file) are never touched.
- PATH additions the user has in `~/.bashrc` or `~/.profile` outside the managed env file are never touched.
- The three managed files are fully regenerated on any divergence — user edits to them are lost. This is by design: the files are declared as managed in their header comments.
- Removing `shell` from the manifest does NOT remove the managed files — it's a convergence-only model.

---

# Wizard

## Input type

| Field                | Type | Widget |
|----------------------|------|--------|
| `prompt.style`       | list toggle | Options: `minimal` (`\w\$ `), `full` (`user@host:path(branch)$ `), `custom` |
| `prompt.custom_ps1`  | free text | Shown only when `style: custom` |
| `prompt.git_branch`  | yes/no toggle | Show git branch in prompt |
| `prompt.color`       | yes/no toggle | ANSI colors in prompt |
| `path_extend`        | multi free text | Add/remove list of absolute paths |
| `env`                | key-value list | Dynamic add/remove of `KEY: VALUE` pairs |
| `functions`          | multi-block | Add named function with multi-line body text area |
| `source_files`       | multi free text | Add/remove list of absolute file paths |

## Default values

- `prompt.style`: `full`
- `prompt.git_branch`: `true`
- `prompt.color`: `true`
- `path_extend`: empty
- `env`: `{ EDITOR: nvim, VISUAL: nvim, PAGER: less }`
- `functions`: `[{ name: parse_git_branch, body: <standard_git_branch_function> }]` (added automatically if prompt is enabled and `git_branch: true`)
- `source_files`: empty (setmeup adds the three standard sourcing lines automatically)

## Validation rules (wizard-specific)

- `path_extend` entries must be absolute (start with `/` or `~`), must end with `bin` (warn if not), and must not duplicate.
- `env` keys must be valid shell identifiers; values are free text.
- `functions[].name` must be unique and not match an alias name.
- `functions[].body` must be syntactically valid bash (the wizard may run `bash -n` on a constructed snippet).
- `source_files` entries must be absolute paths and must not duplicate the already-managed sourcing lines for env, aliases, and functions.
- Custom PS1 must not exceed 500 characters.
