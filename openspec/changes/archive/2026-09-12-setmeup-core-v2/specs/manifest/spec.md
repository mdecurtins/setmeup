## ADDED Requirements

### Requirement: Manifest sections V2

The manifest SHALL support the following new top-level sections, each optional:

| Section | Purpose | Zero-value behavior |
|---------|---------|-------------------|
| `packages` | System packages and tools to install | No packages managed |
| `nvm` | Node version manager configuration | No nvm managed |
| `rustup` | Rust toolchain manager configuration | No rustup managed |
| `repos` | Git repositories to clone | No repos cloned |
| `aliases` | Shell aliases (name → command) | No aliases managed |
| `configs` | Per-tool configuration files | No configs managed |
| `shell` | Shell customization (functions, prompt, PATH, env, sources) | No shell customization |

#### Scenario: Empty manifest applies cleanly

- **WHEN** a user applies a manifest with no optional sections
- **THEN** every capability reports satisfied (zero work) and apply exits 0

### Requirement: Packages section

The `packages` section SHALL be a map of package name → config. The config
supports:

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `from` | string | `"apt"` on Ubuntu, `"brew"` on macOS, `"winget"` on Windows | Installation method: `apt`, `brew`, `winget`, `pip`, `download` |
| `depends` | list[string] | `[]` | Package names that must be provisioned first |
| `apt` | string | package name | Override package name for apt |
| `brew` | string | package name | Override package name for Homebrew |
| `winget` | string | package name | Override package name for winget |

#### Scenario: Package installs via declared method

- **WHEN** a manifest declares `yt-dlp: { from: pip }`
- **THEN** setmeup installs yt-dlp via pip, not apt

#### Scenario: Dependencies are provisioned first

- **WHEN** a manifest declares `yt-dlp: { from: pip, depends: [ffmpeg] }`
- **THEN** ffmpeg is provisioned before yt-dlp regardless of manifest ordering

#### Scenario: Package already present is skipped

- **WHEN** a declared package is already installed (dpkg/brew/winget/pip check)
- **THEN** it is reported as satisfied and not reinstalled

### Requirement: NVM section

The `nvm` section SHALL be a map with:

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `node-version` | string | `"lts/*"` | Node version to install via nvm |
| `global-packages` | list[string] | `[]` | npm packages to install globally after nvm setup |

If the section is absent, nvm is not configured.

**Trust-boundary note:** The NVM handler MUST install nvm via git clone of the
canonical repository (`https://github.com/nvm-sh/nvm.git`), NOT via
`curl | bash` from the install script. Setmeup verifies installation by
checking that `~/.nvm/nvm.sh` exists and is loadable. This is consistent with
decision D2a in design.md — `from: download` never means unchecked remote
execution.

#### Scenario: NVM installs specified node version

- **WHEN** `nvm: { node-version: "lts/*" }` is declared
- **THEN** setmeup installs nvm via git clone (if absent) and runs
  `nvm install lts/*` (if not already installed) and `nvm alias default lts/*`

#### Scenario: Global npm packages installed after node

- **WHEN** `nvm: { global-packages: [pnpm, bun] }` is declared
- **THEN** after node is installed, setmeup runs `npm install -g pnpm` and
  `npm install -g bun` (each only if not already installed)

#### Scenario: NVM already configured is skipped

- **WHEN** `~/.nvm/nvm.sh` exists and `nvm ls` shows the declared version
- **THEN** the step is reported satisfied

#### Scenario: NVM installation avoids curl-to-sh

- **WHEN** nvm is not installed
- **THEN** setmeup clones `https://github.com/nvm-sh/nvm.git` into `~/.nvm/`
  and sources it, rather than piping the install script from GitHub

### Requirement: Rustup section

The `rustup` section SHALL be a map with:

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `toolchain` | string | `"stable"` | Rust toolchain channel to install |

If the section is absent, rustup is not configured.

**Trust-boundary note:** The rustup handler installs rustup via the official
script (`rustup.rs`), downloaded with TLS verification. The downloaded script
is verified by its size and expected content (from known URL). This is
consistent with decision D2a — the rustup script is the canonical, verified
install path for the Rust project, analogous to how apt repos have their own
signing chains.

#### Scenario: Rustup installs stable toolchain

- **WHEN** `rustup: { toolchain: stable }` is declared and rustup is not installed
- **THEN** setmeup downloads and runs the official rustup installer, then sets
  the stable toolchain as default

#### Scenario: Rustup already configured is skipped

- **WHEN** `rustup` is already installed and the declared toolchain exists
- **THEN** the step is reported satisfied

