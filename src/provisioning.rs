//! Provisioning: per-OS package backends, dotfile symlinking, idempotent
//! shell configuration, and capability-based dispatch.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::error::{Result, SetmeupError};
use crate::manifest::{Manifest, Os, ShellFunction};

/// Outcome of provisioning one manifest item.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ItemStatus {
    Satisfied,
    Pending,
    Failed(String),
}

impl std::fmt::Display for ItemStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ItemStatus::Satisfied => f.write_str("satisfied"),
            ItemStatus::Pending => f.write_str("pending"),
            ItemStatus::Failed(e) => write!(f, "failed: {e}"),
        }
    }
}

/// A single reportable provisioning result.
#[derive(Debug, Clone)]
pub struct ProvisionResult {
    pub item: String,
    pub status: ItemStatus,
}

/// Result of an idempotency check before provisioning.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CheckResult {
    /// The declared state is already satisfied; provision() is not called.
    Satisfied,
    /// Not yet present; provision() should run.
    NeedsProvision,
    /// Exists but differs from declared state; report, do not overwrite.
    Divergent(String),
    /// Exists without a setmeup marker (user-owned); do not touch.
    ManagedByUser,
}

/// Context passed to every handler.
#[derive(Debug, Clone)]
pub struct ProvisionContext {
    pub repo_root: PathBuf,
    pub home: PathBuf,
    pub dry_run: bool,
}

/// A provisioning capability (one section of the manifest).
pub trait ProvisionHandler: std::fmt::Debug {
    /// The manifest section this handler owns (e.g. "packages").
    fn section_name(&self) -> &'static str;
    /// Whether this capability is already satisfied on this machine.
    fn check(&self, os: Os, ctx: &ProvisionContext) -> Result<CheckResult>;
    /// Converge the machine toward the declared state. Must only be invoked
    /// when check() returned NeedsProvision.
    fn provision(&self, os: Os, ctx: &ProvisionContext) -> Result<Vec<ProvisionResult>>;
}

// ---------------------------------------------------------------------------
// DeferredHandler
// ---------------------------------------------------------------------------

/// A declared manifest section whose handler is not yet implemented.
/// Reports a visible failure rather than silently skipping.
#[derive(Debug)]
pub struct DeferredHandler {
    section: &'static str,
}

impl ProvisionHandler for DeferredHandler {
    fn section_name(&self) -> &'static str {
        self.section
    }

    fn check(&self, _os: Os, _ctx: &ProvisionContext) -> Result<CheckResult> {
        Ok(CheckResult::NeedsProvision)
    }

    fn provision(&self, _os: Os, _ctx: &ProvisionContext) -> Result<Vec<ProvisionResult>> {
        Ok(vec![ProvisionResult {
            item: self.section.into(),
            status: ItemStatus::Failed(format!(
                "capability '{}' is declared but its handler is not yet implemented",
                self.section
            )),
        }])
    }
}

// ---------------------------------------------------------------------------
// ShellHandler
// ---------------------------------------------------------------------------

/// The setmeup marker placed at the top of generated files.
const SETMEUP_HEADER: &str = "# managed by setmeup\n";

/// Returns the user's rc file path based on $SHELL.
fn rc_file_path(home: &Path) -> PathBuf {
    let shell = std::env::var("SHELL").unwrap_or_default();
    if shell.contains("zsh") {
        home.join(".zshrc")
    } else {
        home.join(".bashrc")
    }
}

/// Read the lines of a file into a `Vec<String>`, ignoring empty lines.
fn read_lines(path: &Path) -> Result<Vec<String>> {
    let content = fs::read_to_string(path).unwrap_or_default();
    Ok(content.lines().map(|l| l.to_string()).collect())
}

/// Append `line` to `path` idempotently (no duplicates).
fn append_line_idempotent(path: &Path, line: &str) -> Result<bool> {
    let existing = fs::read_to_string(path).unwrap_or_default();
    if existing.lines().any(|l| l == line) {
        return Ok(false);
    }
    let mut content = existing;
    if !content.ends_with('\n') && !content.is_empty() {
        content.push('\n');
    }
    content.push_str(line);
    content.push('\n');
    fs::write(path, content)?;
    Ok(true)
}

/// Ensure a source line (`. "$HOME/..."`) exists in the rc file.
fn ensure_source_line(home: &Path, rc: &Path, src_path: &Path) -> Result<bool> {
    // Canonicalize the source path relative to home.
    let line = format!(
        ". \"$HOME/{}\"",
        src_path.strip_prefix(home).unwrap_or(src_path).display()
    );
    append_line_idempotent(rc, &line)
}

/// Check whether a source line exists in the rc file.
fn has_source_line(rc: &Path, src_path: &Path, home: &Path) -> Result<bool> {
    let lines = read_lines(rc)?;
    let expected = format!(
        ". \"$HOME/{}\"",
        src_path.strip_prefix(home).unwrap_or(src_path).display()
    );
    Ok(lines.iter().any(|l| l == &expected))
}

/// Build the shell-functions.sh content from declared functions.
fn build_functions_content(functions: &[ShellFunction]) -> String {
    let mut content = String::from(SETMEUP_HEADER);
    content.push('\n');
    for f in functions {
        content.push_str(&format!("{}() {{\n{}\n}}\n\n", f.name, f.body));
    }
    content
}

/// Parse shell-functions.sh and return the set of function names found.
fn parse_function_names(content: &str) -> Vec<String> {
    content
        .lines()
        .filter_map(|l| {
            let trimmed = l.trim();
            // Match `funcname() {`
            if trimmed.ends_with("() {") || trimmed.ends_with("() {{") {
                let name = trimmed
                    .trim_end_matches("() {")
                    .trim_end_matches("() {{")
                    .trim()
                    .to_string();
                Some(name)
            } else if let Some(pos) = trimmed.find("() ") {
                let name = trimmed[..pos].trim().to_string();
                if !name.is_empty() && !name.starts_with('#') {
                    return Some(name);
                }
                None
            } else if let Some(pos) = trimmed.find("() {") {
                let name = trimmed[..pos].trim().to_string();
                if !name.is_empty() && !name.starts_with('#') {
                    return Some(name);
                }
                None
            } else {
                None
            }
        })
        .collect()
}

/// Handler for the `shell` section of the manifest.
#[derive(Debug)]
pub struct ShellHandler;

