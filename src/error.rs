//! Shared error types for setmeup.

use thiserror::Error;

/// Top-level error type for library operations.
#[derive(Debug, Error)]
pub enum SetmeupError {
    /// A manifest could not be parsed or failed validation.
    #[error("manifest error: {0}")]
    Manifest(String),

    /// A secret operation failed (read/write/migrate).
    #[error("secret error: {0}")]
    Secret(String),

    /// An identity operation failed (key generation, registration, backup).
    #[error("identity error: {0}")]
    Identity(String),

    /// A provisioning operation failed (package install, dotfiles, shell rc).
    #[error("provisioning error: {0}")]
    Provisioning(String),

    /// The command failed with a non-zero exit code.
    #[error("command failed ({code}): {cmd}")]
    Command { cmd: String, code: i32 },

    /// An I/O error occurred.
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

/// Convenient alias.
pub type Result<T> = std::result::Result<T, SetmeupError>;
