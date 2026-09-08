//! setmeup executable entry point.

use std::io::{IsTerminal, Write};
use std::path::Path;
use std::sync::mpsc;

use clap::Parser;

use setmeup::cli::{Cli, Command, KeysAction, SecretsAction};
use setmeup::config;
use setmeup::error::{Result, SetmeupError};
use setmeup::identity;
use setmeup::manifest::{Manifest, Os, SecretAcquire, SecretBackend as ManifestSecretBackend};
use setmeup::provisioning;
use setmeup::secrets::{
    self, BackendKind, EnvPassphraseProvider, PassphraseProvider, SecretBackend,
};
use setmeup::self_update;
use setmeup::tui;

fn main() {
    let cli = Cli::parse();
    let exit = match run(cli) {
        Ok(code) => code,
        Err(SetmeupError::Manifest(msg)) => {
            eprintln!("error: {msg}");
            2
        }
        Err(e) => {
            eprintln!("error: {e}");
            1
        }
    };
    std::process::exit(exit);
}

fn run(cli: Cli) -> Result<i32> {
    match cli.command {
        Command::Configure => cmd_configure(),
        Command::Apply { dry_run, json } => cmd_apply(dry_run, json),
        Command::Status { json } => cmd_status(json),
        Command::Diff { json } => cmd_diff(json),
        Command::Update => cmd_update(),
        Command::Secrets { action } => cmd_secrets(action),
        Command::Keys { action } => cmd_keys(action),
    }
}

// ---------------------------------------------------------------------------
// configure
// ---------------------------------------------------------------------------

fn cmd_configure() -> Result<i32> {
    let existing = Manifest::load(None).ok();
    let manifest = tui::run_wizard(existing.as_ref())?;
    tui::write_manifest(&manifest, None)?;
    println!("manifest written to {}", config::manifest_path().display());
    Ok(0)
}

// ---------------------------------------------------------------------------
// apply
// ---------------------------------------------------------------------------

fn cmd_apply(dry_run: bool, json: bool) -> Result<i32> {
    let manifest = Manifest::load(None)?;
    let os = Os::current();
    let repo_root = std::env::current_dir()?;

    if dry_run {
        // Report intended actions without performing.
        let resolved = manifest.resolve_for(os);
        let mut lines = Vec::new();
        for (name, pkg) in &resolved.packages {
            let present = provisioning::is_present(os, pkg)?;
            lines.push(format!(
                "would {} {name} ({}): {}",
                if present { "verify" } else { "install" },
                pkg,
                if present {
                    "already present"
                } else {
                    "would install"
                }
            ));
        }
        for d in &resolved.dotfiles {
            lines.push(format!("would link dotfile {} -> {}", d.source, d.dest));
        }
        for line in &resolved.shell.rc_lines {
            lines.push(format!("would add shell line: {line}"));
        }
        if lines.is_empty() {
            println!("no changes would be made; machine already converged");
        }
        if json {
            let report = serde_json::json!({
                "dry_run": true,
                "actions": lines,
            });
            println!("{report}");
        } else {
            for l in lines {
                println!("{l}");
            }
        }
        return Ok(0);
    }

    // Live run: stream events to the dashboard.
    let (tx, rx) = mpsc::channel::<tui::DashboardEvent>();
    let total_guess = manifest.tools.len()
        + manifest.dotfiles.len()
        + if manifest.shell.rc_lines.is_empty() {
            0
        } else {
            1
        };

    // Run provisioning synchronously, feeding the channel.
    let results = provisioning::provision(os, &repo_root, &manifest, None)?;
    let mut has_failure = false;
    for r in &results {
        match &r.status {
            provisioning::ItemStatus::Satisfied => {
                tx.send(tui::DashboardEvent::Succeeded(r.item.clone()))
            }
            provisioning::ItemStatus::Pending => {
                tx.send(tui::DashboardEvent::Started(r.item.clone()))
            }
            provisioning::ItemStatus::Failed(detail) => {
                has_failure = true;
                tx.send(tui::DashboardEvent::Failed(r.item.clone(), detail.clone()))
            }
        }
        .map_err(|e| SetmeupError::Provisioning(e.to_string()))?;
    }
    tx.send(tui::DashboardEvent::Finished)
        .map_err(|e| SetmeupError::Provisioning(e.to_string()))?;
    let summary = tui::render_apply_dashboard(rx, total_guess)?;

    if json {
        let report = serde_json::json!({
            "succeeded": summary.succeeded,
            "failed": summary.failed,
            "skipped": summary.skipped,
            "converged": !has_failure,
            "final_status": if has_failure { "failed" } else { "satisfied" },
        });
        println!("{report}");
    }

    Ok(if has_failure { 1 } else { 0 })
}