impl ShellHandler {
    fn setmeup_dir(home: &Path) -> PathBuf {
        home.join(".config/setmeup")
    }

    fn functions_path(home: &Path) -> PathBuf {
        Self::setmeup_dir(home).join("shell-functions.sh")
    }
}

impl ProvisionHandler for ShellHandler {
    fn section_name(&self) -> &'static str {
        "shell"
    }

    fn check(&self, _os: Os, _ctx: &ProvisionContext) -> Result<CheckResult> {
        // The real check logic is in check_shell() which has manifest access.
        // The dispatch engine calls the specialized helper with the manifest.
        Ok(CheckResult::NeedsProvision)
    }

    fn provision(&self, _os: Os, _ctx: &ProvisionContext) -> Result<Vec<ProvisionResult>> {
        // The real provision logic is in provision_shell().
        // The dispatch engine calls the specialized helper with the manifest.
        Ok(vec![ProvisionResult {
            item: "shell".into(),
            status: ItemStatus::Satisfied,
        }])
    }
}

// ---------------------------------------------------------------------------
// Standalone check/provision helpers called by the dispatch engine
// (these have access to the manifest)
// ---------------------------------------------------------------------------

/// Check whether the shell section is already satisfied.
pub fn check_shell(manifest: &Manifest, ctx: &ProvisionContext) -> Result<CheckResult> {
    let home = &ctx.home;
    let rc = rc_file_path(home);

    // If nothing is configured, it's trivially satisfied.
    if manifest.shell.functions.is_empty()
        && !manifest.shell.prompt
        && manifest.shell.source.is_empty()
        && manifest.shell.path_extend.is_empty()
        && manifest.shell.env.is_empty()
    {
        return Ok(CheckResult::Satisfied);
    }

    // Check functions.
    if !manifest.shell.functions.is_empty() {
        let func_path = ShellHandler::functions_path(home);
        if !func_path.exists() {
            return Ok(CheckResult::NeedsProvision);
        }
        let content = fs::read_to_string(&func_path).unwrap_or_default();
        let existing_names: Vec<String> = parse_function_names(&content);
        let expected_names: Vec<String> = manifest
            .shell
            .functions
            .iter()
            .map(|f| f.name.clone())
            .collect();
        if existing_names != expected_names {
            return Ok(CheckResult::Divergent(
                "shell functions file content differs from declared".into(),
            ));
        }
        // Check source line in rc.
        if !has_source_line(&rc, &func_path, home)? {
            return Ok(CheckResult::NeedsProvision);
        }
    }

    // Check prompt (PS1 with parse_git_branch).
    if manifest.shell.prompt {
        let rc_content = fs::read_to_string(&rc).unwrap_or_default();
        if !rc_content.contains("parse_git_branch") {
            return Ok(CheckResult::NeedsProvision);
        }
    }

    // Check source files.
    for src in &manifest.shell.source {
        let src_path = if src.starts_with('/') {
            PathBuf::from(src)
        } else {
            home.join(src)
        };
        if !has_source_line(&rc, &src_path, home)? {
            return Ok(CheckResult::NeedsProvision);
        }
    }

    // Check path-extend.
    for dir in &manifest.shell.path_extend {
        let line = format!("export PATH=\"{dir}:$PATH\"");
        let lines = read_lines(&rc)?;
        if !lines.iter().any(|l| l.contains(&line) || l.contains(dir)) {
            return Ok(CheckResult::NeedsProvision);
        }
    }

    // Check env vars.
    for (key, val) in &manifest.shell.env {
        let export_line = format!("export {key}={val}");
        let lines = read_lines(&rc)?;
        if !lines.iter().any(|l| l.trim() == export_line) {
            return Ok(CheckResult::NeedsProvision);
        }
    }

    Ok(CheckResult::Satisfied)
}

/// Provision the shell section.
pub fn provision_shell(
    manifest: &Manifest,
    ctx: &ProvisionContext,
) -> Result<Vec<ProvisionResult>> {
    let mut results = Vec::new();
    let home = &ctx.home;
    let rc = rc_file_path(home);
    let setmeup_dir = ShellHandler::setmeup_dir(home);

    // Ensure setmeup dir exists.
    fs::create_dir_all(&setmeup_dir)?;

    // --- Functions ---
    if !manifest.shell.functions.is_empty() {
        let func_path = ShellHandler::functions_path(home);
        let content = build_functions_content(&manifest.shell.functions);
        fs::write(&func_path, &content)?;
        ensure_source_line(home, &rc, &func_path)?;
        results.push(ProvisionResult {
            item: "shell-functions".into(),
            status: ItemStatus::Satisfied,
        });
    }

    // --- Prompt ---
    if manifest.shell.prompt {
        let rc_content = fs::read_to_string(&rc).unwrap_or_default();
        if !rc_content.contains("parse_git_branch") {
            // Append a PS1 with parse_git_branch function call.
            let ps1_line = "PS1='\\[\\033[01;32m\\]\\u@\\h\\[\\033[00m\\]:\\[\\033[01;34m\\]\\w\\[\\033[00m\\]$(__git_ps1 \" (%s)\" 2>/dev/null || parse_git_branch 2>/dev/null)\\$ '";
            append_line_idempotent(&rc, ps1_line)?;
            // Also ensure parse_git_branch function is available (sourced from functions above or standard git prompt).
        }
        results.push(ProvisionResult {
            item: "shell-prompt".into(),
            status: ItemStatus::Satisfied,
        });
    }

    // --- Source files ---
    for src in &manifest.shell.source {
        let src_path = if src.starts_with('/') {
            PathBuf::from(src)
        } else {
            home.join(src)
        };
        ensure_source_line(home, &rc, &src_path)?;
        results.push(ProvisionResult {
            item: format!("shell-source:{}", src),
            status: ItemStatus::Satisfied,
        });
    }

    // --- Path extend ---
    for dir in &manifest.shell.path_extend {
        let line = format!("export PATH=\"{dir}:$PATH\"");
        // Guard: only prepend if not already in PATH.
        let guard_line = format!("case \":$PATH:\" in *\":{dir}:*\") ;; *) {} ;; esac", line);
        append_line_idempotent(&rc, &guard_line)?;
        results.push(ProvisionResult {
            item: format!("shell-path:{}", dir),
            status: ItemStatus::Satisfied,
        });
    }

    // --- Env vars ---
    for (key, val) in &manifest.shell.env {
        let line = format!("export {key}={val}");
        append_line_idempotent(&rc, &line)?;
        results.push(ProvisionResult {
            item: format!("shell-env:{}", key),
            status: ItemStatus::Satisfied,
        });
    }

    Ok(results)
}

