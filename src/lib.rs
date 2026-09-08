//! setmeup — declarative machine provisioning.
//!
//! A personal tool: converge any fresh OS environment (Ubuntu/WSL2 by default,
//! also macOS and native Windows) to a declared desired state, idempotently.
//!
//! Trust boundary: this is a public repository. Secret material NEVER enters
//! git — it lives only in the vault (`secrets`) and non-git backup paths. The
//! manifest and dotfiles are public and must never contain secrets.

pub mod cli;
pub mod config;
pub mod error;
pub mod identity;
pub mod manifest;
pub mod provisioning;
pub mod secrets;
pub mod self_update;
pub mod tui;
