## Context

The setmeup-core MVP (archived change `2026-09-08-setmeup-core`) shipped a
functional provisioning engine with a well-defined trust boundary, secret
backend abstraction, and CLI surface. Production use reveals the next layer of
needs: the manifest must model real-world machine state more richly, the TUI
must be interactive enough to author that state, and the architecture must have
a clean mechanism for growing new capabilities.

This change is architectural — it establishes the foundation for adding
capabilities without rebuilding the engine each time. No new capability is
implemented here except the shell functions/aliases/configs that are trivial
reorganizations of existing patterns.

## Goals / Non-Goals

**Goals:**
- Flat, strongly-typed manifest schema with new sections (packages, nvm, repos,
  aliases, configs, shell)
- `ProvisionHandler` trait + capability dispatch in `provision()`
- No-clobber contract enforced by `check()` before `provision()` on every
  capability
- Linear crossterm wizard replacing the current stub (6 steps, full-screen,
  progress bar)
- Agent skill architecture: each capability gets a `manifest-<name>` skill
  encoding research + implementation
- `docs/ADDING-A-CAPABILITY.md` — the recipe for adding a new thing
- Remove dead dependencies (ratatui, anyhow) and dead-code stubs

**Non-Goals:**
- NOT implementing the actual capabilities beyond the engine + shell/aliases/
  configs — they get their own follow-up change(s)
- NOT a plugin system or dynamic registration — `ProvisionHandler` is a
  compile-time trait, not runtime
- NOT changing the secret backend, identity lifecycle, or self-update
  subsystem — those are stable
- NOT a prettier TUI — crossterm + ANSI, polished but minimal. No ratatui.

## Decisions

### D1. Manifest schema: flat sections with strongly-typed structs

**Decision:** Each capability gets a named top-level YAML key and a dedicated
Rust struct. The `Manifest` struct grows one field per capability. Example:

```yaml
# ── System packages and tools ──────────────────────
# apt is default on Ubuntu. Other methods: brew, winget, pip, download.
packages:
  # Native apt packages (OS package manager handles signing)
  jq: {}
  gh: {}
  tmux: {}
  mupdf-tools: {}
  # Docker ecosystem (6 apt packages from Docker's own repo)
  docker-ce: {}
  docker-ce-cli: {}
  docker-ce-rootless-extras: {}
  docker-compose-plugin: {}
  docker-buildx-plugin: {}
  docker-model-plugin: {}
  containerd.io: {}
  # Google Chrome (Google's own apt repo)
  google-chrome-stable:
    apt: google-chrome-stable
  # Third-party binary downloads (verified by setmeup — see D2a)
  ffmpeg:
    from: download
  yt-dlp:
    from: download
    depends: [ffmpeg]
  # AWS CLI (official installer — checksum-pinned download)
  aws-cli:
    from: download
  # OpenCode — standalone binary
  opencode:
    from: download
  # Python tooling — uv (modern Python package manager, standalone)
  uv:
    from: download
  # Claude CLI (via uv or standalone)
  claude:
    from: download
  # Pip packages (requires pip, verified by pip's hash checking)
  csvkit:
    from: pip

# ── Runtime managers ───────────────────────────────
# NVM is installed via git clone (not curl-to-sh).
nvm:
  node-version: lts/*
  global-packages:
    - pnpm
    - bun

# Rust toolchain — is a special case: rustup manages itself
# and the toolchain. Setmeup installs rustup via the official
# script (verified download, not unchecked curl-to-sh — see D2a),
# then the toolchain follows.
rustup:
  toolchain: stable

# ── Repos (cloned to ~/repos/<name>/) ──────────────
repos:
  setmeup: "github.com/mdecurtins/setmeup"
  hoodhunter: "github.com/mdecurtins/hoodhunter"
  rustydrum: "github.com/mdecurtins/rustydrum"
  trendycrates: "github.com/mdecurtins/trendycrates"
  wp-ac: "github.com/mdecurtins/wp-ac"

# ── Aliases (written to managed file, sourced) ─────
aliases:
  hh: "cd ~/repos/hoodhunter"
  rustydrum: "cd ~/repos/rustydrum"
  trendycrates: "cd ~/repos/trendycrates"
  wpac: "cd ~/repos/wp-ac"
  storage: "du -sh .[^.]* * | sort -hr"
  lsmb: "ls -l --block-size=MB"
  dzi: "cd ~ && find . -name '*:Zone.Identifier' -type f -delete"

# ── Managed configuration files ────────────────────
configs:
  yt-dlp:
    path: "~/.config/yt-dlp/config"
    strategy: create-only
    content: |
      -o ~/downloads/%(title)s.%(ext)s

# ── Shell customization ────────────────────────────
shell:
  prompt: true
  functions:
    - name: parse_git_branch
      body: |
        git branch 2> /dev/null | sed -e "/^[^*]/d;s/* \(.*\)/(\1)/"
    - name: export2env
      body: |
        export $(cat $1 | xargs)
  source:
    - "~/.bash_aliases"
    - "~/.cargo/env"
    - "~/.local/bin/env"
  path-extend:
    - "~/.opencode/bin"
    - "~/.local/share/pnpm/bin"
    - "~/.lando/bin"
  env:
    BROWSER: /mnt/c/Program Files/Mozilla Firefox/firefox.exe
    OPENCODE_EXPERIMENTAL_BACKGROUND_SUBAGENTS: "true"
    PNPM_HOME: "~/.local/share/pnpm"
```