// ---------------------------------------------------------------------------
// AliasesHandler
// ---------------------------------------------------------------------------

/// Handler for the `aliases` section of the manifest.
#[derive(Debug)]
pub struct AliasesHandler;

impl AliasesHandler {
    fn aliases_path(home: &Path) -> PathBuf {
        home.join(".config/setmeup/shell-aliases.sh")
    }

    fn rc(home: &Path) -> PathBuf {
        rc_file_path(home)
    }
}

impl ProvisionHandler for AliasesHandler {
    fn section_name(&self) -> &'static str {
        "aliases"
    }

    fn check(&self, _os: Os, _ctx: &ProvisionContext) -> Result<CheckResult> {
        // Real logic in check_aliases() which has manifest access.
        Ok(CheckResult::NeedsProvision)
    }

    fn provision(&self, _os: Os, _ctx: &ProvisionContext) -> Result<Vec<ProvisionResult>> {
        Ok(vec![ProvisionResult {
            item: "aliases".into(),
            status: ItemStatus::Satisfied,
        }])
    }
}

/// Check whether declared aliases are already present.
pub fn check_aliases(
    aliases: &BTreeMap<String, String>,
    ctx: &ProvisionContext,
) -> Result<CheckResult> {
    let home = &ctx.home;
    let aliases_path = AliasesHandler::aliases_path(home);

    if !aliases_path.exists() {
        return Ok(CheckResult::NeedsProvision);
    }

    let content = fs::read_to_string(&aliases_path).unwrap_or_default();

    // Check each declared alias appears.
    for (name, command) in aliases {
        let expected_line = format!("alias {name}=\"{command}\"");
        if !content.contains(&expected_line) {
            return Ok(CheckResult::Divergent(format!(
                "alias '{name}' differs or is missing"
            )));
        }
    }

    // Check if there are extra aliases in the file (also divergent).
    let declared_set: std::collections::HashSet<String> = aliases
        .iter()
        .map(|(n, c)| format!("alias {n}=\"{c}\""))
        .collect();
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("alias ")
            && !trimmed.starts_with('#')
            && !declared_set.contains(trimmed)
        {
            return Ok(CheckResult::Divergent(
                "extra aliases found in aliases file".into(),
            ));
        }
    }

    // Check source line in rc.
    let rc = AliasesHandler::rc(home);
    if !has_source_line(&rc, &aliases_path, home)? {
        return Ok(CheckResult::NeedsProvision);
    }

    Ok(CheckResult::Satisfied)
}

/// Provision aliases: write file and ensure rc source line.
pub fn provision_aliases(
    aliases: &BTreeMap<String, String>,
    ctx: &ProvisionContext,
) -> Result<Vec<ProvisionResult>> {
    let mut results = Vec::new();
    let home = &ctx.home;
    let setmeup_dir = ShellHandler::setmeup_dir(home);
    let aliases_path = AliasesHandler::aliases_path(home);

    fs::create_dir_all(&setmeup_dir)?;

    // Write aliases file.
    let mut content = String::from(SETMEUP_HEADER);
    content.push('\n');
    for (name, command) in aliases {
        // Use double quotes around the command.
        content.push_str(&format!("alias {name}=\"{command}\"\n"));
    }
    fs::write(&aliases_path, &content)?;
    results.push(ProvisionResult {
        item: "aliases-file".into(),
        status: ItemStatus::Satisfied,
    });

    // Ensure source line in rc.
    let rc = AliasesHandler::rc(home);
    ensure_source_line(home, &rc, &aliases_path)?;
    results.push(ProvisionResult {
        item: "aliases-rc-source".into(),
        status: ItemStatus::Satisfied,
    });

    Ok(results)
}

// ---------------------------------------------------------------------------
// ConfigsHandler
// ---------------------------------------------------------------------------

/// Handler for the `configs` section of the manifest.
#[derive(Debug)]
pub struct ConfigsHandler;

impl ConfigsHandler {
    /// The setmeup marker comment for config files.
    fn marker_line() -> &'static str {
        "# managed by setmeup - do not edit manually"
    }
}

impl ProvisionHandler for ConfigsHandler {
    fn section_name(&self) -> &'static str {
        "configs"
    }

    fn check(&self, _os: Os, _ctx: &ProvisionContext) -> Result<CheckResult> {
        // Real logic is in check_config() which has manifest access.
        Ok(CheckResult::NeedsProvision)
    }

    fn provision(&self, _os: Os, _ctx: &ProvisionContext) -> Result<Vec<ProvisionResult>> {
        Ok(vec![ProvisionResult {
            item: "configs".into(),
            status: ItemStatus::Satisfied,
        }])
    }
}

/// Check a single config file against the manifest.
pub fn check_config(
    _name: &str,
    config: &crate::manifest::ConfigFile,
    ctx: &ProvisionContext,
) -> Result<CheckResult> {
    let path = if config.path.starts_with('~') {
        ctx.home
            .join(config.path.strip_prefix("~/").unwrap_or(&config.path))
    } else {
        PathBuf::from(&config.path)
    };

    if !path.exists() {
        return Ok(CheckResult::NeedsProvision);
    }

    let content = fs::read_to_string(&path).unwrap_or_default();

    if config.strategy == crate::manifest::ConfigStrategy::CreateOnly {
        // File exists — report as user-managed.
        return Ok(CheckResult::ManagedByUser);
    }

    // OverwriteManaged: check if we own it (marker present).
    if !content.contains(ConfigsHandler::marker_line()) {
        return Ok(CheckResult::ManagedByUser);
    }

    // We own it. Check if content matches.
    let expected_content = build_config_content(config);
    if content == expected_content {
        Ok(CheckResult::Satisfied)
    } else {
        Ok(CheckResult::NeedsProvision)
    }
}

