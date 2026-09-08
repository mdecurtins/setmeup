//! Paths and constants for setmeup state on the local machine.

use std::path::PathBuf;

/// The setmeup config directory (`~/.config/setmeup` on Unix).
pub fn config_dir() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("setmeup")
}

/// Path to the declarative manifest (`~/.config/setmeup/manifest.yml`).
pub fn manifest_path() -> PathBuf {
    config_dir().join("manifest.yml")
}

/// Path to the age-encrypted secrets vault (`~/.config/setmeup/secrets.age`).
pub fn secrets_vault_path() -> PathBuf {
    config_dir().join("secrets.age")
}

/// Path to the SSH private key directory.
pub fn ssh_dir() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".ssh")
}

/// Windows-host backup root for WSL2 key survival.
///
/// Returns `Some(<windows-user-profile>/.setmeup/backups)` when running inside
/// WSL2 (the C: drive is mounted at `/mnt/c`), else `None`. The Windows user
/// profile is resolved from `WSL_USER` when set, then by scanning `/mnt/c/Users`
/// for a real account directory. If that fails, `None` (no backup support).
pub fn windows_host_backup_root() -> Option<PathBuf> {
    let mnt_c_users = PathBuf::from("/mnt/c/Users");
    if !mnt_c_users.is_dir() {
        return None;
    }
    let dir = match std::env::var("WSL_USER").ok() {
        Some(user) if !user.is_empty() => Some(mnt_c_users.join(user)),
        _ => std::fs::read_dir(&mnt_c_users)
            .ok()?
            .filter_map(Result::ok)
            .map(|e| e.path())
            .find(|p| p.is_dir()),
    };
    Some(dir?.join(".setmeup").join("backups"))
}
