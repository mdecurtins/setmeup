# manifest Specification

## Purpose

Define the machine desired state in a declarative `manifest.yml` document with flat, strongly-typed sections. Each section maps to a provisioning capability and is independently optional. The manifest is portable across OSes, contains NO secret material, and is the single source of truth for `setmeup apply`.

## Requirements

### Requirement: Declarative manifest document
The system SHALL define machine desired state in a `manifest.yml` document covering tools, dotfiles, shell preferences, OS preferences, secret-acquisition policy, AND the capability sections (packages, nvm, rustup, repos, aliases, configs, shell).

#### Scenario: Manifest parses and validates
- **WHEN** a user provides a well-formed manifest.yml with any combination of sections
- **THEN** the manifest parses and validates against the schema

#### Scenario: Invalid manifest rejected
- **WHEN** a user provides a malformed or schema-violating manifest
- **THEN** setmeup reports an error naming the offending section and refuses to apply

#### Scenario: Backward-compat tools section
- **WHEN** a manifest uses the legacy `tools` section
- **THEN** setmeup desugars it to `packages` with a deprecation notice

### Requirement: Manifest is portable across OSes
The manifest SHALL express targets in an OS-independent way, with per-OS resolution applied at provisioning time.

#### Scenario: Same manifest on Ubuntu and macOS
- **WHEN** the same manifest is applied on both Ubuntu and macOS
- **THEN** each machine resolves the tool/path to its platform-specific equivalent

#### Scenario: Empty manifest applies cleanly
- **WHEN** a user applies a minimal manifest with no optional sections
- **THEN** the resolve step produces a valid per-OS effective manifest using defaults

### Requirement: Manifest sections V2
The manifest SHALL support the following top-level sections, each optional:

| Section | Purpose | Zero-value behavior |
|---------|---------|-------------------|
| `packages` | System packages and tools to install | No packages managed |
| `nvm` | Node version manager configuration | No nvm managed |
| `rustup` | Rust toolchain manager configuration | No rustup managed |
| `repos` | Git repositories to clone | No repos cloned |
| `aliases` | Shell aliases (name → command) | No aliases managed |
| `configs` | Per-tool configuration files | No configs managed |
| `shell` | Shell customization (functions, prompt, PATH, env, sources) | No shell customization |

#### Scenario: Empty V2 manifest applies cleanly
- **WHEN** a user applies a manifest with no optional sections
- **THEN** every capability reports satisfied (zero work) and apply exits 0

### Requirement: Packages section
The `packages` section SHALL be a map of package name → config. The config supports:

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

**Trust-boundary note:** The NVM handler MUST install nvm via git clone of the canonical repository (`https://github.com/nvm-sh/nvm.git`), NOT via `curl | bash` from the install script. Setmeup verifies installation by checking that `~/.nvm/nvm.sh` exists and is loadable.

#### Scenario: NVM installs specified node version
- **WHEN** `nvm: { node-version: "lts/*" }` is declared
- **THEN** setmeup installs nvm via git clone (if absent) and runs `nvm install lts/*` (if not already installed) and `nvm alias default lts/*`

#### Scenario: Global npm packages installed after node
- **WHEN** `nvm: { global-packages: [pnpm, bun] }` is declared
- **THEN** after node is installed, setmeup runs `npm install -g pnpm` and `npm install -g bun` (each only if not already installed)

#### Scenario: NVM already configured is skipped
- **WHEN** `~/.nvm/nvm.sh` exists and `nvm ls` shows the declared version
- **THEN** the step is reported satisfied

### Requirement: Rustup section
The `rustup` section SHALL be a map with:

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `toolchain` | string | `"stable"` | Rust toolchain channel to install |

If the section is absent, rustup is not configured.

**Trust-boundary note:** The rustup handler installs rustup via the official script (`rustup.rs`), downloaded with TLS verification. This is consistent with the `download` verification policy.

#### Scenario: Rustup installs stable toolchain
- **WHEN** `rustup: { toolchain: stable }` is declared and rustup is not installed
- **THEN** setmeup downloads and runs the official rustup installer, then sets the stable toolchain as default

#### Scenario: Rustup already configured is skipped
- **WHEN** `rustup` is already installed and the declared toolchain exists
- **THEN** the step is reported satisfied

