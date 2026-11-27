//! What changed since the brief's stale results were true: paths, not contents.
//!
//! The stale section says a result "held on an earlier tree"; this says which
//! paths moved since and who the ledger attributes each move to. Contents never
//! enter the brief (an agent could plant text for the next session in a diff),
//! so the reader is pointed at the files instead.

use std::path::Path;

use bedrock::git::GitError;
use bedrock::ir::{Body, Event, Obs};
use docket::Record;

use crate::Finding;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Added,
    Modified,
    Deleted,
}

/// How the ledger saw one write to a path.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Seen {
    /// Recorded while the session was running.
    During(String),
    /// Recorded as the session started: the change was made while no session was running.
    BeforeStart(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Drift {
    pub path: String,
    pub kind: Kind,
    pub changed_lines: usize,
    /// In ledger order. Empty: no session has recorded the change yet.
    pub seen: Vec<Seen>,
}

/// Changes from the tree of the earliest stale verified result to the current tree.
/// Empty when nothing is stale.
pub fn since_stale(docket_git: &Path, records: &[Record], findings: &[Finding]) -> Result<Vec<Drift>, GitError> {
    let anchor = findings
        .iter()
        .filter(|f| f.verified.is_some() && f.claim.claim.commit.is_none())
        .filter_map(|f| match (f.subject_tree.as_deref(), f.subject_seq, f.current_tree.as_deref()) {
            (Some(then), Some(seq), Some(now)) if then != now => Some((seq, then, now)),
            _ => None,
        })
        .min_by_key(|(seq, _, _)| *seq);
    let Some((since, then, now)) = anchor else { return Ok(Vec::new()) };

    // A session's opening snapshot records, under its own name and with the same
    // timestamp as its `session_start`, every change made since the repository was
    // last seen. Those writes are the ones up to the session's first checkpoint.
    // SHORTCUT: a start that found no changes records no checkpoint, so a write
    // recorded in the same second as that start is reported as before-start too;
    // an unconditional opening checkpoint would make this exact.
    let mut writes: Vec<(String, Seen)> = Vec::new();
    let mut starting: std::collections::BTreeMap<&str, &str> = std::collections::BTreeMap::new();
    for r in records {
        let Body::Obs(Obs { session, ts, event }) = &r.body else { continue };
        match event {
            Event::SessionStart { .. } => {
                starting.insert(session, ts);
            }
            Event::FileWrite { path, .. } => {
                let before_start = starting.get(session.as_str()).is_some_and(|t| *t == ts);
                if r.seq > since {
                    let seen = if before_start { Seen::BeforeStart(session.clone()) } else { Seen::During(session.clone()) };
                    writes.push((path.clone(), seen));
                }
            }
            _ => {
                starting.remove(session.as_str());
            }
        }
    }

    Ok(exhibit::diff::changes(docket_git, then, now)?
        .into_iter()
        .map(|c| {
            let mut seen: Vec<Seen> = Vec::new();
            for (_, s) in writes.iter().filter(|(p, _)| *p == c.path) {
                if !seen.contains(s) {
                    seen.push(s.clone());
                }
            }
            let kind = match (&c.before, &c.after) {
                (None, _) => Kind::Added,
                (_, None) => Kind::Deleted,
                _ => Kind::Modified,
            };
            Drift { path: c.path, kind, changed_lines: c.added_lines.len(), seen }
        })
        .collect())
}
