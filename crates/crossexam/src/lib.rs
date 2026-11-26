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
    Body, Check, Claim, ClaimKind, ClaimRecord, Event, Outcome, PREDICATE_VERDICT, PassId, ResourceDescriptor, Trust, VerdictRecord,
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
    /// Ledger position of the kernel's signed verdict, once `adjudicate` has recorded it.
    pub verdict_seq: Option<u64>,
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
    /// Every write recorded since the latest baseline, by any session: what the work
    /// under this contract did to the repository. Checks of tampering, signatures and
    /// "untouched" or "no change" claims use these, so no session choice hides a write.
    writes: Vec<Write<'a>>,
    /// The claim's own session's writes since the baseline, for `files_changed`, which
    /// is a statement about that session's work.
    session_writes: Vec<Write<'a>>,
    /// The code before the work: the baseline's tree, or without a baseline the
    /// claim's session's starting tree.
    start_tree: Option<&'a str>,
    /// The commit checked out at that point, if any.
    start_head: Option<&'a str>,
    /// The repository's latest snapshot before the claim, from any session.
    end_tree: Option<&'a str>,
    /// Ledger position of the observation that recorded `end_tree`.
    end_seq: Option<u64>,
}

fn evidence<'a>(records: &'a [Record], session: &str, before: u64) -> SessionEvidence<'a> {
    let mut ev = SessionEvidence {
        writes: Vec::new(),
        session_writes: Vec::new(),
        start_tree: None,
        start_head: None,
        end_tree: None,
        end_seq: None,
    };
    let baseline = records.iter().filter(|r| r.seq < before).rev().find_map(|r| match &r.body {
        Body::Obs(o) => match &o.event {
            Event::Baseline { tree, head, .. } => Some((r.seq, tree.as_str(), head.as_deref())),
            _ => None,
        },
        _ => None,
    });
    let since = baseline.map_or(0, |(seq, ..)| seq);
    if let Some((_, tree, head)) = baseline {
        ev.start_tree = Some(tree);
        ev.start_head = head;
    }
    for r in records.iter().filter(|r| r.seq < before) {
        let Body::Obs(obs) = &r.body else { continue };
        let own = obs.session == session;
        match &obs.event {
            Event::FileWrite { path, after, added_lines, .. } if r.seq > since => {
                let w = Write { seq: r.seq, path, after: after.as_deref(), added: added_lines };
                if own {
                    ev.session_writes.push(Write { seq: r.seq, path, after: after.as_deref(), added: added_lines });
                }
                ev.writes.push(w);
            }
            Event::SessionStart { tree, head, .. } => {
                if own && baseline.is_none() {
                    ev.start_tree = Some(tree);
                    ev.start_head = head.as_deref();
                }
                ev.end_tree = Some(tree);
                ev.end_seq = Some(r.seq);
            }
            Event::SessionEnd { tree, .. } | Event::Checkpoint { tree } | Event::Baseline { tree, .. } => {
                ev.end_tree = Some(tree);
                ev.end_seq = Some(r.seq);
            }
            Event::FileWrite { .. } | Event::NoClaim { .. } | Event::PlanUpdate { .. } => {}
        }
    }
    ev
}

/// Git's empty tree: an overlay from it removes the listed paths.
const EMPTY_TREE: &str = "4b825dc642cb6eb9a060e54bf8d69288fbee4904";

/// The repository's most recent snapshot, from any session: what "the code now" means
/// to the ledger. Callers that can snapshot the working tree directly override it.
fn latest_tree(records: &[Record]) -> Option<String> {
    records.iter().rev().find_map(|r| match &r.body {
        Body::Obs(o) => match &o.event {
            Event::SessionStart { tree, .. } | Event::Checkpoint { tree } | Event::SessionEnd { tree, .. } => Some(tree.clone()),
            _ => None,
        },
        _ => None,
    })
}

fn is_test_like(kind: ClaimKind) -> bool {
    matches!(kind, ClaimKind::TestsPass | ClaimKind::CommandRan)
}

/// P4: a test claim must name one of the policy's test commands.
fn pass_command(claim: &Claim, argv: Option<&Vec<String>>) -> Option<Check> {
    is_test_like(claim.kind).then(|| check(PassId::Command, argv.is_some(), "command-not-in-policy", vec![], claim.command.clone()))
}

/// P6: a `done` claim is refuted when the session wrote a protected path.
fn pass_tamper(ev: &SessionEvidence<'_>, compiled: &policy::Compiled) -> Check {
    // Whatever status the claim reports: a session that wrote protected paths cannot
    // have any of its claims promoted (a `blocked` label does not excuse gutted tests).
    let hits: Vec<&Write> = ev.writes.iter().filter(|w| compiled.is_protected(w.path)).collect();
    let detail = hits.first().map(|w| w.path.to_owned());
    check(PassId::Tamper, hits.is_empty(), "protected-path-written", hits.iter().map(|w| w.seq).collect(), detail)
}