// ---------------------------------------------------------------------------
// status / diff
// ---------------------------------------------------------------------------

fn cmd_status(json: bool) -> Result<i32> {
    if !Path::new(&config::manifest_path()).exists() {
        return Err(SetmeupError::Manifest(format!(
            "no manifest at {}; run configure or specify one",
            config::manifest_path().display()
        )));
    }
    let manifest = Manifest::load(None)?;
    let os = Os::current();
    let resolved = manifest.resolve_for(os);
    // For status we focus on package presence (cheap and truthful).
    let mut rows = Vec::new();
    for (name, pkg) in &resolved.packages {
        let present = provisioning::is_present(os, pkg)?;
        let state = if present { "satisfied" } else { "pending" };
        rows.push(serde_json::json!({ "item": name, "package": pkg, "state": state }));
    }
    // Dotfiles: report based on symlink existence is best-effort.
    let home = dirs::home_dir().unwrap_or_else(|| std::path::PathBuf::from("."));
    for d in &resolved.dotfiles {
        let dest = home.join(&d.dest);
        let state = if dest.exists() || dest.is_symlink() {
            "satisfied"
        } else {
            "pending"
        };
        rows.push(serde_json::json!({ "item": format!("dotfile:{}", d.dest), "state": state }));
    }
    if json {
        println!("{}", serde_json::json!({ "items": rows }));
    } else {
        for r in &rows {
            println!("{}: {}", r["item"], r["state"]);
        }
    }
    Ok(0)
}

fn cmd_diff(json: bool) -> Result<i32> {
    if !Path::new(&config::manifest_path()).exists() {
        return Err(SetmeupError::Manifest(format!(
            "no manifest at {}; run configure or specify one",
            config::manifest_path().display()
        )));
    }
    let manifest = Manifest::load(None)?;
    let os = Os::current();
    let resolved = manifest.resolve_for(os);
    let mut divergent = Vec::new();
    for (name, pkg) in &resolved.packages {
        let present = provisioning::is_present(os, pkg)?;
        if !present {
            divergent
                .push(serde_json::json!({ "item": name, "package": pkg, "action": "install" }));
        }
    }
    if json {
        // `diff` performs NO mutation by definition.
        println!(
            "{}",
            serde_json::json!({ "divergent": divergent, "note": "no changes applied" })
        );
    } else {
        if divergent.is_empty() {
            println!("no differences; machine matches manifest");
        } else {
            for d in &divergent {
                println!("{}: needs {}", d["item"], d["action"]);
            }
        }
    }
    Ok(0)
}

// ---------------------------------------------------------------------------
// update
// ---------------------------------------------------------------------------

fn cmd_update() -> Result<i32> {
    let report = self_update::update()?;
    println!(
        "setmeup updated (rebuild={}{})",
        report.rebuild_needed,
        report
            .binary_path
            .map(|p| format!(" at {}", p.display()))
            .unwrap_or_default()
    );
    Ok(0)
}

// ---------------------------------------------------------------------------
// secrets
// ---------------------------------------------------------------------------

fn cmd_secrets(action: SecretsAction) -> Result<i32> {
    match action {
        SecretsAction::List => {
            let backend = default_secret_backend()?;
            let entries = backend.list()?;
            for e in &entries {
                println!("{}", e.name);
            }
            let _ = backend;
            Ok(0)
        }
        SecretsAction::Set { name, value } => {
            let mut backend = default_secret_backend()?;
            let value = match value {
                Some(v) => v,
                None => tui::prompt_masked(&format!("enter value for '{name}': "))?,
            };
            backend.set(&name, &value)?;
            println!("stored {name}");
            Ok(0)
        }
        SecretsAction::Get { name } => {
            let backend = default_secret_backend()?;
            let v = backend.get(&name)?;
            println!("{v}");
            Ok(0)
        }
        SecretsAction::Delete { name } => {
            let mut backend = default_secret_backend()?;
            backend.delete(&name)?;
            println!("deleted {name}");
            Ok(0)
        }
        SecretsAction::Migrate { to } => {
            let to_kind = match to.as_str() {
                "vault-file" => BackendKind::VaultFile,
                "keyring" => BackendKind::Keyring,
                other => {
                    return Err(SetmeupError::Secret(format!(
                        "unknown backend '{other}' (expected 'vault-file' or 'keyring')"
                    )));
                }
            };
            let mut from = default_secret_backend()?;
            let mut to_backend = secrets::select_backend(to_kind, None)?;
            let count = secrets::migrate(from.as_mut(), to_backend.as_mut())?;
            println!("migrated {count} secrets to {to_kind}");
            Ok(0)
        }
    }
}

