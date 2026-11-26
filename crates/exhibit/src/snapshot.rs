//! Snapshot a working tree into a git tree object stored in the ledger repo.
//!
//! `git add -A` through a temporary `GIT_INDEX_FILE` stages every tracked and
//! untracked file that `.gitignore` does not exclude. Files matching protected
//! patterns are then force-added even when ignored, because hook and agent
//! configuration (`.claude/settings.local.json` and friends) is commonly
//! gitignored and is exactly what a gate-disabling agent would edit.

use std::path::Path;

use bedrock::git::{Git, GitError};
use bedrock::ir::Oid;

/// Bytecode and test-runner caches: produced by running the code, never authored.
/// Excluding them keeps an honest test run from looking like a write to `tests/`,
/// and re-execution compiles from source (`PYTHONDONTWRITEBYTECODE`), so nothing
/// a cache could hide reaches a verdict.
const DERIVED: [&str; 3] = [":(glob)**/__pycache__/**", ":(glob)**/*.py[co]", ":(glob)**/.pytest_cache/**"];

fn derived(path: &str) -> bool {
    // Python writes only lowercase `.pyc`/`.pyo`, and the pathspec above matches the same case.
    path.split('/').any(|p| p == "__pycache__" || p == ".pytest_cache")
        || Path::new(path).extension().is_some_and(|e| e == "pyc" || e == "pyo")
}

pub fn snapshot(ledger_git_dir: &Path, worktree: &Path, force_patterns: &[String]) -> Result<Oid, GitError> {
    let index = tempfile::NamedTempFile::new()?;
    let index_path = index.path().to_path_buf();
    drop(index);
    let git = Git::bare(ledger_git_dir).work_tree(worktree).env("GIT_INDEX_FILE", &index_path).env("GIT_DIR", ledger_git_dir);
    let staged = (|| {
        git.run(&["add", "--all", "--", "."])?;
        let mut rm = vec!["rm", "--cached", "-r", "-q", "--ignore-unmatch", "--"];
        rm.extend(DERIVED);
        git.run(&rm)?;
        if !force_patterns.is_empty() {
            let mut args = vec!["ls-files", "-z", "--others", "--ignored", "--exclude-standard", "--"];
            let specs: Vec<String> = force_patterns.iter().map(|p| format!(":(glob){p}")).collect();
            args.extend(specs.iter().map(String::as_str));
            let ignored = git.run(&args)?;
            let files: Vec<&str> = ignored.split('\0').filter(|s| !s.is_empty() && !derived(s)).collect();
            if !files.is_empty() {
                let mut add = vec!["add", "--force", "--"];
                add.extend(files);
                git.run(&add)?;
            }
        }
        git.run(&["write-tree"])
    })();
    let _ = std::fs::remove_file(&index_path);
    staged
}

/// `base` with `paths` taken from `from` (paths absent in `from` are removed).
pub fn overlay(ledger_git_dir: &Path, base: &str, from: &str, paths: &[String]) -> Result<Oid, GitError> {
    let index = tempfile::NamedTempFile::new()?;
    let index_path = index.path().to_path_buf();
    drop(index);
    let git = Git::bare(ledger_git_dir).env("GIT_INDEX_FILE", &index_path);
    let result = (|| {
        git.run(&["read-tree", base])?;
        for path in paths {
            let entry = git.run(&["ls-tree", from, "--", path])?;
            match entry.split_once('\t').map(|(meta, _)| meta.split(' ').collect::<Vec<_>>()) {
                Some(meta) if meta.len() == 3 => {
                    git.run(&["update-index", "--add", "--cacheinfo", &format!("{},{},{path}", meta[0], meta[2])])?;
                }
                _ => {
                    git.run(&["update-index", "--force-remove", "--", path])?;
                }
            }
        }
        git.run(&["write-tree"])
    })();
    let _ = std::fs::remove_file(&index_path);
    result
}

/// Write the files of `tree` into the empty directory `dest`.
pub fn materialize(ledger_git_dir: &Path, tree: &str, dest: &Path) -> Result<(), GitError> {
    materialize_from(ledger_git_dir, tree, dest, None)
}

/// Like `materialize`, reading objects also from `alternate` (another
/// repository's object directory) without running any git command in that
/// repository: its config, hooks and filters never apply. `checkout-index`
/// refuses paths with `..` or `.git` components.
pub fn materialize_from(ledger_git_dir: &Path, tree: &str, dest: &Path, alternate: Option<&Path>) -> Result<(), GitError> {
    let index = tempfile::NamedTempFile::new()?;
    let index_path = index.path().to_path_buf();
    drop(index);
    let mut git = Git::bare(ledger_git_dir).work_tree(dest).env("GIT_INDEX_FILE", &index_path);
    if let Some(objects) = alternate {
        git = git.env("GIT_ALTERNATE_OBJECT_DIRECTORIES", objects);
    }
    let result = git.run(&["read-tree", tree]).and_then(|_| git.run(&["checkout-index", "--all", "--force"]));
    let _ = std::fs::remove_file(&index_path);
    result.map(|_| ())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn snapshots_untracked_and_forced_ignored_files_without_touching_the_worktree_repo() {
        let tmp = tempfile::tempdir().unwrap();
        let ledger = tmp.path().join("ledger.git");
        Git::init_bare(&ledger).unwrap();
        let wt = tmp.path().join("wt");
        std::fs::create_dir_all(wt.join(".claude")).unwrap();
        Git::default().run(&["init", "-q", wt.to_str().unwrap()]).unwrap();
        std::fs::write(wt.join(".gitignore"), ".claude/\nbuild/\n").unwrap();
        std::fs::write(wt.join("a.py"), "x = 1\n").unwrap();
        std::fs::write(wt.join(".claude/settings.local.json"), "{}").unwrap();
        std::fs::create_dir_all(wt.join("build")).unwrap();
        std::fs::write(wt.join("build/out.bin"), "junk").unwrap();

        let tree = snapshot(&ledger, &wt, &[".claude/**".into()]).unwrap();
        let listing = Git::bare(&ledger).run(&["ls-tree", "-r", "--name-only", &tree]).unwrap();
        assert_eq!(listing.lines().collect::<Vec<_>>(), [".claude/settings.local.json", ".gitignore", "a.py"]);
        assert_eq!(Git::at(&wt).run(&["status", "--porcelain"]).unwrap().lines().count(), 2, "worktree index untouched");

        let out = tmp.path().join("out");
        std::fs::create_dir_all(&out).unwrap();
        materialize(&ledger, &tree, &out).unwrap();
        assert_eq!(std::fs::read_to_string(out.join("a.py")).unwrap(), "x = 1\n");
    }
}
