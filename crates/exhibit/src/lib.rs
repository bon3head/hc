//! exhibit: collects the evidence. Runs in the harness, outside the agent's
//! cell, and records what happened as harness-signed docket entries.
//!
//! Snapshots are git tree objects written into the docket repository's object
//! database through a throwaway index, so the agent can neither see nor alter
//! the baseline it is compared against.

pub mod diff;
pub mod snapshot;

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use bedrock::git::Git;
use bedrock::ir::{
    AgentAttestation, ClaimEnvelope, ClaimRecord, Diagnostic, Event, Isolation, Obs, Oid, PREDICATE_CLAIM, PREDICATE_OBS,
    ResourceDescriptor,
};
use docket::{Docket, DocketError};
use seal::Identity;

pub use diff::Change;

/// A session the recorder has opened and not yet closed.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Open {
    pub session: String,
    pub task: String,
    pub worktree: PathBuf,
    pub tree: Oid,
}

const GIT_META_FILE: &str = "hc-git-meta.json";

/// Copy of the contract recorded by the latest baseline, beside the ledger, for the
/// hooks' fast path. The signed `Baseline` record is the authority (`hc brief` reads it).
pub const CONTRACT_FILE: &str = "hc-contract.json";

pub fn recorded_contract(docket_dir: &Path) -> Option<serde_json::Value> {
    serde_json::from_str(&std::fs::read_to_string(docket_dir.join(CONTRACT_FILE)).ok()?).ok()
}

fn load_git_meta(docket_dir: &Path) -> Option<BTreeMap<String, bedrock::Digest>> {
    serde_json::from_str(&std::fs::read_to_string(docket_dir.join(GIT_META_FILE)).ok()?).ok()
}

/// Best effort: a lost cache means the next comparison is skipped, and records nothing.
fn store_git_meta(docket_dir: &Path, meta: &BTreeMap<String, bedrock::Digest>) {
    if let Ok(text) = serde_json::to_string(meta) {
        let _ = std::fs::write(docket_dir.join(GIT_META_FILE), text);
    }
}

/// `.git/config` and every file in `.git/hooks`, by digest. Git never tracks its
/// own directory, so a session that plants a hook or rewrites the repository's
/// config (code that runs the next time anyone uses git here) is invisible to
/// tree snapshots and is recorded from these digests instead.
pub fn git_meta(worktree: &Path) -> BTreeMap<String, bedrock::Digest> {
    let dot = worktree.join(".git");
    let dir = match std::fs::read_to_string(&dot) {
        Ok(pointer) => pointer.strip_prefix("gitdir:").map_or(dot.clone(), |p| worktree.join(p.trim())),
        Err(_) => dot,
    };
    // A linked worktree's gitdir names the shared repository in `commondir`; config and hooks live there.
    let dir = match std::fs::read_to_string(dir.join("commondir")) {
        Ok(common) => dir.join(common.trim()),
        Err(_) => dir,
    };
    let mut files = vec![(".git/config".to_owned(), dir.join("config"))];
    if let Ok(entries) = std::fs::read_dir(dir.join("hooks")) {
        for e in entries.flatten().filter(|e| e.path().is_file()) {
            files.push((format!(".git/hooks/{}", e.file_name().to_string_lossy()), e.path()));
        }
    }
    files.into_iter().filter_map(|(name, path)| Some((name, bedrock::Digest::of(&std::fs::read(path).ok()?)))).collect()
}

#[derive(Debug)]
pub struct Recorder<'a> {
    pub docket: &'a Docket,
    pub harness: &'a Identity,
    pub force_paths: &'a [String],
}

