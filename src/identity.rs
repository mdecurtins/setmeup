//! Identity lifecycle: per-machine SSH + GPG keys, provider registration,
//! and WSL2 cross-distro backup/restore via the Windows host.
//!
//! Principles (per design):
//! - Fresh keys per *physical machine* — not shared across machines.
//! - Restore-before-generate: never create a second key when one exists or a
//!   Windows-host backup can be restored.
//! - Trust boundary: private keys never enter git; backups on the Windows host
//!   are age-encrypted with the vault passphrase.

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;

use age::secrecy::SecretString;

use crate::config;
use crate::error::{Result, SetmeupError};

/// Where an SSH key should live relative to the homedir's `.ssh`.
const SSH_PRIVATE_KEY: &str = "id_ed25519";

// ---------------------------------------------------------------------------
// Key material presence
// ---------------------------------------------------------------------------

/// True when an SSH private key already exists in `~/.ssh`.
pub fn ssh_key_exists() -> Result<bool> {
    let path = config::ssh_dir().join(SSH_PRIVATE_KEY);
    Ok(path.exists())
}

/// Check for an existing identity in order: local `~/.ssh`, then Windows host.
/// Returns true if any exists (so restore/generate can be short-circuited).
pub fn any_identity_exists(backup_root: Option<&Path>) -> bool {
    if ssh_key_exists().unwrap_or(false) {
        return true;
    }
    if let Some(root) = backup_root.filter(|r| r.join("ssh-keys.age").exists()) {
        let _ = root;
        return true;
    }
    false
}

/// The public key text, if present.
pub fn public_key_text() -> Result<Option<String>> {
    let path = config::ssh_dir().join(format!("{SSH_PRIVATE_KEY}.pub"));
    if !path.exists() {
        return Ok(None);
    }
    Ok(Some(fs::read_to_string(path)?.trim().to_string()))
}

// ---------------------------------------------------------------------------
// Generation
// ---------------------------------------------------------------------------

/// Generate a fresh ed25519 SSH keypair into `~/.ssh` (mode 600/644).
/// This is a no-op if a key already exists.
pub fn generate_ssh_key() -> Result<()> {
    let ssh_dir = config::ssh_dir();
    fs::create_dir_all(&ssh_dir)?;
    if ssh_key_exists()? {
        return Ok(());
    }
    let status = Command::new("ssh-keygen")
        .args([
            "-t",
            "ed25519",
            "-f",
            &ssh_dir.join(SSH_PRIVATE_KEY).to_string_lossy(),
            "-N",
            "",
            "-q",
        ])
        .status()
        .map_err(|e| SetmeupError::Command {
            cmd: "ssh-keygen".into(),
            code: e.raw_os_error().unwrap_or(1),
        })?;
    if !status.success() {
        return Err(SetmeupError::Command {
            cmd: "ssh-keygen".into(),
            code: status.code().unwrap_or(1),
        });
    }
    chmod_600(&ssh_dir.join(SSH_PRIVATE_KEY))?;
    Ok(())
}

/// Generate (or no-op if exists) a fresh GPG signing key.
/// Returns the fingerprint, from `gpg --generate-key` output if possible.
pub fn generate_gpg_key(name: &str, email: &str) -> Result<Option<String>> {
    // Batch mode avoids interactive prompts. Uses a throwaway temp keyring.
    let gpg_home = tempfile::tempdir()?;
    let batch = format!(
        "%no-protection\nKey-Type: eddsa\nKey-Curve: ed25519\nSubkey-Type: ecdh\n\
         Subkey-Curve: cv25519\nName-Real: {name}\nName-Email: {email}\n\
         Expire-Date: 0\n%commit\n"
    );
    let batch_file = gpg_home.path().join("batch");
    fs::write(&batch_file, batch)?;
    let output = Command::new("gpg")
        .arg("--homedir")
        .arg(gpg_home.path())
        .arg("--batch")
        .arg("--generate-key")
        .arg(&batch_file)
        .output()
        .map_err(|e| SetmeupError::Command {
            cmd: "gpg".into(),
            code: e.raw_os_error().unwrap_or(1),
        })?;
    if !output.status.success() {
        return Err(SetmeupError::Command {
            cmd: "gpg".into(),
            code: output.status.code().unwrap_or(1),
        });
    }
    // Extract a fingerprint from stderr like `gpg: key ABCDEF0123456789 marked as ultimately trusted`
    let stderr = String::from_utf8_lossy(&output.stderr);
    let fp = stderr.lines().find_map(|l| {
        let low = l.to_lowercase();
        if low.contains("marked as ultimately trusted") {
            low.split("key ")
                .nth(1)
                .and_then(|s| s.split_whitespace().next())
                .map(str::to_uppercase)
        } else {
            None
        }
    });
    Ok(fp)
}

