//! Declarative desired-state manifest.
//!
//! The manifest (`manifest.yml`) is the spec of what a machine should look
//! like: tools, dotfiles, shell configuration, OS preferences, and the policy
//! for acquiring secrets. It is portable across OSes and contains NO secret
//! material.

use std::collections::BTreeMap;
use std::fmt;
use std::fs;
use std::io::Write;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::config;
use crate::error::{Result, SetmeupError};

/// Target operating system family. Used for per-OS resolution of the manifest.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Os {
    Ubuntu,
    Macos,
    Windows,
}

impl fmt::Display for Os {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Os::Ubuntu => "ubuntu",
            Os::Macos => "macos",
            Os::Windows => "windows",
        };
        f.write_str(s)
    }
}

impl Os {
    /// The OS of the machine setmeup is currently running on.
    pub fn current() -> Self {
        if cfg!(target_os = "macos") {
            Os::Macos
        } else if cfg!(target_os = "windows") {
            Os::Windows
        } else {
            // Linux: assume Ubuntu-family (the default target, per project)
            Os::Ubuntu
        }
    }
}

/// How a secret is acquired from the user.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SecretAcquire {
    #[default]
    Paste,
    DeviceFlow,
    Env,
}

/// How a secret is stored once acquired.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SecretBackend {
    #[default]
    VaultFile,
    Keyring,
}

/// A declared tool. `name` is the canonical (cross-OS) identifier; the
/// per-OS fields override the package name on each platform.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub struct Tool {
    pub name: String,
    #[serde(default)]
    pub apt: Option<String>,
    #[serde(default)]
    pub brew: Option<String>,
    #[serde(default)]
    pub winget: Option<String>,
}

impl Tool {
    /// Package name to use on `os`, falling back to the canonical `name`.
    pub fn package_name_for(&self, os: Os) -> &str {
        match os {
            Os::Ubuntu => self.apt.as_deref().unwrap_or(&self.name),
            Os::Macos => self.brew.as_deref().unwrap_or(&self.name),
            Os::Windows => self.winget.as_deref().unwrap_or(&self.name),
        }
    }
}

/// A dotfile mapping: repo-relative `source` to home-relative `dest`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Dotfile {
    pub source: String,
    pub dest: String,
}

/// Shell configuration to apply (idempotently) to the user's rc file.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct ShellConfig {
    #[serde(default)]
    pub rc_lines: Vec<String>,
}

/// OS-level preferences (currently minimal by design).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct OsPreferences {
    #[serde(default)]
    pub timezone: Option<String>,
    #[serde(default)]
    pub locale: Option<String>,
}

/// Secret-acquisition policy. Declares HOW and WHERE a credential is stored;
/// never the credential itself.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct SecretPolicy {
    pub name: String,
    #[serde(default)]
    pub acquire: SecretAcquire,
    #[serde(default)]
    pub backend: SecretBackend,
}

/// The full desired state of a machine.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Manifest {
    #[serde(default)]
    pub tools: Vec<Tool>,
    #[serde(default)]
    pub dotfiles: Vec<Dotfile>,
    #[serde(default)]
    pub shell: ShellConfig,
    #[serde(default)]
    pub os: OsPreferences,
    #[serde(default)]
    pub secrets: Vec<SecretPolicy>,
    /// Extra free-form config for custom scripts (forward-compatible).
    #[serde(default)]
    pub extra: BTreeMap<String, String>,
}

impl Manifest {
    /// Parse and validate a manifest from raw YAML.
    pub fn from_yaml(raw: &str) -> Result<Self> {
        let manifest: Manifest =
            serde_yaml_ng::from_str(raw).map_err(|e| SetmeupError::Manifest(e.to_string()))?;
        manifest.validate()?;
        Ok(manifest)
    }

    /// Load the manifest at `path`, or the default config location when `None`.
    pub fn load(path: Option<&Path>) -> Result<Self> {
        let path = path.map_or_else(config::manifest_path, Path::to_path_buf);
        let raw = fs::read_to_string(&path)
            .map_err(|e| SetmeupError::Manifest(format!("cannot read {}: {e}", path.display())))?;
        Self::from_yaml(&raw)
    }

    /// Serialize to YAML (for the TUI wizard writing the manifest).
    pub fn to_yaml(&self) -> Result<String> {
        serde_yaml_ng::to_string(self).map_err(|e| SetmeupError::Manifest(e.to_string()))
    }

