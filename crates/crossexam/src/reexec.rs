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
    /// Why the command never ran (runner missing or not executable); no exit code means anything then.
    pub unavailable: Option<String>,
    /// A run whose exit code contradicts its own test report (see `report_mismatch`).
    pub mismatch: Option<String>,
}

#[derive(Debug, Clone, Copy)]
pub enum Source<'a> {
    Commit { repo: &'a Path, commit: &'a str, ledger: &'a Path },
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
            Source::Commit { repo, commit, ledger } => {
                // Never check out inside the agent's repository: a checkout runs its hooks and
                // filters. Resolve the commit read-only, then materialize through hc's ledger
                // with the agent's object directory as an alternate.
                let agent = Git::untrusted(repo);
                let Some(tree) = agent.try_run(&["rev-parse", "--verify", "--quiet", "--end-of-options", &format!("{commit}^{{tree}}")])?
                else {
                    return Ok(Run {
                        exit_codes: vec![],
                        passed: None,
                        failed: None,
                        unavailable: Some(format!("commit {commit:?} is not in the repository")),
                        mismatch: None,
                    });
                };
                let objects = agent.run(&["rev-parse", "--path-format=absolute", "--git-path", "objects"])?;
                std::fs::create_dir_all(&dir)?;
                exhibit::snapshot::materialize_from(ledger, &tree, &dir, Some(Path::new(&objects)))?;
            }
            Source::Tree { docket_git, tree } => {
                std::fs::create_dir_all(&dir)?;
                exhibit::snapshot::materialize(docket_git, tree, &dir)?;
            }
        }
        if let Some(why) = escaping_symlink(&dir) {
            return Ok(Run { exit_codes: vec![], passed: None, failed: None, unavailable: Some(why), mismatch: None });
        }
        let mut exit_codes = Vec::new();
        let mut counts = (None, None);
        let mut unavailable = None;
        let mut mismatch = None;
        for _ in 0..self.runs.max(1) {
            // Python must not import the user's site-packages (a `.pth` file there runs at startup).
            // Variables that make Python load code from outside the tree are dropped as well.
            let out = match self
                .sandbox
                .command(argv, &dir)
                .env("PYTHONDONTWRITEBYTECODE", "1")
                .env("PYTHONNOUSERSITE", "1")
                .env_remove("PYTHONPATH")
                .env_remove("PYTHONSTARTUP")
                .env_remove("PYTHONHOME")
                .env_remove("PYTHONINSPECT")
                .output()
            {
                Ok(out) => out,
                Err(e) if matches!(e.kind(), std::io::ErrorKind::NotFound | std::io::ErrorKind::PermissionDenied) => {
                    unavailable = Some(format!("`{}` could not start: {e}", argv[0]));
                    break;
                }
                Err(e) => return Err(e.into()),
            };
            // bwrap reports an exec failure of its child on stderr with exit 1; 126/127 are the shell conventions.
            let stderr = String::from_utf8_lossy(&out.stderr);
            if matches!(out.status.code(), Some(126 | 127)) || stderr.starts_with("bwrap: execvp") {
                unavailable = Some(format!("`{}` could not start: {}", argv[0], stderr.lines().next().unwrap_or("not found")));
                break;
            }
            exit_codes.push(out.status.code().unwrap_or(-1));
            let text = format!("{}\n{}", String::from_utf8_lossy(&out.stdout), String::from_utf8_lossy(&out.stderr));
            counts = parse_counts(&text);
            mismatch = mismatch.or_else(|| report_mismatch(out.status.code().unwrap_or(-1), counts));
        }
        Ok(Run { exit_codes, passed: counts.0, failed: counts.1, unavailable, mismatch })
    }
}

/// Paths that `commit` changes relative to `base` (a commit or tree id). Nothing runs
/// in the agent's repository except read-only `rev-parse`; the diff is computed in
/// hc's ledger with the agent's object directory as an alternate. `None` when either
/// side cannot be resolved. `filter` is a `--diff-filter` letter set.
pub fn commit_changes(ledger: &Path, repo: &Path, base: &str, commit: &str, filter: &str) -> Result<Option<Vec<String>>, GitError> {
    let agent = Git::untrusted(repo);
    let Some(tree) = agent.try_run(&["rev-parse", "--verify", "--quiet", "--end-of-options", &format!("{commit}^{{tree}}")])? else {
        return Ok(None);
    };
    let objects = agent.run(&["rev-parse", "--path-format=absolute", "--git-path", "objects"])?;
    let git = Git::bare(ledger).env("GIT_ALTERNATE_OBJECT_DIRECTORIES", &objects);
    let out =
        git.output(&["diff-tree", "-r", "--no-renames", "--name-only", "-z", &format!("--diff-filter={filter}"), base, &tree], None)?;
    if !out.status.success() {
        return Ok(None);
    }
    Ok(Some(String::from_utf8_lossy(&out.stdout).split('\0').filter(|s| !s.is_empty()).map(str::to_owned).collect()))
}

