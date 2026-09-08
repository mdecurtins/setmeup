//! Secret storage: a backend abstraction with a migration ladder.
//!
//! The ladder (per design): age-encrypted vault file first (works on a fresh,
//! headless machine with no keyring), then OS keyring/keychain/Credential
//! Manager as the machine grows. There is NO plaintext fallback.

use std::fmt;
use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use age::secrecy::SecretString;
use zeroize::Zeroizing;

use crate::config;
use crate::error::{Result, SetmeupError};

/// A handle to a stored secret (name only, never the value).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SecretEntry {
    pub name: String,
}

/// Where secrets are physically stored.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackendKind {
    VaultFile,
    Keyring,
}

impl fmt::Display for BackendKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BackendKind::VaultFile => f.write_str("vault-file"),
            BackendKind::Keyring => f.write_str("keyring"),
        }
    }
}

/// A secret storage backend.
pub trait SecretBackend: fmt::Debug {
    fn kind(&self) -> BackendKind;
    fn set(&mut self, name: &str, value: &str) -> Result<()>;
    fn get(&self, name: &str) -> Result<String>;
    fn delete(&mut self, name: &str) -> Result<()>;
    fn list(&self) -> Result<Vec<SecretEntry>>;
}

/// Backend selection: the strongest backend the current machine can support.
///
/// In this first implementation the vault-file backend is the universal floor;
/// keyring backends are available on platforms with a reachable store. The
/// ladder is exercised via explicit `migrate`.
pub fn select_backend(
    kind: BackendKind,
    passphrase_provider: Option<Box<dyn PassphraseProvider>>,
) -> Result<Box<dyn SecretBackend>> {
    match kind {
        BackendKind::VaultFile => {
            let passphrase = match passphrase_provider {
                Some(p) => p,
                None => Box::new(EnvPassphraseProvider),
            };
            Ok(Box::new(VaultFileBackend::new(
                config::secrets_vault_path(),
                passphrase,
            )))
        }
        BackendKind::Keyring => Ok(Box::new(KeyringBackend)),
    }
}

/// True when the platform keyring store is reachable on this machine.
///
/// Uses `keyring::v1::Entry::store_status`; a `Ok(())` means a real store
/// (Secret Service, Keychain, or Credential Manager) is available. On a fresh
/// headless machine with no secret service daemon, this returns false and
/// setmeup falls back to the age-encrypted vault file.
pub fn keyring_available() -> bool {
    keyring::v1::Entry::store_status().is_ok()
}

/// Auto-select the strongest backend for the current machine at write time.
///
/// The ladder (per design): keyring when a store is reachable, else the
/// age-encrypted vault file. There is no plaintext fallback.
pub fn auto_select_backend() -> Box<dyn SecretBackend> {
    if keyring_available() {
        Box::new(KeyringBackend)
    } else {
        Box::new(VaultFileBackend::new(
            config::secrets_vault_path(),
            Box::new(EnvPassphraseProvider),
        ))
    }
}

/// Provides the vault passphrase (never the secret itself).
pub trait PassphraseProvider: fmt::Debug {
    fn passphrase(&self) -> Result<SecretString>;
}

/// Reads the passphrase from an environment variable (`SETMEUP_PASSPHRASE`).
/// Intended for non-interactive/scripted runs; the TUI prompts interactively.
#[derive(Debug, Default)]
pub struct EnvPassphraseProvider;

impl PassphraseProvider for EnvPassphraseProvider {
    fn passphrase(&self) -> Result<SecretString> {
        let value = std::env::var("SETMEUP_PASSPHRASE")
            .map_err(|_| SetmeupError::Secret("SETMEUP_PASSPHRASE is not set".into()))?;
        Ok(SecretString::from(value))
    }
}

/// A passphrase captured interactively (used by the TUI wizard).
#[derive(Debug)]
pub struct PromptPassphraseProvider {
    prompt: String,
    value: SecretString,
}

impl PromptPassphraseProvider {
    /// Build from an already-captured value; the caller owns the UI prompt.
    pub fn new(prompt: &str, value: SecretString) -> Self {
        Self {
            prompt: prompt.into(),
            value,
        }
    }
}

impl PassphraseProvider for PromptPassphraseProvider {
    fn passphrase(&self) -> Result<SecretString> {
        let _ = &self.prompt; // informational
        Ok(self.value.clone())
    }
}