    /// Persist the manifest to `path` (or the default config location).
    pub fn save(&self, path: Option<&Path>) -> Result<()> {
        let path = path.map_or_else(config::manifest_path, Path::to_path_buf);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let yaml = self.to_yaml()?;
        let mut f = fs::File::create(&path)?;
        f.write_all(yaml.as_bytes())?;
        Ok(())
    }

    /// Validate invariants; schema violations are named by section.
    pub fn validate(&self) -> Result<()> {
        for tool in &self.tools {
            if tool.name.trim().is_empty() {
                return Err(SetmeupError::Manifest(
                    "tools: a tool name must not be empty".into(),
                ));
            }
        }
        for dotfile in &self.dotfiles {
            if dotfile.source.trim().is_empty() {
                return Err(SetmeupError::Manifest(
                    "dotfiles: a source must not be empty".into(),
                ));
            }
            if dotfile.dest.trim().is_empty() {
                return Err(SetmeupError::Manifest(
                    "dotfiles: a dest must not be empty".into(),
                ));
            }
        }
        for policy in &self.secrets {
            if policy.name.trim().is_empty() {
                return Err(SetmeupError::Manifest(
                    "secrets: a secret name must not be empty".into(),
                ));
            }
        }
        Ok(())
    }

    /// Resolve the manifest for a specific OS (used by provisioning).
    pub fn resolve_for(&self, os: Os) -> ResolvedManifest {
        ResolvedManifest {
            packages: self
                .tools
                .iter()
                .map(|t| (t.name.clone(), t.package_name_for(os).to_owned()))
                .collect(),
            dotfiles: self.dotfiles.clone(),
            shell: self.shell.clone(),
        }
    }
}

/// A manifest resolved for a concrete OS: package names are decided, dotfiles
/// and shell config pass through.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ResolvedManifest {
    pub packages: Vec<(String, String)>,
    pub dotfiles: Vec<Dotfile>,
    pub shell: ShellConfig,
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = r#"
tools:
  - name: git
    apt: git
    brew: git
    winget: Git.Git
  - name: neovim
  - name: docker
    apt: docker.io
dotfiles:
  - source: dotfiles/.zshrc
    dest: .zshrc
shell:
  rc-lines:
    - 'alias gs="git status"'
os:
  timezone: UTC
secrets:
  - name: github_token
    acquire: device-flow
    backend: keyring
"#;

    #[test]
    fn parses_sample_manifest() {
        let m = Manifest::from_yaml(SAMPLE).expect("valid sample");
        assert_eq!(m.tools.len(), 3);
        assert_eq!(m.dotfiles.len(), 1);
        assert_eq!(m.secrets[0].name, "github_token");
        assert_eq!(m.shell.rc_lines, ["alias gs=\"git status\""]);
    }

    #[test]
    fn rejects_empty_tool_name_naming_section() {
        let raw = "tools:\n  - name: ''\n";
        let err = Manifest::from_yaml(raw).expect_err("must fail");
        assert!(err.to_string().contains("tools"), "named section: {err}");
    }

    #[test]
    fn rejects_malformed_yaml() {
        let err = Manifest::from_yaml("tools: [unclosed").expect_err("must fail");
        assert!(err.to_string().contains("manifest"));
    }

    #[test]
    fn per_os_package_resolution_with_fallback() {
        let m = Manifest::from_yaml(SAMPLE).unwrap();
        let r = m.resolve_for(Os::Ubuntu);
        let git = r
            .packages
            .iter()
            .find(|(n, _)| n == "git")
            .unwrap()
            .1
            .as_str();
        let nvim = r
            .packages
            .iter()
            .find(|(n, _)| n == "neovim")
            .unwrap()
            .1
            .as_str();
        let docker = r
            .packages
            .iter()
            .find(|(n, _)| n == "docker")
            .unwrap()
            .1
            .as_str();
        assert_eq!(git, "git");
        assert_eq!(nvim, "neovim", "falls back to canonical name");
        assert_eq!(docker, "docker.io", "apt override wins");
    }

    #[test]
    fn empty_manifest_applies_cleanly() {
        let m = Manifest::default();
        m.validate().unwrap();
        let r = m.resolve_for(Os::Ubuntu);
        assert!(r.packages.is_empty());
        assert!(r.dotfiles.is_empty());
    }

    #[test]
    fn yaml_round_trip_preserves_content() {
        let m = Manifest::from_yaml(SAMPLE).unwrap();
        let yaml = m.to_yaml().unwrap();
        let m2 = Manifest::from_yaml(&yaml).unwrap();
        assert_eq!(m, m2);
    }
}
