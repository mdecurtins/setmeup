//! Provisioning: per-OS package backends, dotfile symlinking, and idempotent
//! shell configuration.
//!
//! Idempotency contract: presence-check before install; never duplicate a
//! shell line; overwrite only with `ln -sf` semantics; never silently
//! overwrite a conflicting regular file.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::error::{Result, SetmeupError};
use crate::manifest::{Manifest, Os};

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

/// Run a full provisioning pass over a manifest for `os`.
/// Failed items do NOT halt the pass; they're reported at the end.
pub fn provision(
    os: Os,
    repo_root: &Path,
    manifest: &Manifest,
    rc_path: Option<&Path>,
) -> Result<Vec<ProvisionResult>> {
    let mut results = Vec::new();
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
    results.extend(apply_dotfiles(repo_root, manifest)?);
    let rc = rc_path
        .map(Path::to_path_buf)
        .or_else(|| default_rc_path(os));
    if let Some(rc) = rc.filter(|_| !manifest.shell.rc_lines.is_empty()) {
        results.extend(apply_shell_config(&rc, &manifest.shell.rc_lines)?);
    }
    Ok(results)
}

fn default_rc_path(os: Os) -> Option<PathBuf> {
    let home = dirs::home_dir()?;
    match os {
        Os::Ubuntu | Os::Macos => Some(home.join(
            if std::env::var("SHELL").unwrap_or_default().contains("zsh") {
                ".zshrc"
            } else {
                ".bashrc"
            },
        )),
        Os::Windows => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::manifest::{Dotfile, ShellConfig};

    #[test]
    fn presence_check_returns_bool_without_panicking() {
        // `echo` is universally present; the check should be Ok and true on real shells.
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

    #[test]
    fn provision_does_not_halt_on_failure() {
        // A manifest with a bogus source dotfile + a real one; provision
        // should report the bogus as failed and continue.
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
        let results = provision(Os::Ubuntu, &repo, &m, Some(&dir.path().join("rcfile"))).unwrap();
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
}