impl Recorder<'_> {
    fn obs(&self, ts: &str, session: &str, subject: Vec<ResourceDescriptor>, event: Event) -> Result<u64, DocketError> {
        let obs = Obs { session: session.to_owned(), ts: ts.to_owned(), event };
        self.docket.append(self.harness, ts, PREDICATE_OBS, subject, &obs)
    }

    /// Record a `file_write` for every `.git/config` or `.git/hooks/*` change since the
    /// repository's last observation, in whichever session observes it.
    fn record_git_meta(&self, ts: &str, session: &str, worktree: &Path) -> Result<(), DocketError> {
        let dir = self.docket.git_dir();
        let now = git_meta(worktree);
        if let Some(before) = load_git_meta(&dir) {
            let names: BTreeSet<&String> = now.keys().chain(before.keys()).collect();
            for name in names.into_iter().filter(|n| now.get(*n) != before.get(*n)) {
                let subject = now.get(name).map(|d| vec![ResourceDescriptor::sha256(name.clone(), *d)]).unwrap_or_default();
                self.obs(ts, session, subject, Event::FileWrite { path: name.clone(), before: None, after: None, added_lines: vec![] })?;
            }
        }
        store_git_meta(&dir, &now);
        Ok(())
    }

    /// Record the repository as it is now, under `session`: what a claim filed next is judged against.
    pub fn observe(&self, ts: &str, session: &str, worktree: &Path) -> Result<Oid, DocketError> {
        Ok(self.advance(ts, session, worktree)?.0)
    }

    /// Snapshot the working tree and record every change since the repository's last
    /// snapshot (from any session) as this session's writes. Returns the tree and changes.
    fn advance(&self, ts: &str, session: &str, worktree: &Path) -> Result<(Oid, Vec<Change>), DocketError> {
        self.record_git_meta(ts, session, worktree)?;
        let tree = self.snapshot(worktree)?;
        let base = self.docket.last_tree()?;
        if base.as_deref() == Some(tree.as_str()) {
            return Ok((tree, Vec::new()));
        }
        let changes = match &base {
            Some(base) => self.record_changes(ts, session, base, &tree)?,
            None => Vec::new(),
        };
        self.obs(ts, session, vec![ResourceDescriptor::git("worktree", "gitTree", &tree)], Event::Checkpoint { tree: tree.clone() })?;
        self.docket.set_last_tree(&tree)?;
        Ok((tree, changes))
    }

    fn snapshot(&self, worktree: &Path) -> Result<Oid, DocketError> {
        Ok(snapshot::snapshot(&self.docket.git_dir(), worktree, self.force_paths)?)
    }

    pub fn open(
        &self,
        ts: &str,
        session: &str,
        task: &str,
        worktree: &Path,
        agent: AgentAttestation,
        isolation: Isolation,
    ) -> Result<Open, DocketError> {
        // The session starts where the repository was last seen, so anything changed in
        // between is recorded as this session's writes by the first `advance`.
        let start = match self.docket.last_tree()? {
            Some(tree) => tree,
            None => self.snapshot(worktree)?,
        };
        let head = Git::untrusted(worktree).try_run(&["rev-parse", "--verify", "-q", "HEAD"])?;
        let event = Event::SessionStart { task: task.to_owned(), agent, isolation, tree: start.clone(), head };
        self.obs(ts, session, vec![ResourceDescriptor::git("worktree", "gitTree", &start)], event)?;
        let mut open = Open { session: session.to_owned(), task: task.to_owned(), worktree: worktree.to_path_buf(), tree: start };
        self.checkpoint(ts, &mut open)?;
        Ok(open)
    }

    /// Record the owner's accepted starting point: the current code and the task
    /// contract. Later writes are attributed to agent sessions; earlier ones are not.
    pub fn baseline(&self, ts: &str, worktree: &Path, task: serde_json::Value, recorded_by: AgentAttestation) -> Result<Oid, DocketError> {
        let tree = self.snapshot(worktree)?;
        let head = Git::untrusted(worktree).try_run(&["rev-parse", "--verify", "-q", "HEAD"])?;
        std::fs::write(self.docket.git_dir().join(CONTRACT_FILE), serde_json::to_string_pretty(&task).unwrap_or_default())?;
        let event = Event::Baseline { tree: tree.clone(), head, task, recorded_by };
        self.obs(ts, "baseline", vec![ResourceDescriptor::git("worktree", "gitTree", &tree)], event)?;
        self.docket.set_last_tree(&tree)?;
        store_git_meta(&self.docket.git_dir(), &git_meta(worktree));
        Ok(tree)
    }

    fn record_changes(&self, ts: &str, session: &str, base: &str, tree: &str) -> Result<Vec<Change>, DocketError> {
        let changes = diff::changes(&self.docket.git_dir(), base, tree)?;
        for change in &changes {
            let blob = change.after.clone().or_else(|| change.before.clone()).expect("a change has one side");
            let subject = vec![ResourceDescriptor::git(change.path.clone(), "gitBlob", &blob)];
            let event = Event::FileWrite {
                path: change.path.clone(),
                before: change.before.clone(),
                after: change.after.clone(),
                added_lines: change.added_lines.clone(),
            };
            self.obs(ts, session, subject, event)?;
        }
        Ok(changes)
    }

    /// Record the changes since the last snapshot at a step boundary and advance the baseline.
    pub fn checkpoint(&self, ts: &str, open: &mut Open) -> Result<Vec<Change>, DocketError> {
        let (tree, changes) = self.advance(ts, &open.session, &open.worktree)?;
        open.tree = tree;
        Ok(changes)
    }

    /// Record the agent's own plan as read from a native plan tool.
    pub fn plan(&self, ts: &str, session: &str, source: &str, steps: Vec<bedrock::ir::PlanStep>) -> Result<u64, DocketError> {
        let digest = bedrock::canon::digest(&steps)?;
        self.obs(ts, session, vec![ResourceDescriptor::sha256("plan", digest)], Event::PlanUpdate { source: source.to_owned(), steps })
    }

    /// Snapshot again and record one `file_write` per changed path, then `session_end`.
    pub fn close(&self, ts: &str, open: &Open, exit_code: Option<i32>) -> Result<Oid, DocketError> {
        let (tree, _) = self.advance(ts, &open.session, &open.worktree)?;
        let event = Event::SessionEnd { exit_code, tree: tree.clone() };
        self.obs(ts, &open.session, vec![ResourceDescriptor::git("worktree", "gitTree", &tree)], event)?;
        Ok(tree)
    }

    pub fn no_claim(&self, ts: &str, session: &str, diagnostics: Vec<Diagnostic>) -> Result<u64, DocketError> {
        self.obs(ts, session, vec![], Event::NoClaim { diagnostics })
    }

    /// Record that `session` read repository history (see [`Event::HistoryRead`]).
    pub fn history_read(&self, ts: &str, session: &str, command: &str) -> Result<u64, DocketError> {
        self.obs(ts, session, vec![], Event::HistoryRead { command: command.to_owned() })
    }
}

