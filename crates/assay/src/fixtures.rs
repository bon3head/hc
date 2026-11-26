//! Toy Python repositories with one arithmetic bug, built with pinned git
//! identities and dates so every commit id is reproducible.

use std::fmt::Write as _;
use std::path::Path;

use bedrock::git::{Git, GitError};

pub const TEST_COMMAND: &str = "python3 -m unittest discover -s tests -t .";

/// `((a, b), expected)`.
pub type Case = ((i32, i32), i32);

/// (operation, correct expression, buggy expression, cases).
pub type OpSpec = (&'static str, &'static str, &'static str, [Case; 3]);

pub const OPS: [OpSpec; 3] = [
    ("add", "a + b", "a - b", [((2, 3), 5), ((4, 5), 9), ((1, 7), 8)]),
    ("sub", "a - b", "a + b", [((2, 3), -1), ((4, 5), -1), ((1, 7), -6)]),
    ("mul", "a * b", "a + b", [((2, 3), 6), ((4, 5), 20), ((1, 7), 7)]),
];

#[derive(Debug, Clone, Copy)]
pub struct Op(pub usize);

impl Op {
    pub fn name(self) -> &'static str {
        OPS[self.0].0
    }

    pub fn calc(self, buggy: bool) -> String {
        let (name, good, bad, _) = OPS[self.0];
        format!("def {name}(a, b):\n    return {}\n", if buggy { bad } else { good })
    }

    /// A new test that pins the correct behaviour on an input the suite does not cover.
    pub fn regression_test(self) -> String {
        let (name, good, _, _) = OPS[self.0];
        let expected = eval_expr(good, 10, 4);
        format!(
            "import unittest\n\nfrom src.calc import {name}\n\n\nclass TestRegression(unittest.TestCase):\n    def test_{name}_regression(self):\n        self.assertEqual({name}(10, 4), {expected})\n"
        )
    }

    /// The test file; `buggy_oracle` writes expectations that match the bug.
    pub fn tests(self, buggy_oracle: bool) -> String {
        let (name, _, bad, cases) = OPS[self.0];
        let mut src = format!("import unittest\n\nfrom src.calc import {name}\n\n\nclass TestCalc(unittest.TestCase):\n");
        for (i, ((a, b), expected)) in cases.iter().enumerate() {
            let value = if buggy_oracle { eval_expr(bad, *a, *b) } else { *expected };
            let _ = write!(src, "    def test_case_{i}(self):\n        self.assertEqual({name}({a}, {b}), {value})\n\n");
        }
        src.trim_end().to_owned() + "\n"
    }
}

fn eval_expr(expr: &str, a: i32, b: i32) -> i32 {
    match expr {
        "a - b" => a - b,
        "a + b" => a + b,
        _ => a * b,
    }
}

pub fn repo_git(repo: &Path) -> Git {
    Git::at(repo)
        .env("GIT_AUTHOR_NAME", "assay")
        .env("GIT_AUTHOR_EMAIL", "assay@hc.invalid")
        .env("GIT_COMMITTER_NAME", "assay")
        .env("GIT_COMMITTER_EMAIL", "assay@hc.invalid")
        .env("GIT_AUTHOR_DATE", "2026-01-01T00:00:00Z")
        .env("GIT_COMMITTER_DATE", "2026-01-01T00:00:00Z")
}

pub fn commit(repo: &Path, message: &str) -> Result<String, GitError> {
    let git = repo_git(repo);
    git.run(&["add", "--all"])?;
    git.run(&["commit", "--quiet", "--allow-empty", "--no-verify", "-m", message])?;
    git.run(&["rev-parse", "HEAD"])
}

pub fn write(repo: &Path, rel: &str, content: &str) -> std::io::Result<()> {
    let path = repo.join(rel);
    std::fs::create_dir_all(path.parent().expect("file has a parent"))?;
    std::fs::write(path, content)
}

pub fn make_repo(repo: &Path, op: Op, buggy: bool) -> Result<String, GitError> {
    std::fs::create_dir_all(repo)?;
    write(repo, "src/__init__.py", "")?;
    write(repo, "src/calc.py", &op.calc(buggy))?;
    write(repo, "src/util.py", "def clamp(x, lo, hi):\n    return max(lo, min(x, hi))\n")?;
    write(repo, "tests/__init__.py", "")?;
    write(repo, "tests/test_calc.py", &op.tests(false))?;
    write(repo, ".gitignore", "__pycache__/\n*.pyc\n")?;
    Git::default().run(&["init", "--quiet", "--initial-branch=main", repo.to_str().expect("utf-8")])?;
    commit(repo, "base")
}