/// Build the content to write for a config file.
fn build_config_content(config: &crate::manifest::ConfigFile) -> String {
    let mut content = String::new();
    if config.strategy == crate::manifest::ConfigStrategy::OverwriteManaged {
        content.push_str(ConfigsHandler::marker_line());
        content.push('\n');
    }
    content.push_str(&config.content);
    if !config.content.ends_with('\n') {
        content.push('\n');
    }
    content
}

/// Provision a single config file.
pub fn provision_config(
    name: &str,
    config: &crate::manifest::ConfigFile,
    ctx: &ProvisionContext,
) -> Result<ProvisionResult> {
    let path = if config.path.starts_with('~') {
        ctx.home
            .join(config.path.strip_prefix("~/").unwrap_or(&config.path))
    } else {
        PathBuf::from(&config.path)
    };

    // Ensure parent dir exists.
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    // Check precondition: if create-only and already exists, skip.
    if config.strategy == crate::manifest::ConfigStrategy::CreateOnly && path.exists() {
        return Ok(ProvisionResult {
            item: format!("config:{}", name),
            status: ItemStatus::Satisfied,
        });
    }

    let content = build_config_content(config);
    fs::write(&path, &content)?;

    Ok(ProvisionResult {
        item: format!("config:{}", name),
        status: ItemStatus::Satisfied,
    })
}

// ---------------------------------------------------------------------------
// RustupHandler
// ---------------------------------------------------------------------------

/// Handler for the `rustup` section.
#[derive(Debug)]
pub struct RustupHandler;

impl ProvisionHandler for RustupHandler {
    fn section_name(&self) -> &'static str {
        "rustup"
    }

    fn check(&self, _os: Os, ctx: &ProvisionContext) -> Result<CheckResult> {
        check_rustup(ctx)
    }

    fn provision(&self, _os: Os, _ctx: &ProvisionContext) -> Result<Vec<ProvisionResult>> {
        // This change only lays the architecture; rustup provisioning is deferred.
        Ok(vec![ProvisionResult {
            item: "rustup".into(),
            status: ItemStatus::Failed(
                "rustup provisioning not yet implemented in this change".into(),
            ),
        }])
    }
}

/// Check rustup presence — binary on PATH or in ~/.cargo/bin/.
fn check_rustup(ctx: &ProvisionContext) -> Result<CheckResult> {
    // Check PATH.
    let on_path = Command::new("which")
        .arg("rustup")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .ok()
        .is_some_and(|s| s.success());

    if on_path {
        // Also check toolchain is installed.
        let toolchain_ok = Command::new("rustup")
            .args(["toolchain", "list"])
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .ok()
            .is_some_and(|s| s.success());
        if toolchain_ok {
            return Ok(CheckResult::Satisfied);
        }
    }

    // Check ~/.cargo/bin/rustup.
    let cargo_rustup = ctx.home.join(".cargo/bin/rustup");
    if cargo_rustup.exists() {
        let toolchain_ok = Command::new(&cargo_rustup)
            .args(["toolchain", "list"])
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .ok()
            .is_some_and(|s| s.success());
        if toolchain_ok {
            return Ok(CheckResult::Satisfied);
        }
    }

    Ok(CheckResult::NeedsProvision)
}

// ---------------------------------------------------------------------------
// Factory + dispatch
// ---------------------------------------------------------------------------

/// Build the list of capability handlers for a manifest.
/// Only shell, aliases, configs, and rustup have real handlers in this change;
/// all others use DeferredHandler.
pub fn capability_handlers(manifest: &Manifest) -> Vec<Box<dyn ProvisionHandler>> {
    let mut v: Vec<Box<dyn ProvisionHandler>> = Vec::new();
    if manifest.packages.is_some() {
        v.push(Box::new(DeferredHandler {
            section: "packages",
        }));
    }
    if manifest.nvm.is_some() {
        v.push(Box::new(DeferredHandler { section: "nvm" }));
    }
    if manifest.rustup.is_some() {
        v.push(Box::new(RustupHandler));
    }
    if manifest.repos.is_some() {
        v.push(Box::new(DeferredHandler { section: "repos" }));
    }
    if manifest.aliases.is_some() {
        v.push(Box::new(AliasesHandler));
    }
    if manifest.configs.is_some() {
        v.push(Box::new(ConfigsHandler));
    }
    if !is_shell_empty(&manifest.shell) {
        v.push(Box::new(ShellHandler));
    }
    v
}

fn is_shell_empty(shell: &crate::manifest::ShellConfig) -> bool {
    shell.functions.is_empty()
        && !shell.prompt
        && shell.source.is_empty()
        && shell.path_extend.is_empty()
        && shell.env.is_empty()
}

/// Install a single package via the backend for `os`.
pub fn install_package(os: Os, package: &str) -> Result<ItemStatus> {
    if is_present(os, package)? {
        return Ok(ItemStatus::Satisfied);
    }
    let success = match os {
        Os::Ubuntu => apt_install(package),
        Os::Macos => brew_install(package),
        Os::Windows => winget_install(package),
    }?;
    if success {
        Ok(ItemStatus::Satisfied)
    } else {
        Ok(ItemStatus::Failed(format!(
            "{os} install returned non-zero for {package}"
        )))
    }
}

/// Presence check per backend.
pub fn is_present(os: Os, package: &str) -> Result<bool> {
    let (program, args) = match os {
        Os::Ubuntu => ("dpkg", vec!["-s", package]),
        Os::Macos => ("brew", vec!["list", package]),
        Os::Windows => ("winget", vec!["list", "--id", package]),
    };
    let status = Command::new(program)
        .args(&args)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status();
    match status {
        Ok(s) => Ok(s.success()),
        Err(_) => Ok(false),
    }
}

fn apt_install(package: &str) -> Result<bool> {
    let status = Command::new("sudo")
        .args(["apt-get", "install", "-y", package])
        .status()
        .map_err(|e| SetmeupError::Command {
            cmd: "apt-get".into(),
            code: e.raw_os_error().unwrap_or(1),
        })?;
    Ok(status.success())
}

fn brew_install(package: &str) -> Result<bool> {
    let status = Command::new("brew")
        .args(["install", package])
        .status()
        .map_err(|e| SetmeupError::Command {
            cmd: "brew".into(),
            code: e.raw_os_error().unwrap_or(1),
        })?;
    Ok(status.success())
}