/// Record each claim of an envelope as its own entry, signed by the agent's identity.
pub fn record_claims(
    docket: &Docket,
    agent: &Identity,
    ts: &str,
    session: &str,
    attestation: &AgentAttestation,
    envelope: &ClaimEnvelope,
) -> Result<Vec<u64>, DocketError> {
    envelope
        .claims
        .iter()
        .map(|claim| {
            let record = ClaimRecord {
                session: session.to_owned(),
                task: envelope.task.clone(),
                attestation: attestation.clone(),
                claim: claim.clone(),
            };
            let digest = bedrock::canon::digest(claim)?;
            docket.append(agent, ts, PREDICATE_CLAIM, vec![ResourceDescriptor::sha256(format!("claim:{}", claim.id), digest)], &record)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use bedrock::ir::{AttestationLevel, Body, Role};

    fn paths_written(docket: &Docket, session: &str) -> Vec<String> {
        docket
            .records()
            .unwrap()
            .into_iter()
            .filter_map(|r| match r.body {
                Body::Obs(Obs { session: s, event: Event::FileWrite { path, .. }, .. }) if s == session => Some(path),
                _ => None,
            })
            .collect()
    }

    /// The headless path (`hc run`, the assay): open, work, close. Git metadata is
    /// compared between snapshots, so it is an endpoint comparison, not an event log.
    #[test]
    fn records_git_metadata_changes_between_snapshots_not_events() {
        let tmp = tempfile::tempdir().unwrap();
        let wt = tmp.path().join("wt");
        Git::default().run(&["init", "-q", wt.to_str().unwrap()]).unwrap();
        std::fs::write(wt.join("a.py"), "x = 1\n").unwrap();
        let keys = seal::Keyring::new(tmp.path().join("keys"));
        let harness = keys.ensure(Role::Harness, "local").unwrap();
        let docket = Docket::open(&tmp.path().join("docket.git"), keys).unwrap();
        let rec = Recorder { docket: &docket, harness: &harness, force_paths: &[] };
        let agent = AgentAttestation { agent_id: "t".into(), level: AttestationLevel::Declared, executable: None, pid: None };
        let mut open = rec.open("2026-01-01T00:00:00Z", "s", "t", &wt, agent, Isolation::None).unwrap();

        // An ordinary change and two .git changes in the same interval are all recorded.
        std::fs::write(wt.join("a.py"), "x = 2\n").unwrap();
        std::fs::write(wt.join(".git/hooks/post-commit"), "#!/bin/sh\n").unwrap();
        let config = std::fs::read_to_string(wt.join(".git/config")).unwrap();
        std::fs::write(wt.join(".git/config"), format!("{config}[core]\n\tfsmonitor = touch /tmp/x\n")).unwrap();
        rec.checkpoint("2026-01-01T00:00:01Z", &mut open).unwrap();
        assert_eq!(paths_written(&docket, "s"), [".git/config", ".git/hooks/post-commit", "a.py"]);

        // Planted and removed between two snapshots: nothing to compare, nothing recorded.
        std::fs::write(wt.join(".git/hooks/pre-push"), "#!/bin/sh\n").unwrap();
        std::fs::remove_file(wt.join(".git/hooks/pre-push")).unwrap();
        rec.close("2026-01-01T00:00:02Z", &open, Some(0)).unwrap();
        assert_eq!(paths_written(&docket, "s").len(), 3, "a write reverted before the next snapshot leaves no record");
    }
}