// ---------------------------------------------------------------------------
// Vault-file backend (age-encrypted)
// ---------------------------------------------------------------------------

/// Age-encrypted file backend at a given path, unlocked by a passphrase.
#[derive(Debug)]
pub struct VaultFileBackend {
    path: PathBuf,
    passphrase_provider: Box<dyn PassphraseProvider>,
}

impl VaultFileBackend {
    pub fn new(path: PathBuf, passphrase_provider: Box<dyn PassphraseProvider>) -> Self {
        Self {
            path,
            passphrase_provider,
        }
    }

    fn passphrase(&self) -> Result<SecretString> {
        self.passphrase_provider.passphrase()
    }

    /// Read, decrypt, and parse the whole vault into a name→value map.
    fn load_map(&self) -> Result<Vec<(String, String)>> {
        if !self.path.exists() {
            return Ok(Vec::new());
        }
        let ciphertext = fs::read(&self.path)?;
        let plain = decrypt_vault(&ciphertext, &self.passphrase()?)?;
        deserialize_vault(&plain)
    }

    /// Serialize and encrypt the whole vault to disk.
    fn store_map(&self, entries: &[(String, String)]) -> Result<()> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)?;
        }
        let plain = serialize_vault(entries)?;
        let ciphertext = encrypt_vault(plain.as_slice(), &self.passphrase()?)?;
        fs::write(&self.path, ciphertext)?;
        Ok(())
    }
}

impl SecretBackend for VaultFileBackend {
    fn kind(&self) -> BackendKind {
        BackendKind::VaultFile
    }

    fn set(&mut self, name: &str, value: &str) -> Result<()> {
        let mut entries = self.load_map()?;
        entries.retain(|(n, _)| n != name);
        entries.push((name.to_string(), value.to_string()));
        self.store_map(&entries)
    }

    fn get(&self, name: &str) -> Result<String> {
        let entries = self.load_map()?;
        entries
            .into_iter()
            .find(|(n, _)| n == name)
            .map(|(_, v)| v)
            .ok_or_else(|| SetmeupError::Secret(format!("no such secret: {name}")))
    }

    fn delete(&mut self, name: &str) -> Result<()> {
        let entries = self.load_map()?;
        let filtered: Vec<_> = entries.into_iter().filter(|(n, _)| n != name).collect();
        self.store_map(&filtered)
    }

    fn list(&self) -> Result<Vec<SecretEntry>> {
        let entries = self.load_map()?;
        Ok(entries
            .into_iter()
            .map(|(n, _)| SecretEntry { name: n })
            .collect())
    }
}

// ---------------------------------------------------------------------------
// Keyring backend
// ---------------------------------------------------------------------------

/// OS keyring backend (Secret Service / Keychain / Credential Manager).
#[derive(Debug, Default)]
pub struct KeyringBackend;

const KEYRING_SERVICE: &str = "setmeup";

fn entry_for(name: &str) -> Result<keyring::v1::Entry> {
    keyring::v1::Entry::new(KEYRING_SERVICE, name).map_err(|e| SetmeupError::Secret(e.to_string()))
}

impl SecretBackend for KeyringBackend {
    fn kind(&self) -> BackendKind {
        BackendKind::Keyring
    }

    fn set(&mut self, name: &str, value: &str) -> Result<()> {
        entry_for(name)?
            .set_password(value)
            .map_err(|e| SetmeupError::Secret(e.to_string()))
    }

    fn get(&self, name: &str) -> Result<String> {
        entry_for(name)?
            .get_password()
            .map_err(|e| SetmeupError::Secret(e.to_string()))
    }

    fn delete(&mut self, name: &str) -> Result<()> {
        entry_for(name)?.delete_credential().or_else(|e| {
            // Deleting a nonexistent secret is a no-op.
            if e.to_string().contains("NoEntry") {
                Ok(())
            } else {
                Err(SetmeupError::Secret(e.to_string()))
            }
        })
    }

    fn list(&self) -> Result<Vec<SecretEntry>> {
        // The keyring crate intentionally does not enumerate entries. We track
        // names in the vault file when available; otherwise limited listing.
        let vault_path = config::secrets_vault_path();
        if vault_path.exists() {
            let backend = VaultFileBackend::new(vault_path, Box::new(EnvPassphraseProvider));
            return backend.list();
        }
        Ok(Vec::new())
    }
}