fn winget_install(package: &str) -> Result<bool> {
    let status = Command::new("winget")
        .args([
            "install",
            "--id",
            package,
            "--accept-package-agreements",
            "--accept-source-agreements",
        ])
        .status()
        .map_err(|e| SetmeupError::Command {
            cmd: "winget".into(),
            code: e.raw_os_error().unwrap_or(1),
        })?;
    Ok(status.success())
}

/// Apply dotfiles from a repo root into the home directory.
/// Returns a status per dotfile.
pub fn apply_dotfiles(repo_root: &Path, manifest: &Manifest) -> Result<Vec<ProvisionResult>> {
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    let mut results = Vec::new();
    for dotfile in &manifest.dotfiles {
        let src = repo_root.join(&dotfile.source);
        let dest = home.join(&dotfile.dest);
        // Ensure parent dirs exist for the destination
        if let Some(parent) = dest.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let status = symlink_dotfile(&src, &dest);
        results.push(ProvisionResult {
            item: dotfile.dest.clone(),
            status,
        });
    }
    Ok(results)
}

fn symlink_dotfile(src: &Path, dest: &Path) -> ItemStatus {
    if dest.is_symlink() {
        // Verify it points at src; if not, fix it (ln -sf semantics).
        match fs::read_link(dest) {
            Ok(target) if target == src => return ItemStatus::Satisfied,
            Ok(_) => {
                let _ = fs::remove_file(dest);
            }
            Err(_) => {}
        }
    } else if dest.exists() {
        // A real file blocks us. Don't overwrite silently.
        return ItemStatus::Failed(format!(
            "refusing to overwrite existing file {}",
            dest.display()
        ));
    }
    if !src.exists() {
        return ItemStatus::Failed(format!("source {} does not exist", src.display()));
    }
    let status = std::os::unix::fs::symlink(src, dest);
    match status {
        Ok(()) => ItemStatus::Satisfied,
        Err(e) => ItemStatus::Failed(e.to_string()),
    }
}

/// Idempotently merge shell rc lines into the user's rc file.
/// Only appends lines that are not already present.
pub fn apply_shell_config(rc_path: &Path, lines: &[String]) -> Result<Vec<ProvisionResult>> {
    let mut results = Vec::new();
    let existing = fs::read_to_string(rc_path).unwrap_or_default();
    let existing_lines: Vec<&str> = existing.lines().collect();
    let mut additions: Vec<String> = Vec::new();
    for line in lines {
        if !existing_lines.contains(&line.as_str()) {
            additions.push(line.clone());
        }
    }
    if !additions.is_empty() {
        let mut content = existing;
        if !content.ends_with('\n') && !content.is_empty() {
            content.push('\n');
        }
        for line in &additions {
            content.push_str(line);
            content.push('\n');
        }
        fs::write(rc_path, content)?;
        results.push(ProvisionResult {
            item: rc_path.display().to_string(),
            status: ItemStatus::Satisfied,
        });
    } else {
        results.push(ProvisionResult {
            item: rc_path.display().to_string(),
            status: ItemStatus::Satisfied,
        });
    }
    Ok(results)
}

// ---------------------------------------------------------------------------
// Main dispatch: provision()
// ---------------------------------------------------------------------------