static UNITTEST_RAN: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"Ran (\d+) tests? in").unwrap());
static UNITTEST_FAILED: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"FAILED \(([^)]*)\)").unwrap());
static UNITTEST_PART: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?:failures|errors)=(\d+)").unwrap());
static PYTEST: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(\d+) (passed|failed|error)").unwrap());
static CARGO: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"test result: \w+\. (\d+) passed; (\d+) failed").unwrap());

/// (passed, failed) as reported by unittest, pytest or cargo test; `None` when absent.
/// A symlink in the materialized tree whose target lies outside it: the code that
/// would run is then not the code the ledger recorded, so nothing it does is evidence.
fn escaping_symlink(root: &Path) -> Option<String> {
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        for entry in std::fs::read_dir(&dir).ok()?.flatten() {
            let path = entry.path();
            let Ok(meta) = std::fs::symlink_metadata(&path) else { continue };
            if meta.file_type().is_symlink() {
                let target = std::fs::read_link(&path).ok()?;
                // Depth of the link's directory below the root; leaving the root makes it negative.
                let mut depth = i64::try_from(path.strip_prefix(root).ok()?.components().count()).unwrap_or(i64::MAX) - 1;
                let escapes = target.is_absolute()
                    || target.components().any(|c| {
                        match c {
                            std::path::Component::ParentDir => depth -= 1,
                            std::path::Component::Normal(_) => depth += 1,
                            _ => {}
                        }
                        depth < 0
                    });
                if escapes {
                    let rel = path.strip_prefix(root).unwrap_or(&path).display().to_string();
                    return Some(format!("{rel} is a symlink to {} outside the recorded tree", target.display()));
                }
            } else if meta.is_dir() {
                stack.push(path);
            }
        }
    }
    None
}

/// The process exit code and the test report it printed must agree: exit 0 with
/// failures reported, or a non-zero exit with only passes reported, means the exit
/// code was forced (`sys.exit(0)`, an `atexit` hook, a patched runner) or the report
/// was. `None` when there is no report or nothing ran.
pub fn report_mismatch(exit: i32, counts: (Option<u64>, Option<u64>)) -> Option<String> {
    match counts {
        (_, Some(failed)) if exit == 0 && failed > 0 => Some(format!("exit 0 but the report shows {failed} failed")),
        (Some(passed), Some(0)) if exit != 0 && passed > 0 => {
            Some(format!("exit {exit} but the report shows {passed} passed and 0 failed"))
        }
        _ => None,
    }
}

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
    use super::{parse_counts, report_mismatch};

    #[test]
    fn exit_code_and_report_must_agree() {
        assert_eq!(report_mismatch(0, (Some(22), Some(1))).as_deref(), Some("exit 0 but the report shows 1 failed"));
        assert_eq!(report_mismatch(1, (Some(23), Some(0))).as_deref(), Some("exit 1 but the report shows 23 passed and 0 failed"));
        assert_eq!(report_mismatch(0, (Some(23), Some(0))), None);
        assert_eq!(report_mismatch(1, (Some(22), Some(1))), None);
        assert_eq!(report_mismatch(0, (None, None)), None, "no report: nothing to compare");
        assert_eq!(report_mismatch(1, (Some(0), Some(0))), None, "nothing ran: an import error, not a mismatch");
    }

    #[test]
    fn counts() {
        assert_eq!(parse_counts("...\nRan 3 tests in 0.001s\n\nOK\n"), (Some(3), Some(0)));
        assert_eq!(parse_counts("Ran 3 tests in 0.002s\n\nFAILED (failures=1, errors=1)\n"), (Some(1), Some(2)));
        assert_eq!(parse_counts("===== 4 passed, 1 failed in 0.1s ====="), (Some(4), Some(1)));
        assert_eq!(parse_counts("test result: ok. 7 passed; 0 failed; 0 ignored"), (Some(7), Some(0)));
        assert_eq!(parse_counts("nothing here"), (None, None));
    }
}