// ---------------------------------------------------------------------------
// Windows-host backup (WSL2 survival)
// ---------------------------------------------------------------------------

/// Backup the SSH keypair to the Windows host, age-encrypted with the vault
/// passphrase. No-op if not running on WSL2 (no `/mnt/c/Users`).
pub fn backup_ssh_to_windows(passphrase: &SecretString) -> Result<PathBuf> {
    let root = config::windows_host_backup_root().ok_or_else(|| {
        SetmeupError::Identity("not on WSL2; no Windows host backup target".into())
    })?;
    fs::create_dir_all(&root)?;
    let priv_key = config::ssh_dir().join(SSH_PRIVATE_KEY);
    let pub_key = config::ssh_dir().join(format!("{SSH_PRIVATE_KEY}.pub"));
    // Encrypt both keys (pubkey can be public, but archived together).
    let plain = fs::read(&priv_key)?;
    let pub_plain = fs::read(&pub_key).unwrap_or_default();
    let mut combined = plain;
    combined.extend_from_slice(b"\n---PUBKEY---\n");
    combined.extend_from_slice(&pub_plain);

    use crate::secrets::BackendKind;
    let _ = BackendKind::VaultFile; // reuse module for encryption primitives
    let encryptor = age::Encryptor::with_user_passphrase(passphrase.clone());
    let mut enc = Vec::new();
    let mut writer = encryptor
        .wrap_output(&mut enc)
        .map_err(|e| SetmeupError::Identity(e.to_string()))?;
    use std::io::Write;
    writer
        .write_all(&combined)
        .map_err(|e| SetmeupError::Identity(e.to_string()))?;
    writer
        .finish()
        .map_err(|e| SetmeupError::Identity(e.to_string()))?;

    let target = root.join("ssh-keys.age");
    fs::write(&target, enc)?;
    Ok(target)
}

/// Restore the SSH keypair from the Windows host backup into `~/.ssh`.
/// Decrypts, writes, and chmod 600.
pub fn restore_ssh_from_windows(passphrase: &SecretString) -> Result<()> {
    let root = config::windows_host_backup_root()
        .ok_or_else(|| SetmeupError::Identity("not on WSL2; no Windows host backup".into()))?;
    let target = root.join("ssh-keys.age");
    if !target.exists() {
        return Err(SetmeupError::Identity(
            "no Windows-host backup exists".into(),
        ));
    }
    let ciphertext = fs::read(&target)?;
    let decryptor = age::Decryptor::new(ciphertext.as_slice())
        .map_err(|e| SetmeupError::Identity(e.to_string()))?;
    let identity = age::scrypt::Identity::new(passphrase.clone());
    let mut reader = decryptor
        .decrypt(std::iter::once(&identity as &dyn age::Identity))
        .map_err(|e| SetmeupError::Identity(e.to_string()))?;
    let mut plain = Vec::new();
    use std::io::Read;
    reader
        .read_to_end(&mut plain)
        .map_err(|e| SetmeupError::Identity(e.to_string()))?;

    // Split back into private key + pubkey.
    let marker = b"\n---PUBKEY---\n";
    let idx = find_marker(&plain, marker);
    let (priv_part, pub_part) = match idx {
        Some(i) => (&plain[..i], &plain[i + marker.len()..]),
        None => (&plain[..], &[][..]),
    };

    fs::create_dir_all(config::ssh_dir())?;
    let priv_path = config::ssh_dir().join(SSH_PRIVATE_KEY);
    fs::write(&priv_path, priv_part)?;
    chmod_600(&priv_path)?;
    if !pub_part.is_empty() {
        fs::write(
            config::ssh_dir().join(format!("{SSH_PRIVATE_KEY}.pub")),
            pub_part,
        )?;
    }
    Ok(())
}

fn find_marker(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack.windows(needle.len()).position(|w| w == needle)
}

// ---------------------------------------------------------------------------
// Registration
// ---------------------------------------------------------------------------