**Why:** Strong typing catches schema errors at compile time. Named sections are
self-documenting in YAML — a user can glance at the manifest and understand
what it configures. Per-capability validation is isolated in each struct's
`validate()` impl.

**Alternative considered:** Ordered task list with a `kind` discriminator
(more flexible, but loses top-level readability and compile-time section
discovery).

### D2. `ProvisionHandler` trait

**Decision:**

```rust
/// Result of an idempotency check before provisioning.
pub enum CheckResult {
    Satisfied,
    NeedsProvision,
    Divergent(String),  // exists but differs from declared state
    ManagedByUser,      // exists without setmeup marker
}

/// A provisioning capability (one section of the manifest).
pub trait ProvisionHandler: Debug {
    fn section_name(&self) -> &'static str;
    fn check(&self, os: Os, ctx: &ProvisionContext) -> Result<CheckResult>;
    fn provision(&self, os: Os, ctx: &ProvisionContext) -> Result<Vec<ProvisionResult>>;
}

/// Context passed to every handler.
pub struct ProvisionContext {
    pub repo_root: PathBuf,
    pub home: PathBuf,
    pub dry_run: bool,
}
```

The dispatch loop in `provision()`:
```rust
pub fn provision(os: Os, repo_root: &Path, manifest: &Manifest, ...) -> Result<Vec<ProvisionResult>> {
    let ctx = ProvisionContext { ... };
    let mut results = Vec::new();
    for handler in capability_handlers(manifest) {
        let check = handler.check(os, &ctx)?;
        match check {
            CheckResult::Satisfied => { /* report as satisfied */ }
            CheckResult::NeedsProvision => {
                results.extend(handler.provision(os, &ctx)?);
            }
            CheckResult::Divergent(detail) => { /* report divergence */ }
            CheckResult::ManagedByUser => { /* report skipped */ }
        }
    }
    Ok(results)
}
```

`capability_handlers()` returns a `Vec<Box<dyn ProvisionHandler>>` in declared
ordering order. Each handler is constructed from the manifest section it owns.

**Why:** Isolates per-capability logic into focused structs. The
check-before-act contract is enforced by the engine, not left to each handler.
Adding a capability = new struct + new handler impl. No changes to the dispatch
loop.

**Alternative considered:** Enum dispatch with a big match on capability
type — more tightly coupled but simpler. Rejected because the trait approach is
trivially testable per-handler and avoids the open-match problem.

### D2a. `from: download` trust boundary — never unchecked remote execution

**Decision:** When a package or capability declares `from: download`, the
implementation MUST verify the downloaded artifact before executing it.
Acceptable verification methods:

- **Git clone over curl-to-sh**: for tools like nvm, install via `git clone` of
  the canonical repository and source directly, rather than piping the install
  script. nvm's install script is a convenience wrapper around a git clone
  anyway.
- **Checksum pinning**: the handler embeds a known-good SHA-256 hash and
  verifies the download against it before use. Only use when git clone is not
  feasible.
- **Package manager delegation**: prefer native packages (e.g., `apt install
  nvm`) when available; only fall to `from: download` when no native package
  exists.

**Unacceptable:** Executing a remote URL via `curl | bash` or `curl | sh`
without verification. This is a trust-boundary violation — an attacker who
compromises the remote source gains execution on setmeup's authority.

**Why:** The MVP trust boundary (no plaintext fallback, no secret-in-git)
extends to installation sources. "Setmeup manages your environment" means it
manages the *verification* of what it installs, not just the install trigger.
Native package managers (apt, brew, winget) have their own signing chains and
are trusted — `from: download` is for tools not available through them, and
requires explicit verification.

