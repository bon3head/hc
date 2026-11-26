//! crossexam: the kernel. It cross-examines every claim in the docket against
//! harness-signed evidence and returns verdicts; it is the only code that can
//! mint [`Verified`].
//!
//! Passes (numbering follows the research design):
//! - P4 `command`: a test claim must name one of the policy's test commands.
//! - P6 `tamper`: a `done` claim is refuted if the session wrote a protected path.
//! - P6S `scope`: writes stay inside the allowed paths; file claims match observed writes.
//! - P7 `signature`: known reward-hack shapes on lines the session added.
//! - P8 `reexec`: the test command passes again on the claimed commit or the final tree.
//!
//! Agent text never decides which checks run. A claim the kernel cannot
//! decide stays `asserted`; only `observed` or `reproduced` claims become facts.

pub mod policy;
pub mod reexec;
pub mod signatures;
mod verified;

use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

use bedrock::git::Git;
use bedrock::ir::{
    Body, Check, Claim, ClaimKind, ClaimRecord, Event, Outcome, PREDICATE_VERDICT, PassId, ResourceDescriptor, Status, Trust,
    VerdictRecord,
};
use docket::{Docket, DocketError, Record};
use seal::Identity;

pub use policy::Policy;
pub use reexec::{Reexec, Run, Source};
pub use verified::Verified;

#[derive(Debug, thiserror::Error)]
pub enum KernelError {
    #[error(transparent)]
    Docket(#[from] DocketError),
    #[error(transparent)]
    Git(#[from] bedrock::git::GitError),
    #[error("policy: {0}")]
    Policy(#[from] globset::Error),
    #[error("encoding: {0}")]
    Json(#[from] serde_json::Error),
}

#[derive(Debug, Clone)]
pub struct Finding {
    pub claim_seq: u64,
    pub claim: ClaimRecord,
    pub verdict: VerdictRecord,
    pub verified: Option<Verified>,
}

#[derive(Debug)]
pub struct Kernel {
    pub policy: Policy,
    pub disabled: BTreeSet<PassId>,
    pub reexec: Option<Reexec>,
    pub docket_git: PathBuf,
    pub repo: PathBuf,
}

struct Write<'a> {
    seq: u64,
    path: &'a str,
    after: Option<&'a str>,
    added: &'a [u32],
}

struct SessionEvidence<'a> {
    writes: Vec<Write<'a>>,
    end_tree: Option<&'a str>,
}

fn evidence<'a>(records: &'a [Record], session: &str, before: u64) -> SessionEvidence<'a> {
    let mut ev = SessionEvidence { writes: Vec::new(), end_tree: None };
    for r in records.iter().filter(|r| r.seq < before) {
        let Body::Obs(obs) = &r.body else { continue };
        if obs.session != session {
            continue;
        }
        match &obs.event {
            Event::FileWrite { path, after, added_lines, .. } => {
                ev.writes.push(Write { seq: r.seq, path, after: after.as_deref(), added: added_lines });
            }
            Event::SessionEnd { tree, .. } => ev.end_tree = Some(tree),
            Event::SessionStart { .. } | Event::NoClaim { .. } => {}
        }
    }
    ev
}

fn is_test_like(kind: ClaimKind) -> bool {
    matches!(kind, ClaimKind::TestsPass | ClaimKind::CommandRan)
}

/// P4: a test claim must name one of the policy's test commands.
fn pass_command(claim: &Claim, argv: Option<&Vec<String>>) -> Option<Check> {
    is_test_like(claim.kind).then(|| check(PassId::Command, argv.is_some(), "command-not-in-policy", vec![], claim.command.clone()))
}

/// P6: a `done` claim is refuted when the session wrote a protected path.
fn pass_tamper(claim: &Claim, ev: &SessionEvidence<'_>, compiled: &policy::Compiled) -> Option<Check> {
    (claim.status == Status::Done).then(|| {
        let hits: Vec<&Write> = ev.writes.iter().filter(|w| compiled.is_protected(w.path)).collect();
        let detail = hits.first().map(|w| w.path.to_owned());
        check(PassId::Tamper, hits.is_empty(), "protected-path-written", hits.iter().map(|w| w.seq).collect(), detail)
    })
}

