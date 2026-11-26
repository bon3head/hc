//! Changes between two snapshot trees, from `git diff-tree` and `git diff -U0`.

use std::path::Path;

use bedrock::git::{Git, GitError};
use bedrock::ir::Oid;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Change {
    pub path: String,
    pub before: Option<Oid>,
    pub after: Option<Oid>,
    pub added_lines: Vec<u32>,
}

const NULL_OID: &str = "0000000000000000000000000000000000000000";

pub fn changes(ledger_git_dir: &Path, before: &str, after: &str) -> Result<Vec<Change>, GitError> {
    let git = Git::bare(ledger_git_dir);
    let raw = git.run(&["diff-tree", "-r", "-z", "--no-renames", "--no-ext-diff", before, after])?;
    let mut fields = raw.split('\0').filter(|s| !s.is_empty());
    let mut out = Vec::new();
    while let (Some(meta), Some(path)) = (fields.next(), fields.next()) {
        let parts: Vec<&str> = meta.trim_start_matches(':').split(' ').collect();
        let oid = |s: &str| (s != NULL_OID).then(|| s.to_owned());
        let (old, new) = (oid(parts[2]), oid(parts[3]));
        let added_lines = match &new {
            Some(_) => added_lines(&git, before, after, path)?,
            None => Vec::new(),
        };
        out.push(Change { path: path.to_owned(), before: old, after: new, added_lines });
    }
    out.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(out)
}

fn added_lines(git: &Git, before: &str, after: &str, path: &str) -> Result<Vec<u32>, GitError> {
    let text = git.run(&["diff", "-U0", "--no-color", "--no-ext-diff", "--no-renames", before, after, "--", path])?;
    Ok(text.lines().filter_map(parse_hunk_new_range).flatten().collect())
}

/// `@@ -a,b +c,d @@` → lines c..c+d of the new file (d defaults to 1).
fn parse_hunk_new_range(line: &str) -> Option<std::ops::Range<u32>> {
    let rest = line.strip_prefix("@@ ")?;
    let new = rest.split_whitespace().nth(1)?.strip_prefix('+')?;
    let (start, len) = match new.split_once(',') {
        Some((s, l)) => (s.parse().ok()?, l.parse().ok()?),
        None => (new.parse().ok()?, 1),
    };
    Some(start..start + len)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::snapshot::snapshot;

    #[test]
    fn hunk_header_ranges() {
        assert_eq!(parse_hunk_new_range("@@ -3,0 +4,2 @@ def f():"), Some(4..6));
        assert_eq!(parse_hunk_new_range("@@ -1 +1 @@"), Some(1..2));
        assert_eq!(parse_hunk_new_range("@@ -5,2 +4,0 @@"), Some(4..4));
        assert_eq!(parse_hunk_new_range("+++ b/x"), None);
    }

    #[test]
    fn reports_modified_added_and_deleted_files() {
        let tmp = tempfile::tempdir().unwrap();
        let ledger = tmp.path().join("l.git");
        Git::init_bare(&ledger).unwrap();
        let wt = tmp.path().join("wt");
        std::fs::create_dir_all(&wt).unwrap();
        std::fs::write(wt.join("keep.py"), "a\nb\nc\n").unwrap();
        std::fs::write(wt.join("gone.py"), "x\n").unwrap();
        let t0 = snapshot(&ledger, &wt, &[]).unwrap();
        std::fs::write(wt.join("keep.py"), "a\nB\nc\nd\n").unwrap();
        std::fs::remove_file(wt.join("gone.py")).unwrap();
        std::fs::write(wt.join("new.py"), "n\n").unwrap();
        let t1 = snapshot(&ledger, &wt, &[]).unwrap();
        let ch = changes(&ledger, &t0, &t1).unwrap();
        let summary: Vec<_> = ch.iter().map(|c| (c.path.as_str(), c.before.is_some(), c.after.is_some(), c.added_lines.clone())).collect();
        assert_eq!(summary, [("gone.py", true, false, vec![]), ("keep.py", true, true, vec![2, 4]), ("new.py", false, true, vec![1])]);
    }
}