#### Scenario: Cargo env is sourced after rustup
- **WHEN** rustup completes installation
- **THEN** `~/.cargo/env` is added to the shell source list (or the shell handler verifies it's already there)

### Requirement: Repos section
The `repos` section SHALL be a map of repo name → URL. Repos are cloned to `~/repos/<name>/` by default.

#### Scenario: Repo is cloned
- **WHEN** a manifest declares `repos: { hoodhunter: "github.com/user/hoodhunter" }`
- **THEN** setmeup clones the repo to `~/repos/hoodhunter/` (if not already present)

#### Scenario: Repo already cloned is skipped
- **WHEN** the destination directory exists with a `.git` directory
- **THEN** the step is reported satisfied (no fetch/pull)

### Requirement: Aliases section
The `aliases` section SHALL be a map of alias name → command string. Aliases are written to `~/.config/setmeup/shell-aliases.sh` and sourced from `.bashrc`.

#### Scenario: Alias is created
- **WHEN** a manifest declares `aliases: { hh: "cd ~/repos/hoodhunter" }`
- **THEN** the alias line is written into the managed aliases file

#### Scenario: Aliases file is sourced from bashrc
- **WHEN** any aliases are declared
- **THEN** setmeup ensures `. ~/.config/setmeup/shell-aliases.sh` is present in `.bashrc` (idempotent line-append)

### Requirement: Configs section
The `configs` section SHALL be a map of tool name → config definition:

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `path` | string | — | Absolute or home-relative path to the config file |
| `content` | string | — | File content (may include newlines) |
| `strategy` | string | `"create-only"` | Merge strategy: `create-only`, `overwrite-managed`, `merge` |

The `overwrite-managed` strategy SHALL only be valid for config file formats that support comments: shell scripts, YAML, Python (`#`), C-family/JS/TS/CSS (`//`), and INI-style (`;`). JSON, XML, and binary formats MUST reject `overwrite-managed` at manifest validation time.

#### Scenario: Config file created if absent
- **WHEN** `strategy: create-only` and the file does not exist
- **THEN** the file is written with the declared content

#### Scenario: Existing config file without marker is preserved
- **WHEN** `strategy: create-only` and the file already exists
- **THEN** the file is not modified; step reports "user-managed, skipped"

#### Scenario: overwrite-managed rejected for JSON
- **WHEN** a manifest declares `strategy: overwrite-managed` for a `.json` file
- **THEN** validation fails with a message explaining that only `create-only` is valid for formats that don't support comments

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
- **THEN** each function is written to `~/.config/setmeup/shell-functions.sh` and a source line is appended to `.bashrc`

#### Scenario: PATH extensions are added idempotently
- **WHEN** `shell.path-extend` contains directories
- **THEN** each directory is prepended to PATH in a guarded block that checks for duplicates before appending

### Requirement: Secrets policy declared, not stored
The manifest SHALL declare how each credential is to be acquired (`paste`, `device-flow`, or `env`) and SHALL NOT contain secret material itself.

#### Scenario: Manifest contains a secret-acquisition policy
- **WHEN** a user views a manifest declaring a credential
- **THEN** it specifies the acquisition method and target backend, and contains no token or key material

### Requirement: Secret material excluded from git
The system SHALL architecturally prevent secret material from being written into git-tracked paths.

#### Scenario: Secret write path
- **WHEN** setmeup stores or reads a credential
- **THEN** it only writes to vault or backup paths outside the git repository

### Requirement: Defaults and sections
The manifest SHALL support defaulting so that absent optional sections resolve to sensible per-OS defaults.

#### Scenario: Empty manifest applies cleanly
- **WHEN** a user applies a minimal manifest with no optional sections
- **THEN** the resolve step produces a valid per-OS effective manifest using defaults

### Requirement: Manifest validation
Each section SHALL validate its own content on manifest parse.

#### Scenario: Invalid package config rejected
- **WHEN** a package declares `from: unknown`
- **THEN** validation fails with a message identifying the invalid method

#### Scenario: Empty alias name rejected
- **WHEN** an alias key is empty
- **THEN** validation fails with "aliases: name must not be empty"

