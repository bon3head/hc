//! P8: run the task's test command again, outside the agent's reach, on
//! exactly the code the claim is about: the named commit (via a detached
//! `git worktree`) or the session's final observed tree (materialized from the
//! docket's object database).

use std::path::Path;
use std::sync::LazyLock;

use bedrock::git::{Git, GitError};
use cell::Sandbox;
use regex::Regex;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Run {
    pub exit_codes: Vec<i32>,
    pub passed: Option<u64>,
    pub failed: Option<u64>,
}

#[derive(Debug, Clone, Copy)]
pub enum Source<'a> {
    Commit { repo: &'a Path, commit: &'a str },
    Tree { docket_git: &'a Path, tree: &'a str },
}

#[derive(Debug, Clone)]
pub struct Reexec {
    pub sandbox: Sandbox,
    pub runs: u32,
}

impl Reexec {
    pub fn run(&self, source: Source<'_>, argv: &[String]) -> Result<Run, GitError> {
        let tmp = tempfile::tempdir()?;
        let dir = tmp.path().join("wt");
        match source {
            Source::Commit { repo, commit } => {
                Git::at(repo).run(&["worktree", "add", "--detach", "--quiet", dir.to_str().expect("utf-8"), commit])?;
            }
            Source::Tree { docket_git, tree } => {
                std::fs::create_dir_all(&dir)?;
                exhibit::snapshot::materialize(docket_git, tree, &dir)?;
            }
        }
        let mut exit_codes = Vec::new();
        let mut counts = (None, None);
        for _ in 0..self.runs.max(1) {
            let out = self.sandbox.command(argv, &dir).env("PYTHONDONTWRITEBYTECODE", "1").output()?;
            exit_codes.push(out.status.code().unwrap_or(-1));
            let text = format!("{}\n{}", String::from_utf8_lossy(&out.stdout), String::from_utf8_lossy(&out.stderr));
            counts = parse_counts(&text);
        }
        if let Source::Commit { repo, .. } = source {
            let _ = Git::at(repo).run(&["worktree", "remove", "--force", dir.to_str().expect("utf-8")]);
        }
        Ok(Run { exit_codes, passed: counts.0, failed: counts.1 })
    }
}

static UNITTEST_RAN: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"Ran (\d+) tests? in").unwrap());
static UNITTEST_FAILED: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"FAILED \(([^)]*)\)").unwrap());
static UNITTEST_PART: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?:failures|errors)=(\d+)").unwrap());
static PYTEST: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(\d+) (passed|failed|error)").unwrap());
static CARGO: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"test result: \w+\. (\d+) passed; (\d+) failed").unwrap());

/// (passed, failed) as reported by unittest, pytest or cargo test; `None` when absent.
pub fn parse_counts(output: &str) -> (Option<u64>, Option<u64>) {
    let num = |s: &str| s.parse::<u64>().unwrap_or(0);
    if let Some(ran) = UNITTEST_RAN.captures(output) {
        let total = num(&ran[1]);
        let failed: u64 = UNITTEST_FAILED.captures(output).map_or(0, |c| UNITTEST_PART.captures_iter(&c[1]).map(|p| num(&p[1])).sum());
        return (Some(total.saturating_sub(failed)), Some(failed));
    }
    let cargo: Vec<_> = CARGO.captures_iter(output).collect();
    if !cargo.is_empty() {
        return (Some(cargo.iter().map(|c| num(&c[1])).sum()), Some(cargo.iter().map(|c| num(&c[2])).sum()));
    }
    let (mut passed, mut failed, mut seen) = (0, 0, false);
    for c in PYTEST.captures_iter(output) {
        seen = true;
        if &c[2] == "passed" { passed += num(&c[1]) } else { failed += num(&c[1]) }
    }
    if seen { (Some(passed), Some(failed)) } else { (None, None) }
}

#[cfg(test)]
mod tests {
    use super::parse_counts;

    #[test]
    fn counts() {
        assert_eq!(parse_counts("...\nRan 3 tests in 0.001s\n\nOK\n"), (Some(3), Some(0)));
        assert_eq!(parse_counts("Ran 3 tests in 0.002s\n\nFAILED (failures=1, errors=1)\n"), (Some(1), Some(2)));
        assert_eq!(parse_counts("===== 4 passed, 1 failed in 0.1s ====="), (Some(4), Some(1)));
        assert_eq!(parse_counts("test result: ok. 7 passed; 0 failed; 0 ignored"), (Some(7), Some(0)));
        assert_eq!(parse_counts("nothing here"), (None, None));
    }
}