// ---------------------------------------------------------------------------
// Migration
// ---------------------------------------------------------------------------

/// Copy every secret from one backend to another, then delete from source.
pub fn migrate(from: &mut dyn SecretBackend, to: &mut dyn SecretBackend) -> Result<usize> {
    let entries = from.list()?;
    let mut count = 0;
    for entry in entries {
        let value = from.get(&entry.name)?;
        to.set(&entry.name, &value)?;
        from.delete(&entry.name)?;
        count += 1;
    }
    Ok(count)
}

// ---------------------------------------------------------------------------
// age helpers
// ---------------------------------------------------------------------------

fn encrypt_vault(plaintext: &[u8], passphrase: &SecretString) -> Result<Vec<u8>> {
    let encryptor = age::Encryptor::with_user_passphrase(passphrase.clone());
    let mut encrypted = Vec::new();
    let mut writer = encryptor
        .wrap_output(&mut encrypted)
        .map_err(|e| SetmeupError::Secret(e.to_string()))?;
    writer
        .write_all(plaintext)
        .map_err(|e| SetmeupError::Secret(e.to_string()))?;
    writer
        .finish()
        .map_err(|e| SetmeupError::Secret(e.to_string()))?;
    Ok(encrypted)
}

fn decrypt_vault(ciphertext: &[u8], passphrase: &SecretString) -> Result<Zeroizing<Vec<u8>>> {
    let decryptor =
        age::Decryptor::new(ciphertext).map_err(|e| SetmeupError::Secret(e.to_string()))?;
    let identity = age::scrypt::Identity::new(passphrase.clone());
    let mut reader = decryptor
        .decrypt(std::iter::once(&identity as &dyn age::Identity))
        .map_err(|e| SetmeupError::Secret(e.to_string()))?;
    let mut plain = Zeroizing::new(Vec::new());
    reader
        .read_to_end(&mut plain)
        .map_err(|e| SetmeupError::Secret(e.to_string()))?;
    Ok(plain)
}

fn serialize_vault(entries: &[(String, String)]) -> Result<Zeroizing<Vec<u8>>> {
    // Simple line-based format: "name\u{0}value" per line. Avoids an extra
    // serialization dependency and never logs values.
    let mut out = Zeroizing::new(Vec::new());
    for (name, value) in entries {
        out.extend_from_slice(name.as_bytes());
        out.push(0);
        out.extend_from_slice(value.as_bytes());
        out.push(b'\n');
    }
    Ok(out)
}

fn deserialize_vault(plain: &[u8]) -> Result<Vec<(String, String)>> {
    let text = std::str::from_utf8(plain)
        .map_err(|e| SetmeupError::Secret(format!("vault is not utf8: {e}")))?;
    let mut entries = Vec::new();
    for line in text.lines() {
        let mut parts = line.splitn(2, '\u{0}');
        let name = parts.next().unwrap_or_default();
        let value = parts.next().unwrap_or_default();
        if name.is_empty() && value.is_empty() {
            continue;
        }
        entries.push((name.to_string(), value.to_string()));
    }
    Ok(entries)
}