fn default_secret_backend() -> Result<Box<dyn SecretBackend>> {
    secrets::select_backend(BackendKind::VaultFile, None)
}

// ---------------------------------------------------------------------------
// keys
// ---------------------------------------------------------------------------

fn cmd_keys(action: KeysAction) -> Result<i32> {
    match action {
        KeysAction::Setup => {
            // Restore-before-generate.
            if identity::ssh_key_exists()? {
                println!("SSH key already exists; nothing to do");
                return Ok(0);
            }
            // Try restore from Windows host (WSL2).
            let backup = config::windows_host_backup_root();
            match backup {
                Some(root) if root.join("ssh-keys.age").exists() => {
                    let passphrase = secret_passphrase()?;
                    identity::restore_ssh_from_windows(&passphrase)?;
                    println!("restored SSH keys from Windows-host backup");
                }
                _ => {
                    // Generate fresh keys.
                    identity::generate_ssh_key()?;
                    println!("generated fresh ed25519 SSH key");
                    // Backup to Windows host if possible.
                    if let Some(_root) = backup {
                        let passphrase = secret_passphrase()?;
                        let target = identity::backup_ssh_to_windows(&passphrase)?;
                        println!("backed up SSH keys to {}", target.display());
                    }
                }
            }
            // Optionally register with providers if a cert is available.
            if let Some(pubkey) = identity::public_key_text()? {
                let token_opt = try_github_token();
                if let Some(token) = token_opt {
                    match identity::register_ssh_key("github", &token, &pubkey) {
                        Ok(newly) => println!(
                            "github ssh key {}",
                            if newly {
                                "registered"
                            } else {
                                "already present (skipped)"
                            }
                        ),
                        Err(e) => eprintln!("warning: ssh registration failed: {e}"),
                    }
                }
            }
            Ok(0)
        }
        KeysAction::List => {
            if let Some(pubkey) = identity::public_key_text()? {
                println!("{pubkey}");
                Ok(0)
            } else {
                println!("no ssh public key present");
                Ok(0)
            }
        }
        KeysAction::Revoke { .. } => {
            // Revocation requires provider API call with a stored token.
            Err(SetmeupError::Identity(
                "revoke is not yet wired to a provider CLI; use the provider site directly".into(),
            ))
        }
    }
}

fn secret_passphrase() -> Result<age::secrecy::SecretString> {
    // Reuse EnvPassphraseProvider; the TUI path would prompt.
    let provider = EnvPassphraseProvider;
    provider.passphrase()
}

fn try_github_token() -> Option<String> {
    let backend = default_secret_backend().ok()?;
    backend.get("github_token").ok()
}

// Re-export guard so the manifest backends enum is referenced (keeps imports honest).
#[allow(dead_code)]
fn _manifest_backend_to_secrets(b: ManifestSecretBackend) -> BackendKind {
    match b {
        ManifestSecretBackend::VaultFile => BackendKind::VaultFile,
        ManifestSecretBackend::Keyring => BackendKind::Keyring,
    }
}

#[allow(dead_code)]
fn _secret_acquire_to_hint(a: SecretAcquire) -> &'static str {
    match a {
        SecretAcquire::Paste => "paste",
        SecretAcquire::DeviceFlow => "device-flow",
        SecretAcquire::Env => "env",
    }
}

// ---------------------------------------------------------------------------
// helpers
// ---------------------------------------------------------------------------

#[allow(dead_code)]
fn _is_tty() -> bool {
    std::io::stdin().is_terminal()
}

#[allow(dead_code)]
fn _flush() -> Result<()> {
    std::io::stdout().flush().map_err(SetmeupError::Io)
}
