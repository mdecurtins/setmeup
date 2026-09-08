//! Setmeup CLI entry point.

use clap::{Parser, Subcommand};

/// setmeup — converge any new machine to your preferred environment.
#[derive(Debug, Parser)]
#[command(name = "setmeup", version, about)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Launch the TUI wizard to author the manifest.
    Configure,
    /// Reconcile the machine toward the manifest.
    Apply {
        /// Report intended actions without performing them.
        #[arg(long)]
        dry_run: bool,
        /// Machine-readable JSON output.
        #[arg(long)]
        json: bool,
    },
    /// Report which manifest items are satisfied/pending/failed.
    Status {
        #[arg(long)]
        json: bool,
    },
    /// Show the divergence between the machine and the manifest.
    Diff {
        #[arg(long)]
        json: bool,
    },
    /// Pull the latest source and rebuild setmeup itself.
    Update,
    /// Secret store operations.
    Secrets {
        #[command(subcommand)]
        action: SecretsAction,
    },
    /// Identity (SSH/GPG) key operations.
    Keys {
        #[command(subcommand)]
        action: KeysAction,
    },
}

#[derive(Debug, Subcommand)]
pub enum SecretsAction {
    /// List stored secret names (without values).
    List,
    /// Store a secret.
    Set {
        name: String,
        #[arg(long, hide = true)]
        value: Option<String>,
    },
    /// Get a secret's value.
    Get { name: String },
    /// Delete a secret.
    Delete { name: String },
    /// Migrate secrets between backends.
    Migrate { to: String },
}

#[derive(Debug, Subcommand)]
pub enum KeysAction {
    /// Generate or restore SSH/GPG identity keys and register public keys.
    Setup,
    /// List public keys registered with providers (via stored token).
    List,
    /// Revoke a registered public key.
    Revoke { provider: String, key_id: String },
}
