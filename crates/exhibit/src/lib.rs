//! exhibit: collects the evidence. Runs in the harness, outside the agent's
//! cell, and records what happened as harness-signed docket entries.
//!
//! Snapshots are git tree objects written into the docket repository's object
//! database through a throwaway index, so the agent can neither see nor alter
//! the baseline it is compared against.

pub mod diff;
pub mod snapshot;

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
#[derive(Debug, Clone)]
pub struct Open {
    pub session: String,
    pub task: String,
    pub worktree: PathBuf,
    pub tree: Oid,
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
        let head = Git::at(worktree).try_run(&["rev-parse", "--verify", "-q", "HEAD"])?;
        let event = Event::SessionStart { task: task.to_owned(), agent, isolation, tree: tree.clone(), head };
        self.obs(ts, session, vec![ResourceDescriptor::git("worktree", "gitTree", &tree)], event)?;
        Ok(Open { session: session.to_owned(), task: task.to_owned(), worktree: worktree.to_path_buf(), tree })
    }

    /// Snapshot again and record one `file_write` per changed path, then `session_end`.
    pub fn close(&self, ts: &str, open: &Open, exit_code: Option<i32>) -> Result<Oid, DocketError> {
        let tree = self.snapshot(&open.worktree)?;
        for change in diff::changes(&self.docket.git_dir(), &open.tree, &tree)? {
            let blob = change.after.clone().or_else(|| change.before.clone()).expect("a change has one side");
            let subject = vec![ResourceDescriptor::git(change.path.clone(), "gitBlob", &blob)];
            let event = Event::FileWrite { path: change.path, before: change.before, after: change.after, added_lines: change.added_lines };
            self.obs(ts, &open.session, subject, event)?;
        }
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
            let record = ClaimRecord { session: session.to_owned(), task: envelope.task.clone(), attestation: attestation.clone(), claim: claim.clone() };
            let digest = bedrock::canon::digest(claim)?;
            docket.append(agent, ts, PREDICATE_CLAIM, vec![ResourceDescriptor::sha256(format!("claim:{}", claim.id), digest)], &record)
        })
        .collect()
}
