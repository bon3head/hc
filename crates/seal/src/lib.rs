//! seal: per-identity SSH signing keys, generated on first use with `ssh-keygen`.
//!
//! Principals are `<role>:<name>@hc`, e.g. `agent:claude-code@hc`. The
//! `allowed_signers` file (format: ssh-keygen(1) ALLOWED SIGNERS) maps each
//! principal to its key and is what git verifies ledger commits against.

use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use std::process::Command;

use bedrock::ir::Role;

#[derive(Debug, thiserror::Error)]
pub enum IdentityError {
    #[error("i/o error")]
    Io(#[from] std::io::Error),
    #[error("ssh-keygen failed: {0}")]
    Keygen(String),
    #[error("invalid identity name {0:?}: use [a-z0-9._-]")]
    Name(String),
    #[error("not an hc principal: {0:?}")]
    Principal(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Identity {
    pub role: Role,
    pub name: String,
    pub key_path: PathBuf,
}

impl Identity {
    pub fn principal(&self) -> String {
        principal(self.role, &self.name)
    }
}

pub fn principal(role: Role, name: &str) -> String {
    let role = match role {
        Role::Agent => "agent",
        Role::Harness => "harness",
        Role::Kernel => "kernel",
    };
    format!("{role}:{name}@hc")
}

pub fn parse_principal(principal: &str) -> Result<(Role, String), IdentityError> {
    let body = principal.strip_suffix("@hc").ok_or_else(|| IdentityError::Principal(principal.into()))?;
    let (role, name) = body.split_once(':').ok_or_else(|| IdentityError::Principal(principal.into()))?;
    let role = match role {
        "agent" => Role::Agent,
        "harness" => Role::Harness,
        "kernel" => Role::Kernel,
        _ => return Err(IdentityError::Principal(principal.into())),
    };
    Ok((role, name.to_owned()))
}

fn valid_name(name: &str) -> bool {
    !name.is_empty() && name.bytes().all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b"._-".contains(&b))
}

/// The key directory. It lives inside the hc home, which sandboxed agents never see.
#[derive(Debug, Clone)]
pub struct Keyring {
    dir: PathBuf,
}

impl Keyring {
    pub fn new(dir: impl Into<PathBuf>) -> Self {
        Self { dir: dir.into() }
    }

    pub fn allowed_signers(&self) -> PathBuf {
        self.dir.join("allowed_signers")
    }

    pub fn dir(&self) -> &Path {
        &self.dir
    }

    /// Load the identity's key, generating an ed25519 key pair on first use.
    pub fn ensure(&self, role: Role, name: &str) -> Result<Identity, IdentityError> {
        if !valid_name(name) {
            return Err(IdentityError::Name(name.into()));
        }
        let principal = principal(role, name);
        let slug = principal.replace(['@', ':'], "_");
        let key_path = self.dir.join(&slug).join("id_ed25519");
        if !key_path.exists() {
            std::fs::create_dir_all(key_path.parent().expect("key dir"))?;
            let out = Command::new("ssh-keygen")
                .args(["-q", "-t", "ed25519", "-N", "", "-C", &principal, "-f"])
                .arg(&key_path)
                .output()
                .map_err(|e| {
                    if e.kind() == std::io::ErrorKind::NotFound {
                        IdentityError::Keygen("not found on PATH (install OpenSSH)".into())
                    } else {
                        e.into()
                    }
                })?;
            if !out.status.success() {
                return Err(IdentityError::Keygen(String::from_utf8_lossy(&out.stderr).into_owned()));
            }
            self.rewrite_allowed_signers()?;
        }
        Ok(Identity { role, name: name.to_owned(), key_path })
    }

    fn rewrite_allowed_signers(&self) -> Result<(), IdentityError> {
        let mut entries = Vec::new();
        for dir in std::fs::read_dir(&self.dir)? {
            let pub_path = dir?.path().join("id_ed25519.pub");
            if let Ok(text) = std::fs::read_to_string(&pub_path) {
                let mut parts = text.split_whitespace();
                if let (Some(kind), Some(key), Some(principal)) = (parts.next(), parts.next(), parts.next()) {
                    entries.push((principal.to_owned(), format!("{kind} {key}")));
                }
            }
        }
        entries.sort();
        let mut file = String::new();
        for (principal, key) in entries {
            let _ = writeln!(file, "{principal} namespaces=\"git\" {key}");
        }
        std::fs::write(self.allowed_signers(), file)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn principals_round_trip() {
        assert_eq!(principal(Role::Agent, "claude-code"), "agent:claude-code@hc");
        assert_eq!(parse_principal("kernel:local@hc").unwrap(), (Role::Kernel, "local".into()));
        assert!(parse_principal("agent:x").is_err());
    }

    #[test]
    fn generates_key_once_and_registers_it() {
        let dir = tempfile::tempdir().unwrap();
        let ring = Keyring::new(dir.path());
        let a = ring.ensure(Role::Agent, "codex").unwrap();
        let first = std::fs::read(&a.key_path).unwrap();
        let again = ring.ensure(Role::Agent, "codex").unwrap();
        assert_eq!(std::fs::read(&again.key_path).unwrap(), first);
        let signers = std::fs::read_to_string(ring.allowed_signers()).unwrap();
        assert!(signers.starts_with("agent:codex@hc namespaces=\"git\" ssh-ed25519 "));
        assert!(ring.ensure(Role::Agent, "Bad Name").is_err());
    }
}
