//! cell: process isolation with bubblewrap (bwrap(1)), the unprivileged sandbox
//! Flatpak uses. The whole filesystem is visible except the hc home, which is
//! covered by an empty tmpfs; only the session outbox is bound back in.

use std::path::{Path, PathBuf};
use std::process::Command;

use bedrock::ir::Isolation;

#[derive(Debug, Clone)]
pub struct Sandbox {
    pub isolation: Isolation,
    hide: PathBuf,
    binds: Vec<PathBuf>,
}

impl Sandbox {
    /// Use bwrap when it is installed and can create a namespace here.
    pub fn detect(hide: &Path) -> Self {
        let works = Command::new("bwrap").args(["--dev-bind", "/", "/", "--", "true"]).output().is_ok_and(|o| o.status.success());
        Self { isolation: if works { Isolation::Bwrap } else { Isolation::None }, hide: hide.to_path_buf(), binds: Vec::new() }
    }

    pub fn none(hide: &Path) -> Self {
        Self { isolation: Isolation::None, hide: hide.to_path_buf(), binds: Vec::new() }
    }

    /// Make `dir` (inside the hidden home) visible and writable inside the sandbox.
    #[must_use]
    pub fn bind(mut self, dir: impl Into<PathBuf>) -> Self {
        self.binds.push(dir.into());
        self
    }

    pub fn command(&self, argv: &[String], cwd: &Path) -> Command {
        match self.isolation {
            Isolation::None => {
                let mut cmd = Command::new(&argv[0]);
                cmd.args(&argv[1..]).current_dir(cwd);
                cmd
            }
            Isolation::Bwrap => {
                let mut cmd = Command::new("bwrap");
                cmd.args(["--dev-bind", "/", "/", "--die-with-parent", "--tmpfs"]).arg(&self.hide);
                for dir in &self.binds {
                    cmd.arg("--bind").arg(dir).arg(dir);
                }
                // bwrap resolves a relative --chdir inside the new namespace, from `/`.
                let cwd = cwd.canonicalize().unwrap_or_else(|_| std::path::absolute(cwd).unwrap_or_else(|_| cwd.to_path_buf()));
                cmd.arg("--chdir").arg(cwd).arg("--").args(argv);
                cmd
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hides_the_hc_home_but_exposes_the_outbox() {
        let tmp = tempfile::tempdir().unwrap();
        let home = tmp.path().join("hc-home");
        let outbox = home.join("outbox/s1");
        std::fs::create_dir_all(&outbox).unwrap();
        std::fs::write(home.join("secret"), "k").unwrap();
        let sandbox = Sandbox::detect(&home).bind(&outbox);
        if sandbox.isolation != Isolation::Bwrap {
            eprintln!("bwrap unavailable; isolation test skipped");
            return;
        }
        let script = format!("test ! -e {0}/secret && echo ok > {1}/claims.json", home.display(), outbox.display());
        let status = sandbox.command(&["sh".into(), "-c".into(), script], tmp.path()).status().unwrap();
        assert!(status.success());
        assert_eq!(std::fs::read_to_string(outbox.join("claims.json")).unwrap(), "ok\n");
        assert!(home.join("secret").exists());
    }
}
