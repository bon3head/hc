//! A thin, hermetic wrapper over the git CLI.
//!
//! Global and system git configuration are disabled for every call, so a
//! user's (or a sandbox's) signing program, hooks or aliases can never change
//! what the ledger records or how it is verified.

use std::ffi::OsString;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

#[derive(Debug, thiserror::Error)]
pub enum GitError {
    #[error("failed to spawn git: {0}")]
    Spawn(#[from] std::io::Error),
    #[error("git {args} exited with {code:?}: {stderr}")]
    Failed { args: String, code: Option<i32>, stderr: String },
    #[error("git output was not UTF-8")]
    Utf8,
}

#[derive(Debug, Clone, Default)]
pub struct Git {
    repo_dir: Option<PathBuf>,
    work_tree: Option<PathBuf>,
    cwd: Option<PathBuf>,
    env: Vec<(OsString, OsString)>,
    config: Vec<String>,
}

impl Git {
    /// Operate on a repository whose working directory is `dir`.
    pub fn at(dir: impl Into<PathBuf>) -> Self {
        Self { cwd: Some(dir.into()), ..Self::default() }
    }

    /// Operate on a bare repository (or an explicit git dir).
    pub fn bare(git_dir: impl Into<PathBuf>) -> Self {
        Self { repo_dir: Some(git_dir.into()), ..Self::default() }
    }

    #[must_use]
    pub fn work_tree(mut self, dir: impl Into<PathBuf>) -> Self {
        self.work_tree = Some(dir.into());
        self
    }

    #[must_use]
    pub fn env(mut self, key: impl Into<OsString>, value: impl Into<OsString>) -> Self {
        self.env.push((key.into(), value.into()));
        self
    }

    #[must_use]
    pub fn config(mut self, key_value: impl Into<String>) -> Self {
        self.config.push(key_value.into());
        self
    }

    pub fn git_dir(&self) -> Option<&Path> {
        self.repo_dir.as_deref()
    }

    fn command(&self, args: &[&str]) -> Command {
        let mut cmd = Command::new("git");
        cmd.env("GIT_CONFIG_GLOBAL", "/dev/null")
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .env("GIT_TERMINAL_PROMPT", "0")
            .env_remove("GIT_DIR")
            .env_remove("GIT_WORK_TREE")
            .env_remove("GIT_INDEX_FILE");
        if let Some(cwd) = &self.cwd {
            cmd.current_dir(cwd);
        }
        if let Some(dir) = &self.repo_dir {
            cmd.arg("--git-dir").arg(dir);
        }
        if let Some(tree) = &self.work_tree {
            cmd.arg("--work-tree").arg(tree);
        }
        for kv in &self.config {
            cmd.arg("-c").arg(kv);
        }
        for (k, v) in &self.env {
            cmd.env(k, v);
        }
        cmd.args(args);
        cmd
    }

    pub fn output(&self, args: &[&str], stdin: Option<&[u8]>) -> Result<Output, GitError> {
        let mut cmd = self.command(args);
        cmd.stdin(if stdin.is_some() { Stdio::piped() } else { Stdio::null() })
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        let mut child = cmd.spawn()?;
        if let Some(input) = stdin {
            child.stdin.take().expect("piped stdin").write_all(input)?;
        }
        Ok(child.wait_with_output()?)
    }

    /// Run and return trimmed stdout, failing on a non-zero exit.
    pub fn run(&self, args: &[&str]) -> Result<String, GitError> {
        self.run_with_input(args, None)
    }

    pub fn run_with_input(&self, args: &[&str], stdin: Option<&[u8]>) -> Result<String, GitError> {
        let out = self.output(args, stdin)?;
        if !out.status.success() {
            return Err(GitError::Failed {
                args: args.join(" "),
                code: out.status.code(),
                stderr: String::from_utf8_lossy(&out.stderr).trim().to_owned(),
            });
        }
        let text = String::from_utf8(out.stdout).map_err(|_| GitError::Utf8)?;
        Ok(text.trim_end_matches('\n').to_owned())
    }

    /// Like `run`, but a non-zero exit yields `None` (for `rev-parse --verify -q`).
    pub fn try_run(&self, args: &[&str]) -> Result<Option<String>, GitError> {
        let out = self.output(args, None)?;
        if !out.status.success() {
            return Ok(None);
        }
        Ok(Some(String::from_utf8(out.stdout).map_err(|_| GitError::Utf8)?.trim().to_owned()))
    }

    pub fn init_bare(dir: &Path) -> Result<Self, GitError> {
        if !dir.join("HEAD").exists() {
            std::fs::create_dir_all(dir)?;
            Self::default().run(&["init", "--quiet", "--bare", dir.to_str().expect("utf-8 path")])?;
        }
        Ok(Self::bare(dir))
    }
}
