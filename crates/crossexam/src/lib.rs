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
    Body, Check, Claim, ClaimKind, ClaimRecord, Event, Outcome, PREDICATE_VERDICT, PassId, ResourceDescriptor, Status, Trust, VerdictRecord,
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
    #[error("invalid policy")]
    Policy(#[from] globset::Error),
    #[error("encoding failed")]
    Json(#[from] serde_json::Error),
}

#[derive(Debug, Clone)]
pub struct Finding {
    pub claim_seq: u64,
    pub claim: ClaimRecord,
    pub verdict: VerdictRecord,
    pub verified: Option<Verified>,
    /// The tree the claim was judged against: the last snapshot before it was filed.
    pub subject_tree: Option<String>,
    /// Ledger position of the observation that recorded `subject_tree`.
    pub subject_seq: Option<u64>,
    /// The session's latest snapshot in the whole docket. When it differs from
    /// `subject_tree`, a test result describes code that has since changed.
    pub current_tree: Option<String>,
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
    start_tree: Option<&'a str>,
    end_tree: Option<&'a str>,
    /// Ledger position of the observation that recorded `end_tree`.
    end_seq: Option<u64>,
}

fn evidence<'a>(records: &'a [Record], session: &str, before: u64) -> SessionEvidence<'a> {
    let mut ev = SessionEvidence { writes: Vec::new(), start_tree: None, end_tree: None, end_seq: None };
    for r in records.iter().filter(|r| r.seq < before) {
        let Body::Obs(obs) = &r.body else { continue };
        if obs.session != session {
            continue;
        }
        match &obs.event {
            Event::FileWrite { path, after, added_lines, .. } => {
                ev.writes.push(Write { seq: r.seq, path, after: after.as_deref(), added: added_lines });
            }
            Event::SessionEnd { tree, .. } | Event::Checkpoint { tree } => {
                ev.end_tree = Some(tree);
                ev.end_seq = Some(r.seq);
            }
            Event::SessionStart { tree, .. } => {
                // Until something changes, the session's tree is the one it started from.
                ev.start_tree = Some(tree);
                ev.end_tree = Some(tree);
                ev.end_seq = Some(r.seq);
            }
            Event::NoClaim { .. } | Event::PlanUpdate { .. } => {}
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
    let claimed: BTreeSet<String> = claim.paths.iter().map(|p| normalize_path(p)).collect();
    let covers = |path: &str| claimed.iter().any(|c| path == c || path.strip_prefix(c.as_str()).is_some_and(|rest| rest.starts_with('/')));
    match claim.kind {
        ClaimKind::FilesUntouched => {
            let touched = seqs(&|w| covers(w.path));
            checks.push(check(PassId::Scope, touched.is_empty(), "claimed-untouched-was-written", touched, None));
        }
        ClaimKind::FilesChanged => {
            let ok = written.len() == claimed.len() && written.iter().all(|w| claimed.contains(*w));
            let detail = (!ok).then(|| format!("claimed {claimed:?}, observed {written:?}"));
            checks.push(check(PassId::Scope, ok, "files-changed-mismatch", seqs(&|_| true), detail));
        }
        ClaimKind::NoChangeNeeded => checks.push(check(PassId::Scope, written.is_empty(), "writes-on-no-change", seqs(&|_| true), None)),
        _ => {}
    }
    checks
}

/// `./src//a.py` and `src/a.py` name the same file; intake already rejected `..`, absolute and empty paths.
fn normalize_path(path: &str) -> String {
    path.split('/').filter(|part| !part.is_empty() && *part != ".").collect::<Vec<_>>().join("/")
}

/// Any failing check refutes; test claims need clean re-execution; file claims need a scope check.
fn trust_of(kind: ClaimKind, checks: &[Check]) -> Trust {
    let passed = |pass: PassId| checks.iter().any(|c| c.pass == pass && c.result == Outcome::Pass);
    if checks.iter().any(|c| c.result == Outcome::Fail) {
        Trust::Refuted
    } else if checks.iter().any(|c| c.result == Outcome::Unresolved) {
        Trust::Asserted
    } else if is_test_like(kind) && passed(PassId::Reexec) {
        Trust::Reproduced
    } else if matches!(kind, ClaimKind::FilesChanged | ClaimKind::FilesUntouched | ClaimKind::NoChangeNeeded) && passed(PassId::Scope) {
        Trust::Observed
    } else {
        Trust::Asserted
    }
}

fn unresolved(pass: PassId, rule: &str, detail: String) -> Check {
    Check { pass, result: Outcome::Unresolved, rule: Some(rule.to_owned()), evidence: vec![], detail: Some(detail) }
}

fn check(pass: PassId, ok: bool, rule: &str, evidence: Vec<u64>, detail: Option<String>) -> Check {
    Check { pass, result: if ok { Outcome::Pass } else { Outcome::Fail }, rule: (!ok).then(|| rule.to_owned()), evidence, detail }
}

impl Kernel {
    fn on(&self, pass: PassId) -> bool {
        !self.disabled.contains(&pass)
    }

    /// Digest of everything that decides a verdict: the policy and the disabled passes.
    fn digest(&self) -> Result<bedrock::Digest, KernelError> {
        Ok(bedrock::canon::digest(&(&self.policy, &self.disabled))?)
    }

    /// The kernel's own recorded verdicts that settled their claim under this exact policy.
    /// A verdict with an unresolved check is not settled: the claim is examined again.
    fn settled(records: &[Record], digest: bedrock::Digest) -> BTreeMap<u64, VerdictRecord> {
        records
            .iter()
            .filter_map(|r| if let Body::Verdict(v) = &r.body { Some(v) } else { None })
            .filter(|v| v.policy == digest && v.checks.iter().all(|c| c.result != Outcome::Unresolved))
            .map(|v| (v.claim_seq, v.clone()))
            .collect()
    }

    /// Judge every claim in the docket. A claim the kernel already settled under
    /// this policy keeps its recorded verdict: an observation made then stays
    /// true of the tree it was made on, whatever this machine can run now.
    pub fn examine(&self, records: &[Record]) -> Result<Vec<Finding>, KernelError> {
        let compiled = self.policy.compile()?;
        let policy_digest = self.digest()?;
        let settled = Self::settled(records, policy_digest);
        let mut findings = Vec::new();
        for record in records {
            let Body::Claim(cr) = &record.body else { continue };
            let ev = evidence(records, &cr.session, record.seq);
            let claim = &cr.claim;
            if let Some(verdict) = settled.get(&record.seq) {
                let evidence: Vec<u64> =
                    verdict.checks.iter().flat_map(|c| c.evidence.iter().copied()).collect::<BTreeSet<_>>().into_iter().collect();
                let verified = Verified::mint(record.seq, claim.id.clone(), claim.kind, verdict.trust, evidence);
                findings.push(Finding {
                    claim_seq: record.seq,
                    claim: cr.clone(),
                    verdict: verdict.clone(),
                    verified,
                    subject_tree: ev.end_tree.map(str::to_owned),
                    subject_seq: ev.end_seq,
                    current_tree: self::evidence(records, &cr.session, u64::MAX).end_tree.map(str::to_owned),
                });
                continue;
            }
            let argv = claim.command.as_deref().and_then(|c| compiled.test_argv(c));
            let mut checks = Vec::new();
            if self.on(PassId::Command) {
                checks.extend(pass_command(claim, argv.as_ref()));
            }
            if ev.start_tree.is_none() {
                // No recorded session: what the agent wrote is unknown, so the write-based
                // passes cannot pass. They would pass vacuously on an empty write set.
                checks.push(unresolved(
                    PassId::Tamper,
                    "no-session-evidence",
                    format!("no observations were recorded for session {:?}", cr.session),
                ));
            } else {
                if self.on(PassId::Tamper) {
                    checks.extend(pass_tamper(claim, &ev, &compiled));
                }
                if self.on(PassId::Scope) {
                    checks.extend(pass_scope(claim, &ev, &compiled));
                }
                if self.on(PassId::Signature) {
                    checks.extend(self.pass_signature(claim, &ev, &compiled)?);
                }
            }
            if self.on(PassId::Reexec) {
                checks.extend(self.pass_reexec(claim, &ev, argv.as_deref())?);
            }
            if self.on(PassId::Falsify) {
                checks.extend(self.pass_falsify(claim, &ev, &compiled, argv.as_deref())?);
            }
            let trust = trust_of(claim.kind, &checks);
            let evidence: Vec<u64> = checks.iter().flat_map(|c| c.evidence.iter().copied()).collect::<BTreeSet<_>>().into_iter().collect();
            let verdict = VerdictRecord {
                session: cr.session.clone(),
                about: claim.id.clone(),
                claim_seq: record.seq,
                checks,
                trust,
                policy: policy_digest,
            };
            let verified = Verified::mint(record.seq, claim.id.clone(), claim.kind, trust, evidence);
            let current_tree = self::evidence(records, &cr.session, u64::MAX).end_tree.map(str::to_owned);
            findings.push(Finding {
                claim_seq: record.seq,
                claim: cr.clone(),
                verdict,
                verified,
                subject_tree: ev.end_tree.map(str::to_owned),
                subject_seq: ev.end_seq,
                current_tree,
            });
        }
        Ok(findings)
    }

    /// P7: cheat signatures on the Python lines a `done` session added.
    fn pass_signature(&self, claim: &Claim, ev: &SessionEvidence<'_>, compiled: &policy::Compiled) -> Result<Vec<Check>, KernelError> {
        let mut checks = Vec::new();
        if claim.status != Status::Done {
            return Ok(checks);
        }
        let git = Git::bare(&self.docket_git);
        for w in ev.writes.iter().filter(|w| signatures::is_python(w.path)) {
            let Some(blob) = w.after else { continue };
            let source = git.run(&["cat-file", "blob", blob])?;
            for hit in signatures::scan_python(&source, w.added, compiled.is_test(w.path)) {
                checks.push(check(PassId::Signature, false, hit.rule, vec![w.seq], Some(format!("{}:{}", w.path, hit.line))));
            }
        }
        Ok(checks)
    }

    /// P9: when the task may edit tests, the session's tests must fail on the
    /// pre-session code. With P8 (the full suite passes on the final tree) this is
    /// SWT-Bench's reproducing-test condition (Mündler et al., arXiv:2406.12952,
    /// §3.1): at least one fail-to-pass test and none that go from pass to fail.
    /// Tests that pass on the unfixed code do not discriminate between broken and fixed.
    fn pass_falsify(
        &self,
        claim: &Claim,
        ev: &SessionEvidence<'_>,
        compiled: &policy::Compiled,
        argv: Option<&[String]>,
    ) -> Result<Vec<Check>, KernelError> {
        let (true, ClaimKind::TestsPass, Some(reexec), Some(argv), Some(start), Some(end)) =
            (self.policy.tests_writable, claim.kind, &self.reexec, argv, ev.start_tree, ev.end_tree)
        else {
            return Ok(Vec::new());
        };
        let tests: Vec<String> = ev.writes.iter().filter(|w| compiled.is_test(w.path)).map(|w| w.path.to_owned()).collect();
        if tests.is_empty() {
            return Ok(Vec::new());
        }
        let probe = exhibit::snapshot::overlay(&self.docket_git, start, end, &tests)?;
        let run = reexec.run(Source::Tree { docket_git: &self.docket_git, tree: &probe }, argv)?;
        if let Some(why) = run.unavailable {
            return Ok(vec![unresolved(PassId::Falsify, "falsify-unavailable", why)]);
        }
        let discriminates = run.exit_codes.iter().all(|c| *c != 0);
        let seqs = ev.writes.iter().filter(|w| compiled.is_test(w.path)).map(|w| w.seq).collect();
        Ok(vec![check(
            PassId::Falsify,
            discriminates,
            "tests-do-not-discriminate",
            seqs,
            Some(format!("new tests on pre-session code: exit codes {:?}", run.exit_codes)),
        )])
    }

    /// P8: re-run the policy test command on the claimed commit or the final observed tree.
    fn pass_reexec(&self, claim: &Claim, ev: &SessionEvidence<'_>, argv: Option<&[String]>) -> Result<Vec<Check>, KernelError> {
        let (Some(reexec), Some(argv), true) = (&self.reexec, argv, is_test_like(claim.kind)) else { return Ok(Vec::new()) };
        let source = match (&claim.commit, ev.end_tree) {
            (Some(commit), _) => Source::Commit { repo: &self.repo, commit, ledger: &self.docket_git },
            (None, Some(tree)) => Source::Tree { docket_git: &self.docket_git, tree },
            (None, None) => {
                return Ok(vec![unresolved(
                    PassId::Reexec,
                    "nothing-to-reexecute",
                    "no commit named and no snapshot recorded before the claim".into(),
                )]);
            }
        };
        let run = reexec.run(source, argv)?;
        if let Some(why) = run.unavailable {
            return Ok(vec![unresolved(PassId::Reexec, "reexec-unavailable", why)]);
        }
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

    /// A kernel for one task: its test command is the only accepted test command.
    pub fn for_task(
        test_command: &str,
        allowed: Option<Vec<String>>,
        tests_writable: bool,
        docket_git: PathBuf,
        repo: PathBuf,
        sandbox: cell::Sandbox,
    ) -> Self {
        let mut policy = Policy::new(vec![test_command.to_owned()]);
        if tests_writable {
            policy = policy.with_writable_tests();
        }
        policy.allowed = allowed;
        Self { policy, disabled: BTreeSet::new(), reexec: Some(Reexec { sandbox, runs: 2 }), docket_git, repo }
    }

    /// Examine the docket, record verdicts for claims not yet ruled on, and
    /// return the findings for `task`.
    pub fn adjudicate(&self, docket: &Docket, kernel: &Identity, ts: &str, task: &str) -> Result<Vec<Finding>, KernelError> {
        let records = docket.records()?;
        let ruled: BTreeSet<u64> = Self::settled(&records, self.digest()?).into_keys().collect();
        let findings: Vec<Finding> = self.examine(&records)?.into_iter().filter(|f| f.claim.task == task).collect();
        let fresh: Vec<Finding> = findings.iter().filter(|f| !ruled.contains(&f.claim_seq)).cloned().collect();
        if !fresh.is_empty() {
            self.rule(docket, kernel, ts, &fresh)?;
        }
        Ok(findings)
    }

    /// Append each verdict, signed by the kernel identity. Returns the new entry seqs.
    pub fn rule(&self, docket: &Docket, kernel: &Identity, ts: &str, findings: &[Finding]) -> Result<Vec<u64>, KernelError> {
        let claim_commits: BTreeMap<u64, String> = docket.records()?.into_iter().map(|r| (r.seq, r.commit)).collect();
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