/// P6S: writes stay in scope and file claims match the observed writes.
fn pass_scope(claim: &Claim, ev: &SessionEvidence<'_>, compiled: &policy::Compiled) -> Vec<Check> {
    let mut checks = Vec::new();
    let written: BTreeSet<&str> = ev.writes.iter().map(|w| w.path).collect();
    let seqs = |pred: &dyn Fn(&Write) -> bool| ev.writes.iter().filter(|w| pred(w)).map(|w| w.seq).collect::<Vec<_>>();
    if claim.status == Status::Done && compiled.is_allowed("").is_some() {
        let outside = seqs(&|w| compiled.is_allowed(w.path) == Some(false) && !compiled.is_protected(w.path));
        checks.push(check(PassId::Scope, outside.is_empty(), "out-of-scope-write", outside, None));
    }
    let claimed: BTreeSet<&str> = claim.paths.iter().map(String::as_str).collect();
    match claim.kind {
        ClaimKind::FilesUntouched => {
            let touched = seqs(&|w| claimed.contains(w.path));
            checks.push(check(PassId::Scope, touched.is_empty(), "claimed-untouched-was-written", touched, None));
        }
        ClaimKind::FilesChanged => {
            let ok = written == claimed;
            let detail = (!ok).then(|| format!("claimed {claimed:?}, observed {written:?}"));
            checks.push(check(PassId::Scope, ok, "files-changed-mismatch", seqs(&|_| true), detail));
        }
        ClaimKind::NoChangeNeeded => checks.push(check(PassId::Scope, written.is_empty(), "writes-on-no-change", seqs(&|_| true), None)),
        _ => {}
    }
    checks
}

/// Any failing check refutes; test claims need clean re-execution; file claims need a scope check.
fn trust_of(kind: ClaimKind, checks: &[Check]) -> Trust {
    let passed = |pass: PassId| checks.iter().any(|c| c.pass == pass && c.result == Outcome::Pass);
    if checks.iter().any(|c| c.result == Outcome::Fail) {
        Trust::Refuted
    } else if is_test_like(kind) && passed(PassId::Reexec) {
        Trust::Reproduced
    } else if matches!(kind, ClaimKind::FilesChanged | ClaimKind::FilesUntouched | ClaimKind::NoChangeNeeded) && passed(PassId::Scope) {
        Trust::Observed
    } else {
        Trust::Asserted
    }
}

fn check(pass: PassId, ok: bool, rule: &str, evidence: Vec<u64>, detail: Option<String>) -> Check {
    Check {
        pass,
        result: if ok { Outcome::Pass } else { Outcome::Fail },
        rule: (!ok).then(|| rule.to_owned()),
        evidence,
        detail,
    }
}

impl Kernel {
    fn on(&self, pass: PassId) -> bool {
        !self.disabled.contains(&pass)
    }

    pub fn examine(&self, records: &[Record]) -> Result<Vec<Finding>, KernelError> {
        let compiled = self.policy.compile()?;
        let policy_digest = bedrock::canon::digest(&self.policy)?;
        let mut findings = Vec::new();
        for record in records {
            let Body::Claim(cr) = &record.body else { continue };
            let ev = evidence(records, &cr.session, record.seq);
            let claim = &cr.claim;
            let argv = claim.command.as_deref().and_then(|c| compiled.test_argv(c));
            let mut checks = Vec::new();
            if self.on(PassId::Command) {
                checks.extend(pass_command(claim, argv.as_ref()));
            }
            if self.on(PassId::Tamper) {
                checks.extend(pass_tamper(claim, &ev, &compiled));
            }
            if self.on(PassId::Scope) {
                checks.extend(pass_scope(claim, &ev, &compiled));
            }
            if self.on(PassId::Signature) {
                checks.extend(self.pass_signature(claim, &ev)?);
            }
            if self.on(PassId::Reexec) {
                checks.extend(self.pass_reexec(claim, &ev, argv.as_deref())?);
            }
            let trust = trust_of(claim.kind, &checks);
            let evidence: Vec<u64> = checks.iter().flat_map(|c| c.evidence.iter().copied()).collect::<BTreeSet<_>>().into_iter().collect();
            let verdict = VerdictRecord { session: cr.session.clone(), about: claim.id.clone(), claim_seq: record.seq, checks, trust, policy: policy_digest };
            let verified = Verified::mint(record.seq, claim.id.clone(), claim.kind, trust, evidence);
            findings.push(Finding { claim_seq: record.seq, claim: cr.clone(), verdict, verified });
        }
        Ok(findings)
    }