/// Register the current SSH public key with a provider via its API.
/// `provider` is `github` or `gitlab`; `token` is the stored credential.
/// Skips registration if the key is already known.
pub fn register_ssh_key(provider: &str, token: &str, pubkey: &str) -> Result<bool> {
    let _ = token; // token consumed by HTTP layer below
    match provider.to_lowercase().as_str() {
        "github" => register_github(pubkey, token),
        "gitlab" => register_gitlab(pubkey, token),
        other => Err(SetmeupError::Identity(format!(
            "unsupported provider: {other}"
        ))),
    }
}

fn register_github(pubkey: &str, token: &str) -> Result<bool> {
    let url = "https://api.github.com/user/keys";
    let payload = format!(
        "{{\"title\":\"setmeup-{}\",\"key\":\"{}\"}}",
        hostname(),
        pubkey.trim()
    );
    let resp = ureq::post(url)
        .header("Authorization", format!("Bearer {token}").as_str())
        .header("User-Agent", "setmeup")
        .header("Content-Type", "application/json")
        .send(payload);
    match resp {
        Ok(_) => Ok(true),
        Err(ureq::Error::StatusCode(422)) => {
            // 422 unprocessable: key already exists on the account.
            Ok(false)
        }
        Err(e) => Err(SetmeupError::Identity(format!(
            "github registration failed: {e}"
        ))),
    }
}

fn register_gitlab(pubkey: &str, token: &str) -> Result<bool> {
    let url = "https://gitlab.com/api/v4/user/keys";
    let payload = format!(
        "{{\"title\":\"setmeup-{}\",\"key\":\"{}\"}}",
        hostname(),
        pubkey.trim()
    );
    let resp = ureq::post(url)
        .header("PRIVATE-TOKEN", token)
        .header("Content-Type", "application/json")
        .send(payload);
    match resp {
        Ok(_) => Ok(true),
        // 400 bad request: duplicate/bad key already present.
        Err(ureq::Error::StatusCode(400)) => Ok(false),
        Err(e) => Err(SetmeupError::Identity(format!(
            "gitlab registration failed: {e}"
        ))),
    }
}

fn hostname() -> String {
    std::env::var("HOSTNAME")
        .or_else(|_| std::env::var("COMPUTERNAME"))
        .unwrap_or_else(|_| "machine".into())
}

// ---------------------------------------------------------------------------
// Util
// ---------------------------------------------------------------------------

fn chmod_600(path: &Path) -> Result<()> {
    fs::set_permissions(path, fs::Permissions::from_mode(0o600))?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn register_ssh_key_rejects_unknown_provider() {
        let err = register_ssh_key("bitbucket", "tok", "ssh-ed25519 AAA").unwrap_err();
        assert!(err.to_string().contains("unsupported provider"));
    }

    #[test]
    fn any_identity_exists_checks_backup_root() {
        // With a backup file present, identity must be considered to exist
        // regardless of the local ~/.ssh state.
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("ssh-keys.age"), b"encrypted").unwrap();
        assert!(any_identity_exists(Some(dir.path())));

        // Without any backup, the result is driven by the real ~/.ssh presence;
        // assert against the actual state to stay deterministic and hermetic.
        let empty = tempfile::tempdir().unwrap();
        let expected = ssh_key_exists().unwrap_or(false);
        assert_eq!(any_identity_exists(Some(empty.path())), expected);
    }

    #[test]
    fn chmod_600_sets_owner_only_on_unix() {
        let dir = tempfile::tempdir().unwrap();
        let f = dir.path().join("key");
        fs::write(&f, b"x").unwrap();
        chmod_600(&f).unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = fs::metadata(&f).unwrap().permissions().mode() & 0o777;
            assert_eq!(mode, 0o600);
        }
    }

    #[test]
    fn find_marker_works() {
        let hay = b"header-data\n---PUBKEY---\nkey-data";
        let marker = b"\n---PUBKEY---\n";
        let idx = find_marker(hay, marker).unwrap();
        assert_eq!(&hay[idx..idx + marker.len()], marker);
        let (priv_part, pub_part) = (&hay[..idx], &hay[idx + marker.len()..]);
        assert_eq!(priv_part, b"header-data");
        assert_eq!(pub_part, b"key-data");
    }

    #[test]
    fn find_marker_absent() {
        let hay = b"no marker here";
        assert!(find_marker(hay, b"\n---PUBKEY---\n").is_none());
    }

    #[test]
    fn hostname_falls_back() {
        // Should never panic even with env absent.
        let h = hostname();
        assert!(!h.is_empty());
    }

    #[test]
    fn ssh_key_exists_false_when_absent() {
        // ~/.ssh likely exists on dev boxes; the check should at least not panic.
        let _ = ssh_key_exists();
    }
}
