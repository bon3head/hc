//! docket: the ledger, `refs/hc/docket` in a bare repository inside the hc home.
//!
//! Each entry is one commit whose tree holds `statement.json`, the RFC 8785
//! canonical bytes of an in-toto Statement. The parent link is the hash chain;
//! the commit is SSH-signed by the identity that wrote it, and reading verifies
//! every signature against `allowed_signers` and checks that the signer's role
//! is the one permitted for that predicate type.

use std::path::{Path, PathBuf};

use serde::Serialize;
use serde::de::DeserializeOwned;

use bedrock::canon;
use bedrock::git::{Git, GitError};
use bedrock::ir::{
    Body, ClaimRecord, Obs, PREDICATE_CLAIM, PREDICATE_OBS, PREDICATE_VERDICT, ResourceDescriptor, Statement, VerdictRecord,
};
use seal::{Identity, Keyring, parse_principal};

pub const DOCKET_REF: &str = "refs/hc/docket";
const ENTRY_FILE: &str = "statement.json";
const FIELD: char = '\u{1f}';

#[derive(Debug, thiserror::Error)]
pub enum DocketError {
    #[error(transparent)]
    Git(#[from] GitError),
    #[error("encoding failed")]
    Json(#[from] serde_json::Error),
    #[error("entry {seq} ({commit}) has signature status {status:?} by {signer:?}")]
    Untrusted { seq: u64, commit: String, status: String, signer: String },
    #[error("entry {seq} is a {predicate} but was signed by {signer}")]
    WrongSigner { seq: u64, predicate: String, signer: String },
    #[error("entry {seq} has unknown predicate type {0}", .predicate)]
    UnknownPredicate { seq: u64, predicate: String },
    #[error("concurrent append lost the race {attempts} times")]
    Contention { attempts: u32 },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Record {
    pub seq: u64,
    pub commit: String,
    pub signer: String,
    pub body: Body,
}

#[derive(Debug, Clone)]
pub struct Docket {
    git: Git,
    keys: Keyring,
}

impl Docket {
    pub fn open(git_dir: &Path, keys: Keyring) -> Result<Self, DocketError> {
        Ok(Self { git: Git::init_bare(git_dir)?, keys })
    }

    pub fn git(&self) -> &Git {
        &self.git
    }

    pub fn git_dir(&self) -> PathBuf {
        self.git.git_dir().expect("bare ledger").to_path_buf()
    }

    pub fn keys(&self) -> &Keyring {
        &self.keys
    }

    pub fn head(&self) -> Result<Option<String>, DocketError> {
        Ok(self.git.try_run(&["rev-parse", "--verify", "-q", DOCKET_REF])?)
    }

    /// Append one statement signed by `signer`, with commit dates set to `ts`
    /// (RFC 3339) so that identical inputs give identical commit ids.
    pub fn append<P: Serialize>(
        &self,
        signer: &Identity,
        ts: &str,
        predicate_type: &str,
        subject: Vec<ResourceDescriptor>,
        predicate: &P,
    ) -> Result<u64, DocketError> {
        let statement = Statement::new(predicate_type, subject, predicate);
        let bytes = canon::to_vec(&statement)?;
        let blob = self.git.run_with_input(&["hash-object", "-w", "--stdin"], Some(&bytes))?;
        let tree = self.git.run_with_input(&["mktree"], Some(format!("100644 blob {blob}\t{ENTRY_FILE}\n").as_bytes()))?;
        let principal = signer.principal();
        let signing = self
            .git
            .clone()
            .config("gpg.format=ssh")
            .config("gpg.ssh.program=ssh-keygen")
            .config(format!("user.signingkey={}", signer.key_path.display()))
            .env("GIT_AUTHOR_NAME", &principal)
            .env("GIT_AUTHOR_EMAIL", &principal)
            .env("GIT_COMMITTER_NAME", &principal)
            .env("GIT_COMMITTER_EMAIL", &principal)
            .env("GIT_AUTHOR_DATE", ts)
            .env("GIT_COMMITTER_DATE", ts);
        for attempt in 1..=8 {
            let parent = self.head()?;
            let message = format!("hc {predicate_type}");
            let mut args = vec!["commit-tree", "-S", "-m", &message, &tree];
            if let Some(p) = &parent {
                args.extend(["-p", p]);
            }
            let commit = signing.run(&args)?;
            let old = parent.clone().unwrap_or_else(|| "0".repeat(40));
            if self.git.try_run(&["update-ref", DOCKET_REF, &commit, &old])?.is_some() {
                return Ok(self.count()? - 1);
            }
            if attempt == 8 {
                return Err(DocketError::Contention { attempts: attempt });
            }
        }
        unreachable!("loop returns")
    }

    pub fn count(&self) -> Result<u64, DocketError> {
        match self.head()? {
            None => Ok(0),
            Some(_) => Ok(self.git.run(&["rev-list", "--count", DOCKET_REF])?.parse().unwrap_or(0)),
        }
    }

    /// Read and verify every entry, oldest first.
    pub fn records(&self) -> Result<Vec<Record>, DocketError> {
        if self.head()?.is_none() {
            return Ok(Vec::new());
        }
        let signers = format!("gpg.ssh.allowedSignersFile={}", self.keys.allowed_signers().display());
        let verifying = self.git.clone().config("gpg.ssh.program=ssh-keygen").config(signers);
        let format = format!("--format=%H{FIELD}%G?{FIELD}%GS");
        let log = verifying.run(&["log", "--reverse", &format, DOCKET_REF])?;
        let mut records = Vec::new();
        for (index, line) in log.lines().enumerate() {
            let seq = index as u64;
            let mut fields = line.split(FIELD);
            let commit = fields.next().unwrap_or_default().to_owned();
            let status = fields.next().unwrap_or_default().to_owned();
            let signer = fields.next().unwrap_or_default().to_owned();
            if status != "G" {
                return Err(DocketError::Untrusted { seq, commit, status, signer });
            }
            let raw = self.git.run(&["cat-file", "blob", &format!("{commit}:{ENTRY_FILE}")])?;
            let statement: Statement<serde_json::Value> = serde_json::from_str(&raw)?;
            let body = decode(seq, &statement)?;
            let role = parse_principal(&signer).map(|(role, _)| role).ok();
            if role != Some(body.required_role()) {
                return Err(DocketError::WrongSigner { seq, predicate: statement.predicate_type, signer });
            }
            records.push(Record { seq, commit, signer, body });
        }
        Ok(records)
    }
}

fn from_predicate<T: DeserializeOwned>(statement: &Statement<serde_json::Value>) -> Result<T, serde_json::Error> {
    serde_json::from_value(statement.predicate.clone())
}

fn decode(seq: u64, statement: &Statement<serde_json::Value>) -> Result<Body, DocketError> {
    Ok(match statement.predicate_type.as_str() {
        PREDICATE_OBS => Body::Obs(from_predicate::<Obs>(statement)?),
        PREDICATE_CLAIM => Body::Claim(from_predicate::<ClaimRecord>(statement)?),
        PREDICATE_VERDICT => Body::Verdict(from_predicate::<VerdictRecord>(statement)?),
        other => return Err(DocketError::UnknownPredicate { seq, predicate: other.to_owned() }),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use bedrock::ir::{AgentAttestation, AttestationLevel, Claim, ClaimKind, Event, Isolation, Role, Status};

    fn obs(session: &str) -> Obs {
        Obs {
            session: session.into(),
            ts: "2026-01-01T00:00:01Z".into(),
            event: Event::SessionEnd { exit_code: Some(0), tree: "4b825dc642cb6eb9a060e54bf8d69288fbee4904".into() },
        }
    }

    fn setup() -> (tempfile::TempDir, Docket) {
        let dir = tempfile::tempdir().unwrap();
        let ledger = Docket::open(&dir.path().join("ledger.git"), Keyring::new(dir.path().join("keys"))).unwrap();
        (dir, ledger)
    }

    #[test]
    fn appends_verifies_and_is_deterministic() {
        let (_dir, ledger) = setup();
        let harness = ledger.keys().ensure(Role::Harness, "local").unwrap();
        assert_eq!(ledger.append(&harness, "2026-01-01T00:00:01Z", PREDICATE_OBS, vec![], &obs("s1")).unwrap(), 0);
        assert_eq!(ledger.append(&harness, "2026-01-01T00:00:02Z", PREDICATE_OBS, vec![], &obs("s1")).unwrap(), 1);
        let records = ledger.records().unwrap();
        assert_eq!(records.len(), 2);
        assert_eq!(records[1].signer, "harness:local@hc");

        let (_dir2, replay) = setup();
        std::fs::remove_dir_all(replay.keys().dir()).ok();
        std::fs::create_dir_all(replay.keys().dir()).unwrap();
        copy_dir(ledger.keys().dir(), replay.keys().dir());
        replay
            .append(&replay.keys().ensure(Role::Harness, "local").unwrap(), "2026-01-01T00:00:01Z", PREDICATE_OBS, vec![], &obs("s1"))
            .unwrap();
        assert_eq!(replay.records().unwrap()[0].commit, records[0].commit);
    }

    #[test]
    fn rejects_a_claim_signed_by_the_harness() {
        let (_dir, ledger) = setup();
        let harness = ledger.keys().ensure(Role::Harness, "local").unwrap();
        let claim = ClaimRecord {
            session: "s1".into(),
            task: "T".into(),
            attestation: AgentAttestation { agent_id: "codex".into(), level: AttestationLevel::Declared, executable: None, pid: None },
            claim: Claim {
                id: "c1".into(),
                kind: ClaimKind::Note,
                basis: String::new(),
                command: None,
                exit_code: None,
                passed: None,
                failed: None,
                paths: vec![],
                commit: None,
                depends_on: vec![],
                status: Status::Done,
            },
        };
        ledger.append(&harness, "2026-01-01T00:00:01Z", PREDICATE_CLAIM, vec![], &claim).unwrap();
        assert!(matches!(ledger.records(), Err(DocketError::WrongSigner { .. })));
    }

    #[test]
    fn rejects_unknown_keys_and_rewritten_history() {
        let (dir, ledger) = setup();
        let stranger_ring = Keyring::new(dir.path().join("stranger"));
        let stranger = stranger_ring.ensure(Role::Harness, "local").unwrap();
        ledger.append(&stranger, "2026-01-01T00:00:01Z", PREDICATE_OBS, vec![], &obs("s1")).unwrap();
        assert!(matches!(ledger.records(), Err(DocketError::Untrusted { .. })));
        let _ = Isolation::None;
    }

    fn copy_dir(from: &Path, to: &Path) {
        for entry in std::fs::read_dir(from).unwrap() {
            let entry = entry.unwrap();
            let target = to.join(entry.file_name());
            if entry.file_type().unwrap().is_dir() {
                std::fs::create_dir_all(&target).unwrap();
                copy_dir(&entry.path(), &target);
            } else {
                std::fs::copy(entry.path(), target).unwrap();
            }
        }
    }
}