#### Scenario: Cargo env is sourced after rustup

- **WHEN** rustup completes installation
- **THEN** `~/.cargo/env` is added to the shell source list (or the shell
  handler verifies it's already there)

### Requirement: Repos section

The `repos` section SHALL be a map of repo name → URL. Repos are cloned to
`~/repos/<name>/` by default.

#### Scenario: Repo is cloned

- **WHEN** a manifest declares `repos: { hoodhunter: "[EMAIL]:user/hoodhunter.git" }`
- **THEN** setmeup clones the repo to `~/repos/hoodhunter/` (if not already present)

#### Scenario: Repo already cloned is skipped

- **WHEN** the destination directory exists with a `.git` directory
- **THEN** the step is reported satisfied (no fetch/pull)

### Requirement: Aliases section

The `aliases` section SHALL be a map of alias name → command string. Aliases
are written to `~/.config/setmeup/shell-aliases.sh` and sourced from `.bashrc`.

#### Scenario: Alias is created

- **WHEN** a manifest declares `aliases: { hh: "cd ~/repos/hoodhunter" }`
- **THEN** the alias line is written into the managed aliases file

#### Scenario: Aliases file is sourced from bashrc

- **WHEN** any aliases are declared
- **THEN** setmeup ensures `. ~/.config/setmeup/shell-aliases.sh` is present in
  `.bashrc` (idempotent line-append)

### Requirement: Configs section

The `configs` section SHALL be a map of tool name → config definition:

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `path` | string | — | Absolute or home-relative path to the config file |
| `content` | string | — | File content (may include newlines) |
| `strategy` | string | `"create-only"` | Merge strategy: `create-only`, `overwrite-managed`, `merge` |

The `overwrite-managed` strategy SHALL only be valid for config file formats
that support comments: shell scripts, YAML, Python (`#`), C-family/JS/TS/CSS
(`//`), and INI-style (`;`). JSON, XML, and binary formats MUST reject
`overwrite-managed` at manifest validation time because they cannot carry a
setmeup ownership marker.

#### Scenario: Config file created if absent

- **WHEN** `strategy: create-only` and the file does not exist
- **THEN** the file is written with the declared content

#### Scenario: Existing config file without marker is preserved

- **WHEN** `strategy: create-only` and the file already exists
- **THEN** the file is not modified; step reports "user-managed, skipped"

#### Scenario: overwrite-managed rejected for JSON

- **WHEN** a manifest declares `strategy: overwrite-managed` for a `.json` file
- **THEN** validation fails with a message explaining that only `create-only` is
  valid for formats that don't support comments

### Requirement: Shell section

The `shell` section SHALL be a map with sub-sections:

| Field | Type | Description |
|-------|------|-------------|
| `prompt` | bool | Enable colored PS1 with git branch |
| `functions` | list | Named shell functions written to managed file |
| `source` | list[string] | Paths to source in `.bashrc` |
| `path-extend` | list[string] | Directories to prepend to PATH |
| `env` | map[string,string] | Environment variables to export |

#### Scenario: Shell functions written to managed file

- **WHEN** `shell.functions` contains one or more entries
- **THEN** each function is written to `~/.config/setmeup/shell-functions.sh`
  and a source line is appended to `.bashrc`

#### Scenario: PATH extensions are added idempotently

- **WHEN** `shell.path-extend` contains directories
- **THEN** each directory is prepended to PATH in a guarded block that checks
  for duplicates before appending

### Requirement: Manifest validation

Each section SHALL validate its own content on manifest parse.

#### Scenario: Invalid package config rejected

- **WHEN** a package declares `from: unknown`
- **THEN** validation fails with a message identifying the invalid method

#### Scenario: Empty alias name rejected

- **WHEN** an alias key is empty
- **THEN** validation fails with "aliases: name must not be empty"

## MODIFIED Requirements

### Requirement: Declarative manifest document

The system SHALL define machine desired state in a `manifest.yml` document
covering tools, dotfiles, shell preferences, OS preferences, secret-acquisition
policy, AND the new capability sections (packages, nvm, repos, aliases, configs,
shell customization). **The schema is now flat and strongly typed — each section
maps to a `ProvisionHandler` implementation.**

#### Scenario: Manifest parses and validates

- **WHEN** a user provides a well-formed manifest.yml with any combination of
  sections
- **THEN** the manifest parses and validates against the V2 schema

#### Scenario: Backward-compat tools section

- **WHEN** a manifest uses the legacy `tools` section
- **THEN** setmeup desugars it to `packages` with a deprecation notice