### D3. No-clobber contract

**Decision:** Every handler MUST call `check()` before `provision()`. The
engine enforces this by construction — `provision()` calls `check()` and skips
provision on non-`NeedsProvision` results. Exceptions require explicit
`#[allow(clippy::unused)]` and documentation.

Per-config merge strategy:
- `create-only`: write if absent; skip if exists (safe default)
- `overwrite-managed`: write if absent OR if file contains the setmeup marker;
  skip if exists without marker. The marker is a comment line appropriate to
  the file format:
  - `# managed by setmeup` for shell scripts, YAML, Python, and other
    `#`-comment formats
  - `// managed by setmeup` for C-family/CSS/JS/TS
  - `; managed by setmeup` for INI-style
  - **For formats that do not support comments** (JSON, XML, binary), this
    strategy is invalid and MUST be rejected at manifest validation time.
    Only `create-only` is valid for those formats.
- `merge`: decode file, blend sections, flag conflicts (future, not in this
  change)

**Why:** Prevents the single most dangerous failure mode in a provisioning
tool: silently overwriting user state. The `check()` method is the idempotency
guarantee — it must tell the truth about whether the capability is satisfied.
False negatives (report "needs provision" when already satisfied) are a bug.

### D4. Linear crossterm wizard (ratatui removed)

**Decision:** The wizard is a sequence of 6 full-screen steps, each rendered
with crossterm (cursor control, color, input) and std I/O:

```
┌─────────────────────────────────────────────────┐
│  setmeup configure                            │
│                                                  │
│  [████████░░░░░░░░░░░░░░░░░░░░]  33%             │
│  Step 2 of 6: Packages                           │
│                                                  │
│  Which tools should be installed?                 │
│                                                  │
│  ✓ git        (apt: git)                         │
│  ✓ neovim     (apt: neovim)                      │
│  ☐ ffmpeg     (apt: ffmpeg)                      │
│  ☐ yt-dlp     (pip: yt-dlp)                     │
│                                                  │
│  [↑↓ navigate] [space toggle] [enter next]      │
│                                                  │
│  Package already present on this system: git     │
└─────────────────────────────────────────────────┘
```

Each step:
1. Clears screen
2. Draws progress bar + step number + title
3. Draws the step content (list, form, toggle)
4. Waits for input via crossterm's event polling
5. Validates and advances

The wizard SHALL handle SIGWINCH (terminal resize) via crossterm's event
stream so that rendering adjusts to the new terminal dimensions. The spinner
SHALL work correctly in high-latency SSH environments (non-blocking, async via
thread — already the pattern in `spin_until`).

Steps: Welcome → Credentials → Packages → Repos → Shell → Review

**Why:** Ratatui is overkill for a linear wizard — it adds compile time, a
complex event loop, and layout DSL. Crossterm + std I/O gives full control with
zero new dependencies. The progress bar and spinner patterns are already in the
codebase (`spin_until`, `prompt_masked`).

### D5. Agent skills per capability

**Decision:** Each manifest capability gets a skill at
`.opencode/skills/manifest-<name>/SKILL.md`. The skill template encodes:

```markdown
# Research
- Official docs URL
- Latest version/release source
- Installation methods and prerequisites

# Manifest schema
- YAML block structure
- Required vs optional fields
- Example

# Provisioning
- Install command template
- Config file location and format
- Verification command

# Idempotency
- How to check if already satisfied
- What constitutes "divergent" state

# Wizard
- What the TUI screen collects
- Defaults and validation
```

The manifest-packages skill covers how to research packages (apt search, brew
info, winget search, pip show). The manifest-nvm skill covers fetching the
latest nvm release tag from GitHub. Etc.

**Why:** The skills are the "extension mechanism" — they're not just
documentation, they're the instruction set for agents to add, update, or debug
a capability. Without them, adding a new capability requires guessing how the
ecosystem works. With them, the agent has a canonical reference.

### D7. Provisioning ordering