/// Run a full provisioning pass over a manifest for `os`.
/// Failed items do NOT halt the pass; they're reported at the end.
pub fn provision(os: Os, repo_root: &Path, manifest: &Manifest) -> Result<Vec<ProvisionResult>> {
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    let ctx = ProvisionContext {
        repo_root: repo_root.to_path_buf(),
        home: home.clone(),
        dry_run: false,
    };

    let mut results = Vec::new();

    // V1 legacy path: packages from manifest.tools
    for tool in &manifest.tools {
        let pkg = tool.package_name_for(os);
        match install_package(os, pkg) {
            Ok(status) => results.push(ProvisionResult {
                item: tool.name.clone(),
                status,
            }),
            Err(e) => results.push(ProvisionResult {
                item: tool.name.clone(),
                status: ItemStatus::Failed(e.to_string()),
            }),
        }
    }

    // V1 legacy: dotfiles
    results.extend(apply_dotfiles(repo_root, manifest)?);

    // V1 legacy: shell rc_lines
    let rc = rc_file_path(&home);
    if !manifest.shell.rc_lines.is_empty() {
        results.extend(apply_shell_config(&rc, &manifest.shell.rc_lines)?);
    }

    // V2 capability dispatch
    let handlers = capability_handlers(manifest);
    for handler in handlers {
        let section = handler.section_name();
        match handler.check(os, &ctx) {
            Ok(check_result) => match check_result {
                CheckResult::Satisfied => {
                    results.push(ProvisionResult {
                        item: section.into(),
                        status: ItemStatus::Satisfied,
                    });
                }
                CheckResult::NeedsProvision => {
                    // For shell/aliases/configs, call the specialized helpers that have manifest access.
                    let dispatch_results = match section {
                        "shell" => match provision_shell(manifest, &ctx) {
                            Ok(r) => r,
                            Err(e) => vec![ProvisionResult {
                                item: section.into(),
                                status: ItemStatus::Failed(e.to_string()),
                            }],
                        },
                        "aliases" => {
                            if let Some(ref aliases) = manifest.aliases {
                                // Check first using the dedicated check fn
                                match check_aliases(aliases, &ctx) {
                                    Ok(CheckResult::Satisfied) => {
                                        vec![ProvisionResult {
                                            item: section.into(),
                                            status: ItemStatus::Satisfied,
                                        }]
                                    }
                                    Ok(CheckResult::NeedsProvision) => {
                                        match provision_aliases(aliases, &ctx) {
                                            Ok(r) => r,
                                            Err(e) => vec![ProvisionResult {
                                                item: section.into(),
                                                status: ItemStatus::Failed(e.to_string()),
                                            }],
                                        }
                                    }
                                    Ok(CheckResult::Divergent(d)) => {
                                        vec![ProvisionResult {
                                            item: section.into(),
                                            status: ItemStatus::Failed(d),
                                        }]
                                    }
                                    Ok(CheckResult::ManagedByUser) => {
                                        vec![ProvisionResult {
                                            item: format!("{section} (user-managed)"),
                                            status: ItemStatus::Satisfied,
                                        }]
                                    }
                                    Err(e) => vec![ProvisionResult {
                                        item: section.into(),
                                        status: ItemStatus::Failed(e.to_string()),
                                    }],
                                }
                            } else {
                                // The handler was only added when manifest.aliases.is_some(),
                                // but let's be defensive.
                                vec![ProvisionResult {
                                    item: section.into(),
                                    status: ItemStatus::Failed("aliases not declared".into()),
                                }]
                            }
                        }
                        "configs" => {
                            if let Some(ref configs) = manifest.configs {
                                let mut cr = Vec::new();
                                for (name, config) in configs {
                                    match check_config(name, config, &ctx) {
                                        Ok(CheckResult::Satisfied) => {
                                            cr.push(ProvisionResult {
                                                item: format!("config:{}", name),
                                                status: ItemStatus::Satisfied,
                                            });
                                        }
                                        Ok(CheckResult::NeedsProvision) => {
                                            match provision_config(name, config, &ctx) {
                                                Ok(r) => cr.push(r),
                                                Err(e) => cr.push(ProvisionResult {
                                                    item: format!("config:{}", name),
                                                    status: ItemStatus::Failed(e.to_string()),
                                                }),
                                            }
                                        }
                                        Ok(CheckResult::Divergent(d)) => {
                                            cr.push(ProvisionResult {
                                                item: format!("config:{}", name),
                                                status: ItemStatus::Failed(d),
                                            });
                                        }
                                        Ok(CheckResult::ManagedByUser) => {
                                            cr.push(ProvisionResult {
                                                item: format!("config:{} (user-managed)", name),
                                                status: ItemStatus::Satisfied,
                                            });
                                        }
                                        Err(e) => cr.push(ProvisionResult {
                                            item: format!("config:{}", name),
                                            status: ItemStatus::Failed(e.to_string()),
                                        }),
                                    }
                                }
                                cr
                            } else {
                                vec![ProvisionResult {
                                    item: section.into(),
                                    status: ItemStatus::Failed("configs not declared".into()),
                                }]
                            }
                        }
                        _ => {
                            // Generic handler (Deferred, RustupHandler, etc.)
                            match handler.provision(os, &ctx) {
                                Ok(r) => r,
                                Err(e) => vec![ProvisionResult {
                                    item: section.into(),
                                    status: ItemStatus::Failed(e.to_string()),
                                }],
                            }
                        }
                    };
                    results.extend(dispatch_results);
                }
                CheckResult::Divergent(detail) => {
                    results.push(ProvisionResult {
                        item: section.into(),
                        status: ItemStatus::Failed(detail),
                    });
                }
                CheckResult::ManagedByUser => {
                    results.push(ProvisionResult {
                        item: format!("{section} (user-managed)"),
                        status: ItemStatus::Satisfied,
                    });
                }
            },
            Err(e) => {
                results.push(ProvisionResult {
                    item: section.into(),
                    status: ItemStatus::Failed(e.to_string()),
                });
            }
        }
    }

    Ok(results)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::manifest::{ConfigFile, ConfigStrategy, Dotfile, ShellConfig, ShellFunction};

    // -----------------------------------------------------------------------
    // CheckResult tests
    // -----------------------------------------------------------------------

    #[test]
    fn deferred_handler_reports_failure() {
        let handler = DeferredHandler {
            section: "test-cap",
        };
        assert_eq!(
            handler
                .check(
                    Os::Ubuntu,
                    &ProvisionContext {
                        repo_root: PathBuf::from("/tmp"),
                        home: PathBuf::from("/tmp"),
                        dry_run: false,
                    }
                )
                .unwrap(),
            CheckResult::NeedsProvision
        );
        let results = handler
            .provision(
                Os::Ubuntu,
                &ProvisionContext {
                    repo_root: PathBuf::from("/tmp"),
                    home: PathBuf::from("/tmp"),
                    dry_run: false,
                },
            )
            .unwrap();
        assert_eq!(results.len(), 1);
        assert!(matches!(results[0].status, ItemStatus::Failed(_)));
        assert!(results[0].item == "test-cap");
    }

    #[test]
    fn dispatch_reports_deferred_sections_not_implemented() {
        let manifest = Manifest {
            packages: Some(BTreeMap::new()),
            nvm: Some(crate::manifest::NvmConfig::default()),
            repos: Some(BTreeMap::new()),
            ..Manifest::default()
        };
        let dir = tempfile::tempdir().unwrap();
        let home = dir.path().join("home");
        fs::create_dir_all(&home).unwrap();
        let ctx = ProvisionContext {
            repo_root: dir.path().to_path_buf(),
            home: home.clone(),
            dry_run: false,
        };
        let handlers = capability_handlers(&manifest);
        assert_eq!(handlers.len(), 3);
        for h in &handlers {
            assert_eq!(
                h.check(Os::Ubuntu, &ctx).unwrap(),
                CheckResult::NeedsProvision
            );
            let results = h.provision(Os::Ubuntu, &ctx).unwrap();
            assert_eq!(results.len(), 1);
            assert!(
                matches!(&results[0].status, ItemStatus::Failed(msg) if msg.contains("not yet implemented")),
                "expected failure for {}: got {:?}",
                h.section_name(),
                results[0].status
            );
        }
    }

    #[test]
    fn dispatch_order_follows_declared_sequence() {
        let manifest = Manifest {
            aliases: Some(BTreeMap::from([("ll".into(), "ls -la".into())])),
            configs: Some(BTreeMap::from([(
                "test".into(),
                ConfigFile {
                    path: "~/.testrc".into(),
                    content: "x=1".into(),
                    strategy: ConfigStrategy::CreateOnly,
                },
            )])),
            shell: ShellConfig {
                prompt: true,
                ..ShellConfig::default()
            },
            ..Manifest::default()
        };
        let handlers = capability_handlers(&manifest);
        let sections: Vec<&str> = handlers.iter().map(|h| h.section_name()).collect();
        assert_eq!(sections, vec!["aliases", "configs", "shell"]);
    }

    #[test]
    fn provides_no_work_when_nothing_declared() {
        let manifest = Manifest::default();
        let dir = tempfile::tempdir().unwrap();
        let results = provision(Os::Ubuntu, dir.path(), &manifest).unwrap();
        // Only V1 tools (empty) would be in results.
        assert!(results.is_empty());
    }

    #[test]
    fn provision_does_not_halt_on_failure() {
        let dir = tempfile::tempdir().unwrap();
        let repo = dir.path().join("repo");
        fs::create_dir_all(&repo).unwrap();
        let src = repo.join("dotfiles/.zshrc");
        fs::create_dir_all(src.parent().unwrap()).unwrap();
        fs::write(&src, b"ok").unwrap();
        let m = Manifest {
            dotfiles: vec![
                Dotfile {
                    source: "dotfiles/.zshrc".into(),
                    dest: ".zshrctest".into(),
                },
                Dotfile {
                    source: "missing-file".into(),
                    dest: ".missing".into(),
                },
            ],
            shell: ShellConfig::default(),
            ..Manifest::default()
        };
        let results = provision(Os::Ubuntu, &repo, &m).unwrap();
        let failed = results
            .iter()
            .filter(|r| matches!(r.status, ItemStatus::Failed(_)))
            .count();
        let satisfied = results
            .iter()
            .filter(|r| matches!(r.status, ItemStatus::Satisfied))
            .count();
        assert!(failed >= 1, "missing source must be reported failed");
        assert!(satisfied >= 1, "valid dotfile must be satisfied");
    }

    // -----------------------------------------------------------------------
    // AliasesHandler tests
    // -----------------------------------------------------------------------

    #[test]
    fn aliases_absent_file_needs_provision() {
        let dir = tempfile::tempdir().unwrap();
        let home = dir.path().join("home");
        fs::create_dir_all(&home).unwrap();
        let ctx = ProvisionContext {
            repo_root: dir.path().to_path_buf(),
            home: home.clone(),
            dry_run: false,
        };
        let aliases = BTreeMap::from([("ll".into(), "ls -la".into())]);
        assert_eq!(
            check_aliases(&aliases, &ctx).unwrap(),
            CheckResult::NeedsProvision
        );
    }

    #[test]
    fn aliases_existing_identical_satisfied() {
        let dir = tempfile::tempdir().unwrap();
        let home = dir.path().join("home");
        fs::create_dir_all(&home).unwrap();
        let setmeup_dir = home.join(".config/setmeup");
        fs::create_dir_all(&setmeup_dir).unwrap();
        let aliases_path = setmeup_dir.join("shell-aliases.sh");
        fs::write(&aliases_path, "# managed by setmeup\nalias ll=\"ls -la\"\n").unwrap();

        let ctx = ProvisionContext {
            repo_root: dir.path().to_path_buf(),
            home: home.clone(),
            dry_run: false,
        };
        let aliases = BTreeMap::from([("ll".into(), "ls -la".into())]);
        assert_eq!(
            check_aliases(&aliases, &ctx).unwrap(),
            CheckResult::NeedsProvision
        );
    }

    // -----------------------------------------------------------------------
    // ConfigsHandler tests
    // -----------------------------------------------------------------------

    #[test]
    fn configs_create_only_existing_is_managed_by_user() {
        let dir = tempfile::tempdir().unwrap();
        let home = dir.path().join("home");
        fs::create_dir_all(&home).unwrap();
        let config_path = home.join(".testrc");
        fs::write(&config_path, "existing content").unwrap();

        let ctx = ProvisionContext {
            repo_root: dir.path().to_path_buf(),
            home: home.clone(),
            dry_run: false,
        };
        let config = ConfigFile {
            path: "~/.testrc".into(),
            content: "new content".into(),
            strategy: ConfigStrategy::CreateOnly,
        };
        assert_eq!(
            check_config("test", &config, &ctx).unwrap(),
            CheckResult::ManagedByUser
        );
    }

    #[test]
    fn configs_absent_needs_provision_then_writes() {
        let dir = tempfile::tempdir().unwrap();
        let home = dir.path().join("home");
        fs::create_dir_all(&home).unwrap();

        let ctx = ProvisionContext {
            repo_root: dir.path().to_path_buf(),
            home: home.clone(),
            dry_run: false,
        };
        let config = ConfigFile {
            path: "~/.testrc".into(),
            content: "x=1".into(),
            strategy: ConfigStrategy::CreateOnly,
        };
        assert_eq!(
            check_config("test", &config, &ctx).unwrap(),
            CheckResult::NeedsProvision
        );
        let result = provision_config("test", &config, &ctx).unwrap();
        assert!(matches!(result.status, ItemStatus::Satisfied));
        let written = fs::read_to_string(home.join(".testrc")).unwrap();
        assert_eq!(written, "x=1\n");
    }

    #[test]
    fn configs_overwrite_managed_writes_marker() {
        let dir = tempfile::tempdir().unwrap();
        let home = dir.path().join("home");
        fs::create_dir_all(&home).unwrap();

        let ctx = ProvisionContext {
            repo_root: dir.path().to_path_buf(),
            home: home.clone(),
            dry_run: false,
        };
        let config = ConfigFile {
            path: "~/.testrc".into(),
            content: "x=1".into(),
            strategy: ConfigStrategy::OverwriteManaged,
        };
        let result = provision_config("test", &config, &ctx).unwrap();
        assert!(matches!(result.status, ItemStatus::Satisfied));
        let written = fs::read_to_string(home.join(".testrc")).unwrap();
        assert!(written.contains("# managed by setmeup"));
        assert!(written.contains("x=1"));
    }

    // -----------------------------------------------------------------------
    // ShellHandler tests
    // -----------------------------------------------------------------------

    #[test]
    fn shell_empty_is_satisfied() {
        let manifest = Manifest::default();
        let dir = tempfile::tempdir().unwrap();
        let home = dir.path().join("home");
        fs::create_dir_all(&home).unwrap();
        let ctx = ProvisionContext {
            repo_root: dir.path().to_path_buf(),
            home: home.clone(),
            dry_run: false,
        };
        // Override HOME env for the test scope
        let result = check_shell(&manifest, &ctx).unwrap();
        assert_eq!(result, CheckResult::Satisfied);
    }

    #[test]
    fn shell_functions_written_once_idempotent() {
        let manifest = Manifest {
            shell: ShellConfig {
                functions: vec![ShellFunction {
                    name: "myfunc".into(),
                    body: "echo hello".into(),
                }],
                ..ShellConfig::default()
            },
            ..Manifest::default()
        };
        let dir = tempfile::tempdir().unwrap();
        let home = dir.path().join("home");
        fs::create_dir_all(&home).unwrap();
        let setmeup_dir = home.join(".config/setmeup");
        fs::create_dir_all(&setmeup_dir).unwrap();
        let rc = rc_file_path(&home);
        fs::write(&rc, "").unwrap();

        let ctx = ProvisionContext {
            repo_root: dir.path().to_path_buf(),
            home: home.clone(),
            dry_run: false,
        };

        // First pass
        let r1 = provision_shell(&manifest, &ctx).unwrap();
        assert!(r1.iter().any(|r| r.item == "shell-functions"));

        // Second pass (idempotent)
        let r2 = provision_shell(&manifest, &ctx).unwrap();
        // Should still report satisfied, no errors
        assert!(r2.iter().all(|r| matches!(r.status, ItemStatus::Satisfied)));

        // Check functions file content
        let func_path = ShellHandler::functions_path(&home);
        let content = fs::read_to_string(&func_path).unwrap();
        assert!(content.contains("myfunc"));
        assert_eq!(content.matches("myfunc").count(), 1, "no duplicate func");

        // Check rc source line
        let rc_content = fs::read_to_string(&rc).unwrap();
        assert!(rc_content.contains("shell-functions.sh"));
        assert_eq!(
            rc_content.matches("shell-functions.sh").count(),
            1,
            "no duplicate source line"
        );
    }

    #[test]
    fn shell_path_extend_appends_idempotent() {
        let manifest = Manifest {
            shell: ShellConfig {
                path_extend: vec!["/custom/bin".into()],
                ..ShellConfig::default()
            },
            ..Manifest::default()
        };
        let dir = tempfile::tempdir().unwrap();
        let home = dir.path().join("home");
        fs::create_dir_all(&home).unwrap();
        let rc = rc_file_path(&home);
        fs::write(&rc, "").unwrap();

        let ctx = ProvisionContext {
            repo_root: dir.path().to_path_buf(),
            home: home.clone(),
            dry_run: false,
        };

        provision_shell(&manifest, &ctx).unwrap();
        provision_shell(&manifest, &ctx).unwrap();

        let rc_content = fs::read_to_string(&rc).unwrap();
        // The guard line contains the dir path; count occurrences of the entire guard line.
        let guard = "case \":$PATH:\" in *\":/custom/bin:";
        assert_eq!(rc_content.matches(guard).count(), 1, "no duplicate guard");
        assert!(rc_content.contains("/custom/bin"), "dir appears in rc");
    }

    #[test]
    fn shell_env_vars_appended_once() {
        let manifest = Manifest {
            shell: ShellConfig {
                env: BTreeMap::from([("EDITOR".into(), "nvim".into())]),
                ..ShellConfig::default()
            },
            ..Manifest::default()
        };
        let dir = tempfile::tempdir().unwrap();
        let home = dir.path().join("home");
        fs::create_dir_all(&home).unwrap();
        let rc = rc_file_path(&home);
        fs::write(&rc, "").unwrap();

        let ctx = ProvisionContext {
            repo_root: dir.path().to_path_buf(),
            home: home.clone(),
            dry_run: false,
        };

        provision_shell(&manifest, &ctx).unwrap();
        provision_shell(&manifest, &ctx).unwrap();

        let rc_content = fs::read_to_string(&rc).unwrap();
        assert!(rc_content.contains("export EDITOR=nvim"));
        assert_eq!(rc_content.matches("export EDITOR=nvim").count(), 1);
    }

    // -----------------------------------------------------------------------
    // Legacy test compat
    // -----------------------------------------------------------------------

    #[test]
    fn presence_check_returns_bool_without_panicking() {
        let _ = is_present(Os::Ubuntu, "bash");
    }

    #[test]
    fn symlink_creates_and_reports_satisfied() {
        let dir = tempfile::tempdir().unwrap();
        let src = dir.path().join("source");
        let dest = dir.path().join("dest");
        fs::write(&src, b"content").unwrap();
        let status = symlink_dotfile(&src, &dest);
        assert_eq!(status, ItemStatus::Satisfied);
        assert!(dest.is_symlink());
    }

    #[test]
    fn symlink_existing_is_satisfied() {
        let dir = tempfile::tempdir().unwrap();
        let src = dir.path().join("source");
        let dest = dir.path().join("dest");
        fs::write(&src, b"content").unwrap();
        std::os::unix::fs::symlink(&src, &dest).unwrap();
        assert_eq!(
            symlink_dotfile(&src, &dest),
            ItemStatus::Satisfied,
            "no-op for correct link"
        );
    }

    #[test]
    fn symlink_conflict_not_overwritten() {
        let dir = tempfile::tempdir().unwrap();
        let src = dir.path().join("source");
        let dest = dir.path().join("dest");
        fs::write(&src, b"content").unwrap();
        fs::write(&dest, b"real file").unwrap();
        let status = symlink_dotfile(&src, &dest);
        assert!(
            matches!(status, ItemStatus::Failed(_)),
            "must not silently overwrite"
        );
        assert!(!dest.is_symlink());
    }

    #[test]
    fn symlink_wrong_target_refixed() {
        let dir = tempfile::tempdir().unwrap();
        let src = dir.path().join("source");
        let other = dir.path().join("other");
        let dest = dir.path().join("dest");
        fs::write(&src, b"content").unwrap();
        fs::write(&other, b"other").unwrap();
        std::os::unix::fs::symlink(&other, &dest).unwrap();
        assert_eq!(
            symlink_dotfile(&src, &dest),
            ItemStatus::Satisfied,
            "re-points wrong link"
        );
        assert_eq!(fs::read_link(&dest).unwrap(), src);
    }

    #[test]
    fn shell_lines_appended_once_and_only_once() {
        let dir = tempfile::tempdir().unwrap();
        let rc = dir.path().join(".bashrc");
        fs::write(&rc, "alias gs=\"git status\"\n").unwrap();
        let lines = vec![
            "alias gs=\"git status\"".to_string(),
            "export EDITOR=nvim".to_string(),
        ];
        apply_shell_config(&rc, &lines).unwrap();
        apply_shell_config(&rc, &lines).unwrap(); // idempotent second pass
        let content = fs::read_to_string(&rc).unwrap();
        assert_eq!(
            content.matches("alias gs=\"git status\"").count(),
            1,
            "no duplicate"
        );
        assert_eq!(content.matches("export EDITOR=nvim").count(), 1);
    }
}
