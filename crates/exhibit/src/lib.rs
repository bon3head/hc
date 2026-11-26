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
    /// Digests of `.git/config` and `.git/hooks/*` at the last snapshot.
    #[serde(default)]
    pub git_meta: BTreeMap<String, bedrock::Digest>,
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

    /// Record a `file_write` for every `.git/config` or `.git/hooks/*` change since the last snapshot; return the new digests.
    fn record_git_meta(&self, ts: &str, open: &Open) -> Result<BTreeMap<String, bedrock::Digest>, DocketError> {
        let now = git_meta(&open.worktree);
        let names: BTreeSet<&String> = now.keys().chain(open.git_meta.keys()).collect();
        for name in names.into_iter().filter(|n| now.get(*n) != open.git_meta.get(*n)) {
            let subject = now.get(name).map(|d| vec![ResourceDescriptor::sha256(name.clone(), *d)]).unwrap_or_default();
            self.obs(ts, &open.session, subject, Event::FileWrite { path: name.clone(), before: None, after: None, added_lines: vec![] })?;
        }
        Ok(now)
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
        let tree = self.snapshot(worktree)?;
        let head = Git::untrusted(worktree).try_run(&["rev-parse", "--verify", "-q", "HEAD"])?;
        let event = Event::SessionStart { task: task.to_owned(), agent, isolation, tree: tree.clone(), head };
        self.obs(ts, session, vec![ResourceDescriptor::git("worktree", "gitTree", &tree)], event)?;
        Ok(Open {
            session: session.to_owned(),
            task: task.to_owned(),
            worktree: worktree.to_path_buf(),
            tree,
            git_meta: git_meta(worktree),
        })
    }

    fn record_changes(&self, ts: &str, open: &Open, tree: &str) -> Result<Vec<Change>, DocketError> {
        let changes = diff::changes(&self.docket.git_dir(), &open.tree, tree)?;
        for change in &changes {
            let blob = change.after.clone().or_else(|| change.before.clone()).expect("a change has one side");
            let subject = vec![ResourceDescriptor::git(change.path.clone(), "gitBlob", &blob)];
            let event = Event::FileWrite {
                path: change.path.clone(),
                before: change.before.clone(),
                after: change.after.clone(),
                added_lines: change.added_lines.clone(),
            };
            self.obs(ts, &open.session, subject, event)?;
        }
        Ok(changes)
    }

    /// Record the changes since the last snapshot at a step boundary and advance the baseline.
    pub fn checkpoint(&self, ts: &str, open: &mut Open) -> Result<Vec<Change>, DocketError> {
        open.git_meta = self.record_git_meta(ts, open)?;
        let tree = self.snapshot(&open.worktree)?;
        if tree == open.tree {
            return Ok(Vec::new());
        }
        let changes = self.record_changes(ts, open, &tree)?;
        self.obs(ts, &open.session, vec![ResourceDescriptor::git("worktree", "gitTree", &tree)], Event::Checkpoint { tree: tree.clone() })?;
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
        self.record_git_meta(ts, open)?;
        let tree = self.snapshot(&open.worktree)?;
        self.record_changes(ts, open, &tree)?;
        let event = Event::SessionEnd { exit_code, tree: tree.clone() };
        self.obs(ts, &open.session, vec![ResourceDescriptor::git("worktree", "gitTree", &tree)], event)?;
        Ok(tree)
    }

    pub fn no_claim(&self, ts: &str, session: &str, diagnostics: Vec<Diagnostic>) -> Result<u64, DocketError> {
        self.obs(ts, session, vec![], Event::NoClaim { diagnostics })
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
