//! chambers: the hc home (`$HC_HOME`, default `~/.hc`): keys, one docket per repository,
//! session outboxes and the user's roster overrides. Sandboxed agents never
//! see this directory; only their own session outbox is bound into the cell.

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use bedrock::Digest;
use bedrock::ir::Role;
use docket::Docket;
use roster::Registry;
use seal::{Identity, Keyring};

#[derive(Debug, Clone)]
pub struct Home {
    root: PathBuf,
}

impl Home {
    pub fn locate() -> Result<Self> {
        let root = match std::env::var_os("HC_HOME") {
            Some(dir) => PathBuf::from(dir),
            None => PathBuf::from(std::env::var_os("HOME").context("neither HC_HOME nor HOME is set")?).join(".hc"),
        };
        std::fs::create_dir_all(&root)?;
        Ok(Self { root })
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn keyring(&self) -> Keyring {
        Keyring::new(self.root.join("keys"))
    }

    pub fn identity(&self, role: Role, name: &str) -> Result<Identity> {
        Ok(self.keyring().ensure(role, name)?)
    }

    /// One docket per repository, keyed by the digest of its canonical path.
    pub fn docket(&self, repo: &Path) -> Result<Docket> {
        Ok(Docket::open(&self.docket_path(repo)?, self.keyring())?)
    }

    pub fn docket_path(&self, repo: &Path) -> Result<PathBuf> {
        let canonical = repo.canonicalize().with_context(|| format!("repository {}", repo.display()))?;
        let id = Digest::of(canonical.to_string_lossy().as_bytes()).hex();
        Ok(self.root.join("repos").join(&id[..16]).join("docket.git"))
    }

    pub fn outbox(&self, session: &str) -> Result<PathBuf> {
        let dir = self.root.join("outbox").join(session);
        std::fs::create_dir_all(&dir)?;
        Ok(dir)
    }

    pub fn registry(&self) -> Result<Registry> {
        Ok(Registry::load(&self.root.join("roster.json"))?)
    }
}