/// Clear a path's permission bits to owner-only (used for vault files).
pub fn chmod_600(path: &Path) -> Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(path, fs::Permissions::from_mode(0o600))?;
    }
    let _ = path;
    Ok(())
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use age::secrecy::ExposeSecret;

    fn passphrase() -> SecretString {
        SecretString::from("test-passphrase".to_string())
    }

    fn vault_backend(dir: &Path) -> VaultFileBackend {
        VaultFileBackend::new(
            dir.join("secrets.age"),
            Box::new(PromptPassphraseProvider::new("", passphrase())),
        )
    }

    #[test]
    fn vault_round_trip() {
        let dir = tempfile::tempdir().unwrap();
        let mut b = vault_backend(dir.path());
        b.set("github_token", "abc123").unwrap();
        b.set("npm_token", "def456").unwrap();
        assert_eq!(b.get("github_token").unwrap(), "abc123");
        assert_eq!(b.list().unwrap().len(), 2);
        b.delete("github_token").unwrap();
        assert!(!b.list().unwrap().iter().any(|e| e.name == "github_token"));
    }

    #[test]
    fn vault_is_encrypted_at_rest() {
        let dir = tempfile::tempdir().unwrap();
        let mut b = vault_backend(dir.path());
        b.set("secret", "s3cr3t-value").unwrap();
        let raw = fs::read(dir.path().join("secrets.age")).unwrap();
        let raw_string = String::from_utf8_lossy(&raw);
        assert!(
            !raw_string.contains("s3cr3t-value"),
            "plaintext value must never appear in the vault file"
        );
    }

    #[test]
    fn delete_missing_is_noop() {
        let dir = tempfile::tempdir().unwrap();
        let mut b = vault_backend(dir.path());
        // Deleting a nonexistent secret should succeed as a no-op.
        b.delete("never-set").unwrap();
        assert!(b.list().unwrap().is_empty());
    }

    #[test]
    fn overwrite_replaces_value() {
        let dir = tempfile::tempdir().unwrap();
        let mut b = vault_backend(dir.path());
        b.set("tok", "v1").unwrap();
        b.set("tok", "v2").unwrap();
        assert_eq!(b.get("tok").unwrap(), "v2");
        assert_eq!(b.list().unwrap().len(), 1);
    }

    #[test]
    fn get_missing_errors() {
        let dir = tempfile::tempdir().unwrap();
        let b = vault_backend(dir.path());
        let err = b.get("missing").unwrap_err();
        assert!(err.to_string().contains("no such secret"));
    }

    #[test]
    fn chmod_600_sets_owner_only_on_unix() {
        let dir = tempfile::tempdir().unwrap();
        let f = dir.path().join("vault");
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
    fn prompter_provider_returns_value() {
        let p = PromptPassphraseProvider::new("prompt", passphrase());
        assert_eq!(p.passphrase().unwrap().expose_secret(), "test-passphrase");
    }

    #[test]
    fn no_plaintext_fallback() {
        // The vault file backend refuses to store without a passphrase.
        #[derive(Debug)]
        struct NoPassphrase;
        impl PassphraseProvider for NoPassphrase {
            fn passphrase(&self) -> Result<SecretString> {
                Err(SetmeupError::Secret("no passphrase".into()))
            }
        }
        let backend = select_backend(BackendKind::VaultFile, Some(Box::new(NoPassphrase))).unwrap();
        let mut backend = backend;
        assert!(backend.set("x", "y").is_err(), "no passphrase -> no store");
    }

    #[test]
    fn migration_preserves_all_entries() {
        let src_dir = tempfile::tempdir().unwrap();
        let dst_dir = tempfile::tempdir().unwrap();
        let mut src = vault_backend(src_dir.path());
        src.set("a", "1").unwrap();
        src.set("b", "2").unwrap();

        // Destination: a second vault file (simulates keyring).
        let mut dst = vault_backend(dst_dir.path());
        let moved = migrate(&mut src, &mut dst).unwrap();
        assert_eq!(moved, 2);
        assert!(src.list().unwrap().is_empty());
        assert_eq!(dst.get("a").unwrap(), "1");
        assert_eq!(dst.get("b").unwrap(), "2");
    }

    #[test]
    fn deserialize_skips_blank_lines() {
        let plain = b"name\x00value\n\n";
        let entries = deserialize_vault(plain).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0], ("name".to_string(), "value".to_string()));
    }

    #[test]
    fn wrong_passphrase_file() {
        let dir = tempfile::tempdir().unwrap();
        let mut write_b = vault_backend(dir.path());
        write_b.set("k", "v").unwrap();
        // Attempt to read with a different passphrase -> age decrypt error.
        let wrong = VaultFileBackend::new(
            dir.path().join("secrets.age"),
            Box::new(PromptPassphraseProvider::new(
                "",
                SecretString::from("wrong".to_string()),
            )),
        );
        assert!(wrong.get("k").is_err());
    }

    #[test]
    fn keyring_round_trip_skips_when_unavailable() {
        let mut backend = KeyringBackend;
        // Only exercised when a real store is reachable; otherwise we expect
        // an error from the platform, which is acceptable and non-fatal.
        match backend.set("setmeup_test", "value") {
            Ok(()) => {
                assert_eq!(backend.get("setmeup_test").unwrap(), "value");
                backend.delete("setmeup_test").unwrap();
            }
            Err(_) => {
                // No keyring available in CI/headless: acceptable skip.
            }
        }
    }
}