**Decision:** Fixed ordering in `provision()`, documented in the manifest spec.
Cross-capability dependency graphs (e.g., "clone this repo before installing
this package") are NOT supported. The `depends` field within `packages` works
only for ordering within that section.

```
1. packages           (system deps first: git, curl, ffmpeg)
2. nvm                (needs git/curl)
3. rustup             (needs curl, installs Rust toolchain)
4. repos              (needs git)
5. aliases            (references repo paths)
6. configs            (post-install config files)
7. shell              (functions, PS1, PATH, env, sources)
8. identity           (SSH/GPG — existing, retained)
9. secrets            (credential backend — existing, retained)
```

Dotfiles and OS preferences (existing) are slotted into ordering as they
conceptually align (dotfiles → shell step, OS prefs → after packages).

**Why:** Simple, predictable, good-enough for a personal tool. A dependency
graph resolver would be over-engineering for 6 capability types with no cycles.

**Accepted limitation:** If a future capability requires repos to be cloned
before a package can be installed from them, the architecture can accommodate
this by:
1. Reordering the fixed sequence (one-line change in `capability_handlers()`)
2. Each handler is independent — changing the dispatch order does not require
   changing handler code
3. The trait boundary ensures no capability needs to know about another

## Risks / Trade-offs

- **[Trait dispatch adds indirection over a single match]** → Mitigation: the
  handler vec is built once per `provision()` call by a simple factory function.
  Zero dynamic dispatch overhead beyond the one `Box<dyn>` indirection per
  handler.
- **[Skipping ratatui limits future TUI complexity]** → Mitigation: if the
  wizard ever needs complex layouts (tables, tabs, split panes), ratatui can be
  reintroduced as a step-level renderer without changing the step orchestration.
- **[Skills drift out of sync with code]** → Mitigation: PR reviews check that
  skill changes accompany capability changes. The done-gate for a new
  capability includes verifying the skill is up to date.
- **[create-only config strategy leaves stale configs]** → Acceptable: the user
  owns their config after initial creation. `status` can report divergent
  configs for awareness.
- **[`from: download` trust boundary]** → Addressed by D2a: verified downloads
  only (git clone, checksum pin, or native package fallback). Never
  `curl | bash`.
- **[Linear ordering insufficient for future cross-capability dependencies]** →
  Accepted. If a future capability needs repos-before-packages, the handler
  dispatch order is a one-line change. No handler code changes required.
- **[Config merge incompatible with non-comment formats]** → Addressed by D3:
  `overwrite-managed` and `merge` are invalid for JSON/XML/binary; validation
  rejects them at parse time. Only `create-only` is valid for those formats.
- **[SIGWINCH garbles TUI]** → Addressed by D4: the wizard handles terminal
  resize via crossterm event polling.
- **[Skill drift — skills out of sync with code]** → Mitigation: the done-gate
  for new capabilities includes verifying skill accuracy in review.

## Migration Plan

1. Define new manifest structs in `manifest.rs` (backward-compat: shim
  `tools` → `packages`)
2. Define `ProvisionHandler` trait + dispatch loop in `provisioning.rs`
3. Implement shell/aliases/configs handlers (trivial reorganization of existing
  patterns)
4. Refactor `tui.rs` into `tui/wizard.rs` + `tui/steps.rs` + `tui/dashboard.rs`
5. Implement linear wizard (crossterm, 6 steps, progress bar)
6. Remove ratatui, anyhow from Cargo.toml
7. Remove dead-code stubs from main.rs
8. Write `manifest-*` skills (one per capability)
9. Write `docs/ADDING-A-CAPABILITY.md`
10. Update specs (manifest, provisioning, tui, new agent-skills spec)
11. Done-gate: fmt, clippy, test, openspec validate

Rollback: revert manifest structs to V1 (keep backward-compat shim), revert
provisioning dispatch to inline match, restore tui stub.

## Open Questions

- Whether to keep the existing `tools` / `dotfiles` / `os` / `secrets` sections
  unchanged alongside new sections, or deprecate them immediately. Decision:
  retain for one release cycle with backward-compat shims, then remove.
- Whether the wizard should validate credentials during `configure` (requires
  live backend calls) or defer to `apply`. Decision (tentative): validate on
  step entry, show a spinner, but allow skip.
- Whether `shell.functions` writes a dedicated managed file or injects into
  `.bashrc` directly. Decision (tentative): dedicated file
  (`~/.config/setmeup/shell-functions.sh`), sourced by a single rc line.
  Cleaner idempotency and easier debugging.

## TODO (deferred)

- **Packages handler** with `from:` dispatch (pip, download) — needs package
  capability skill + implementation. Separate change after this foundation.
- **NVM handler** — needs nvm skill + implementation. Separate change.
- **Repos handler** — needs repos skill + implementation. Separate change.
- **Config merge (strategy: merge)** — future capability, not in this change.
- **Device-flow credential acquisition** (#19) — deferred, unchanged.
- **Full TUI wizard screens** — this change establishes the framework; content
  for each step is driven by the manifest sections present.
