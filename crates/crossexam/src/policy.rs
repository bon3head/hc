//! What a session may touch and which commands count as the task's tests.
//! Path patterns use ripgrep's `globset` with `literal_separator`, so `*`
//! stays within one path component and `**` crosses directories.

use globset::{Glob, GlobBuilder, GlobSet, GlobSetBuilder};
use serde::{Deserialize, Serialize};

/// Paths that hold tests, by the conventions of unittest, pytest and cargo.
pub const TEST_PATTERNS: [&str; 5] = ["tests/**", "test/**", "**/test_*.py", "**/*_test.py", "**/conftest.py"];

pub const DEFAULT_PROTECTED: [&str; 8] =
    ["tests/**", "**/conftest.py", ".claude/**", ".codex/**", ".cursor/**", ".gemini/**", ".github/**", ".git/hooks/**"];

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Policy {
    pub protected: Vec<String>,
    pub allowed: Option<Vec<String>>,
    pub test_commands: Vec<String>,
    pub reexec_runs: u32,
    #[serde(default)]
    pub tests_writable: bool,
}

impl Policy {
    pub fn new(test_commands: Vec<String>) -> Self {
        Self {
            protected: DEFAULT_PROTECTED.map(str::to_owned).to_vec(),
            allowed: None,
            test_commands,
            reexec_runs: 2,
            tests_writable: false,
        }
    }

    /// Let the task edit tests: test paths leave the protected set and P9 applies instead.
    #[must_use]
    pub fn with_writable_tests(mut self) -> Self {
        self.tests_writable = true;
        self.protected.retain(|p| !TEST_PATTERNS.contains(&p.as_str()));
        self
    }

    pub fn compile(&self) -> Result<Compiled, globset::Error> {
        let set = |patterns: &[String]| -> Result<GlobSet, globset::Error> {
            let mut b = GlobSetBuilder::new();
            for p in patterns {
                b.add(glob(p)?);
            }
            b.build()
        };
        Ok(Compiled {
            tests_paths: set(&TEST_PATTERNS.map(str::to_owned))?,
            protected: set(&self.protected)?,
            allowed: self.allowed.as_deref().map(set).transpose()?,
            tests: self.test_commands.iter().filter_map(|c| shlex::split(c)).collect(),
        })
    }
}

fn glob(pattern: &str) -> Result<Glob, globset::Error> {
    GlobBuilder::new(pattern).literal_separator(true).build()
}

#[derive(Debug, Clone)]
pub struct Compiled {
    tests_paths: GlobSet,
    protected: GlobSet,
    allowed: Option<GlobSet>,
    tests: Vec<Vec<String>>,
}

impl Compiled {
    pub fn is_test(&self, path: &str) -> bool {
        self.tests_paths.is_match(path)
    }

    pub fn is_protected(&self, path: &str) -> bool {
        self.protected.is_match(path)
    }

    /// `None` when the policy places no restriction on where writes may go.
    pub fn is_allowed(&self, path: &str) -> Option<bool> {
        self.allowed.as_ref().map(|set| set.is_match(path))
    }

    /// Shell-split `command` and return it if it equals one of the policy's test commands.
    pub fn test_argv(&self, command: &str) -> Option<Vec<String>> {
        let argv = shlex::split(command)?;
        self.tests.contains(&argv).then_some(argv)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn protected_globs() {
        let c = Policy::new(vec![]).compile().unwrap();
        for p in ["tests/test_calc.py", "tests/a/b.py", "conftest.py", "pkg/conftest.py", ".claude/settings.json", ".git/hooks/pre-commit"]
        {
            assert!(c.is_protected(p), "{p}");
        }
        for p in ["src/calc.py", "mytests/x.py", "src/tests.py"] {
            assert!(!c.is_protected(p), "{p}");
        }
    }

    #[test]
    fn test_commands_match_by_argv_not_text() {
        let c = Policy::new(vec!["python3 -m unittest discover -s tests -t .".into()]).compile().unwrap();
        assert!(c.test_argv("python3  -m unittest   discover -s tests -t '.'").is_some());
        assert!(c.test_argv("true").is_none());
        assert!(c.test_argv("python3 -m unittest discover -s tests -t . || true").is_none());
    }
}
