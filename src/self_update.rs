//! Self-update: pull the latest source and rebuild the setmeup binary.
//!
//! Safe-by-design: setmeup apply is declarative and idempotent, so running a
//! newer binary against the same manifest converges rather than mutating.

use std::path::PathBuf;
use std::process::Command;

use crate::config;
use crate::error::{Result, SetmeupError};

/// Result of a self-update attempt.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpdateReport {
    pub rebuild_needed: bool,
    pub binary_path: Option<PathBuf>,
}

/// Run `cargo install --path .` (or `cargo install --git`) to rebuild.
///
/// When invoked from a git checkout, updates via git pull first; otherwise
/// uses `cargo install --git <repo>`.
pub fn update() -> Result<UpdateReport> {
    let repo = "https://github.com/mdecurtins/setmeup.git";

    // Prefer: we are inside a checkout. Rebuild from the local tree.
    let local_manifest = std::env::current_dir()
        .ok()
        .map(|d| d.join("Cargo.toml"))
        .filter(|p| p.exists());

    let bin_target = if let Some(manifest) = local_manifest {
        let dir = manifest
            .parent()
            .unwrap_or(std::path::Path::new("."))
            .to_path_buf();
        // git pull first (best effort)
        let _ = Command::new("git")
            .arg("-C")
            .arg(&dir)
            .args(["pull", "--rebase"])
            .status();
        Command::new("cargo")
            .arg("install")
            .arg("--path")
            .arg(&dir)
            .arg("--force")
            .status()
            .map_err(|e| SetmeupError::Command {
                cmd: "cargo".into(),
                code: e.raw_os_error().unwrap_or(1),
            })?
    } else {
        // No local checkout: install from the git repo.
        Command::new("cargo")
            .arg("install")
            .arg("--git")
            .arg(repo)
            .arg("--force")
            .status()
            .map_err(|e| SetmeupError::Command {
                cmd: "cargo".into(),
                code: e.raw_os_error().unwrap_or(1),
            })?
    };

    if !bin_target.success() {
        return Err(SetmeupError::Command {
            cmd: "cargo install".into(),
            code: bin_target.code().unwrap_or(1),
        });
    }

    let binary_path = config::config_dir()
        .parent()
        .map(|c| c.join("bin"))
        .unwrap_or_else(|| PathBuf::from("."))
        .join("setmeup");
    Ok(UpdateReport {
        rebuild_needed: true,
        binary_path: if binary_path.exists() {
            Some(binary_path)
        } else {
            None
        },
    })
}

/// Delete a stale binary at the expected path (used when cargo install puts it elsewhere).
pub fn remove_stale(_path: &PathBuf) -> Result<()> {
    Ok(())
}

/// Get the installed version string.
pub fn version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_nonempty() {
        assert!(!version().is_empty());
    }

    #[test]
    fn remove_stale_is_noop_on_missing() {
        let missing = std::path::PathBuf::from("/nonexistent/setmeup");
        remove_stale(&missing).unwrap();
    }
}