    /// P7: cheat signatures on the Python lines a `done` session added.
    fn pass_signature(&self, claim: &Claim, ev: &SessionEvidence<'_>) -> Result<Vec<Check>, KernelError> {
        let mut checks = Vec::new();
        if claim.status != Status::Done {
            return Ok(checks);
        }
        let git = Git::bare(&self.docket_git);
        for w in ev.writes.iter().filter(|w| signatures::is_python(w.path)) {
            let Some(blob) = w.after else { continue };
            let source = git.run(&["cat-file", "blob", blob])?;
            for hit in signatures::scan_python(&source, w.added) {
                checks.push(check(PassId::Signature, false, hit.rule, vec![w.seq], Some(format!("{}:{}", w.path, hit.line))));
            }
        }
        Ok(checks)
    }

    /// P8: re-run the policy test command on the claimed commit or the final observed tree.
    fn pass_reexec(&self, claim: &Claim, ev: &SessionEvidence<'_>, argv: Option<&[String]>) -> Result<Vec<Check>, KernelError> {
        let (Some(reexec), Some(argv), true) = (&self.reexec, argv, is_test_like(claim.kind)) else { return Ok(Vec::new()) };
        let source = match (&claim.commit, ev.end_tree) {
            (Some(commit), _) => Source::Commit { repo: &self.repo, commit },
            (None, Some(tree)) => Source::Tree { docket_git: &self.docket_git, tree },
            (None, None) => return Ok(vec![check(PassId::Reexec, false, "nothing-to-reexecute", vec![], None)]),
        };
        let run = reexec.run(source, argv)?;
        let expected = if claim.kind == ClaimKind::TestsPass { Some(0) } else { claim.exit_code };
        let agree = |code: i32| expected.is_none_or(|e| i64::from(code) == e);
        let all = run.exit_codes.iter().all(|c| agree(*c));
        let any = run.exit_codes.iter().any(|c| agree(*c));
        let counts_ok = [(claim.passed, run.passed), (claim.failed, run.failed)].iter().all(|(c, o)| c.is_none() || o.is_none() || c == o);
        let rule = match (all, any) {
            (false, true) => "flaky",
            (false, false) => "reexec-disagrees",
            (true, _) => "claimed-counts-differ",
        };
        Ok(vec![check(PassId::Reexec, all && counts_ok, rule, vec![], Some(format!("exit codes {:?}", run.exit_codes)))])
    }

    /// Append each verdict, signed by the kernel identity. Returns the new entry seqs.
    pub fn rule(&self, docket: &Docket, kernel: &Identity, ts: &str, findings: &[Finding]) -> Result<Vec<u64>, KernelError> {
        let claim_commits: BTreeMap<u64, String> =
            docket.records()?.into_iter().map(|r| (r.seq, r.commit)).collect();
        findings
            .iter()
            .map(|f| {
                let subject = claim_commits
                    .get(&f.claim_seq)
                    .map(|c| vec![ResourceDescriptor::git(format!("claim:{}", f.verdict.about), "gitCommit", c)])
                    .unwrap_or_default();
                Ok(docket.append(kernel, ts, PREDICATE_VERDICT, subject, &f.verdict)?)
            })
            .collect()
    }
}