/// P6S: writes stay in scope and file claims match the observed writes.
fn pass_scope(claim: &Claim, ev: &SessionEvidence<'_>, compiled: &policy::Compiled) -> Vec<Check> {
    let mut checks = Vec::new();
    let written: BTreeSet<&str> = ev.writes.iter().map(|w| w.path).collect();
    let seqs = |pred: &dyn Fn(&Write) -> bool| ev.writes.iter().filter(|w| pred(w)).map(|w| w.seq).collect::<Vec<_>>();
    let session_written: BTreeSet<&str> = ev.session_writes.iter().map(|w| w.path).collect();
    if compiled.is_allowed("").is_some() {
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
            let ok = session_written.len() == claimed.len() && session_written.iter().all(|w| claimed.contains(*w));
            let detail = (!ok).then(|| format!("claimed {claimed:?}, observed {session_written:?}"));
            let seen = ev.session_writes.iter().map(|w| w.seq).collect();
            checks.push(check(PassId::Scope, ok, "files-changed-mismatch", seen, detail));
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
        let latest = latest_tree(records);
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
                    verdict_seq: None,
                    current_tree: latest.clone(),
                });
                continue;
            }
            let argv = claim.command.as_deref().and_then(|c| compiled.test_argv(c));
            let mut checks = Vec::new();
            if self.on(PassId::Command) {
                checks.extend(pass_command(claim, argv.as_ref()));
            }
            if ev.end_tree.is_none() {
                // Nothing recorded about the repository: what was written is unknown, so the
                // write-based passes cannot pass. They would pass vacuously on an empty write set.
                checks.push(unresolved(
                    PassId::Tamper,
                    "no-repository-evidence",
                    "nothing was recorded about this repository before the claim".to_owned(),
                ));
            } else {
                if self.on(PassId::Tamper) {
                    checks.push(pass_tamper(&ev, &compiled));
                }
                if self.on(PassId::Scope) {
                    checks.extend(pass_scope(claim, &ev, &compiled));
                }
                if self.on(PassId::Signature) {
                    checks.extend(self.pass_signature(&ev, &compiled)?);
                }
            }
            if let (Some(commit), Some(_)) = (claim.commit.as_deref(), ev.start_tree) {
                checks.extend(self.pass_commit(commit, &ev, &compiled)?);
            }
            if self.on(PassId::Reexec) {
                checks.extend(self.pass_reexec(claim, &ev, argv.as_deref()));
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
            let current_tree = latest.clone();
            findings.push(Finding {
                claim_seq: record.seq,
                claim: cr.clone(),
                verdict,
                verified,
                subject_tree: ev.end_tree.map(str::to_owned),
                subject_seq: ev.end_seq,
                verdict_seq: None,
                current_tree,
            });
        }
        Ok(findings)
    }

    /// P6 and P6S for a commit-bound claim: the commit is re-run, so its own content is
    /// what the claim is about. Its changes since the session started (against the
    /// commit checked out then, or the starting snapshot) must not touch protected
    /// paths or leave the allowed scope. This catches a commit built with plumbing
    /// that never passed through the working tree the hooks observe.
    fn pass_commit(&self, commit: &str, ev: &SessionEvidence<'_>, compiled: &policy::Compiled) -> Result<Vec<Check>, KernelError> {
        let (base, filter) = match (ev.start_head, ev.start_tree) {
            (Some(head), _) => (head, "AMDT"),
            // No commit at the start: compare with the snapshot, which also holds untracked
            // files a commit would not include, so only additions and modifications count.
            (None, Some(tree)) => (tree, "AMT"),
            (None, None) => return Ok(Vec::new()),
        };
        let Some(changed) = reexec::commit_changes(&self.docket_git, &self.repo, base, commit, filter)? else {
            return Ok(vec![unresolved(
                PassId::Tamper,
                "commit-unreadable",
                format!("cannot compare commit {commit:?} with the session's start"),
            )]);
        };
        let mut checks = Vec::new();
        if self.on(PassId::Tamper) {
            let hit = changed.iter().find(|p| compiled.is_protected(p));
            checks.push(check(PassId::Tamper, hit.is_none(), "protected-path-in-commit", vec![], hit.cloned()));
        }
        if self.on(PassId::Scope) && compiled.is_allowed("").is_some() {
            let out = changed.iter().find(|p| compiled.is_allowed(p) == Some(false) && !compiled.is_protected(p));
            checks.push(check(PassId::Scope, out.is_none(), "out-of-scope-in-commit", vec![], out.cloned()));
        }
        Ok(checks)
    }

    /// P7: cheat signatures on the Python lines a `done` session added.
    fn pass_signature(&self, ev: &SessionEvidence<'_>, compiled: &policy::Compiled) -> Result<Vec<Check>, KernelError> {
        let mut checks = Vec::new();
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
        // Run only the new tests against the unfixed code: every other test file is removed
        // from the probe (helpers such as `__init__.py` or `conftest.py` stay), so a test that
        // already failed before the session cannot make the new ones look discriminating.
        let listing = Git::bare(&self.docket_git).run(&["ls-tree", "-r", "--name-only", start])?;
        let module = |p: &str| p.rsplit('/').next().is_some_and(|f| f.starts_with("test_") || f.ends_with("_test.py"));
        let others: Vec<String> =
            listing.lines().filter(|p| compiled.is_test(p) && module(p) && !tests.iter().any(|t| t == p)).map(str::to_owned).collect();
        let only_new = exhibit::snapshot::overlay(&self.docket_git, start, EMPTY_TREE, &others)?;
        let probe = exhibit::snapshot::overlay(&self.docket_git, &only_new, end, &tests)?;
        let run = match reexec.run(Source::Tree { docket_git: &self.docket_git, tree: &probe }, argv) {
            Ok(run) => run,
            Err(e) => return Ok(vec![unresolved(PassId::Falsify, "falsify-unavailable", format!("could not prepare the probe: {e}"))]),
        };
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
    fn pass_reexec(&self, claim: &Claim, ev: &SessionEvidence<'_>, argv: Option<&[String]>) -> Vec<Check> {
        let (Some(reexec), Some(argv), true) = (&self.reexec, argv, is_test_like(claim.kind)) else { return Vec::new() };
        let source = match (&claim.commit, ev.end_tree) {
            (Some(commit), _) => Source::Commit { repo: &self.repo, commit, ledger: &self.docket_git },
            (None, Some(tree)) => Source::Tree { docket_git: &self.docket_git, tree },
            (None, None) => {
                return vec![unresolved(
                    PassId::Reexec,
                    "nothing-to-reexecute",
                    "no commit named and no snapshot recorded before the claim".into(),
                )];
            }
        };
        // A tree or commit that cannot be materialized affects this claim only, never the brief.
        let run = match reexec.run(source, argv) {
            Ok(run) => run,
            Err(e) => return vec![unresolved(PassId::Reexec, "reexec-unavailable", format!("could not prepare a clean copy: {e}"))],
        };
        if let Some(why) = run.unavailable {
            return vec![unresolved(PassId::Reexec, "reexec-unavailable", why)];
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
        let detail = Some(format!("exit codes {:?}", run.exit_codes));
        if all && counts_ok && claim.kind == ClaimKind::TestsPass && run.passed.is_none_or(|n| n == 0) {
            // Exit 0 with no test report proves only that the process exited 0 (an
            // `os._exit(0)` in the code under test does that before any test runs).
            return vec![unresolved(
                PassId::Reexec,
                "no-test-report",
                format!(
                    "exit codes {:?} but no passing tests were reported (unittest, pytest and cargo test reports are recognized)",
                    run.exit_codes
                ),
            )];
        }
        vec![check(PassId::Reexec, all && counts_ok, rule, vec![], detail)]
    }

    /// Snapshot the working tree now and make it the "current" tree every finding is
    /// compared with, so a fact goes stale after any edit, including one no hook saw.
    /// Snapshots force the default protected patterns, as every recorder does, so equal
    /// code gives equal trees.
    pub fn observe_now(&self, findings: &mut [Finding]) -> Result<(), KernelError> {
        let tree = exhibit::snapshot::snapshot(&self.docket_git, &self.repo, &Policy::new(vec![]).protected)?;
        for f in findings {
            f.current_tree = Some(tree.clone());
        }
        Ok(())
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
        policy.allowed = allowed;
        if tests_writable {
            policy = policy.with_writable_tests();
        }
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
        // Cite each verdict where it now sits in the ledger, so a reader can follow a
        // line of the brief to the kernel's signed record.
        let digest = self.digest()?;
        let recorded: BTreeMap<u64, u64> = docket
            .records()?
            .iter()
            .filter_map(|r| match &r.body {
                Body::Verdict(v) if v.policy == digest => Some((v.claim_seq, r.seq)),
                _ => None,
            })
            .collect();
        let mut findings = findings;
        for f in &mut findings {
            f.verdict_seq = recorded.get(&f.claim_seq).copied();
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
