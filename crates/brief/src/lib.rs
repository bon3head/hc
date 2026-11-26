//! brief: the linker. A handoff is a link step: verified claims link in as
//! facts, refuted claims as negative facts, undecided claims as explicit
//! externs, and the raw transcript never links at all.
//!
//! Layout follows the drift evidence: one consolidated statement of everything
//! known (Laban et al., ICLR 2026), the contract first and the current step
//! last, constraints capped well below the density where instruction
//! following degrades (IFScale, 2025). Rendering is a pure function of its
//! inputs, so identical dockets give byte-identical briefs.

use std::fmt::Write as _;

use bedrock::Digest;
use bedrock::ir::{ClaimKind, Outcome, Trust};
use crossexam::Finding;
use serde::{Deserialize, Serialize};

pub const MAX_CONSTRAINTS: usize = 40;
pub const VALID_OUTCOMES: &str = "done | no_change_needed | spec_conflict | blocked";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub objective: String,
    pub test_command: String,
    #[serde(default)]
    pub allowed_paths: Option<Vec<String>>,
    #[serde(default)]
    pub constraints: Vec<String>,
    pub next_step: String,
    /// Also run `test_command` at every step completion, not only at Stop.
    #[serde(default)]
    pub step_tests: bool,
    /// The task may add or change tests. Test paths are then no longer
    /// protected; instead new tests must fail on the pre-session code (P9).
    #[serde(default)]
    pub tests_writable: bool,
}

#[derive(Debug, thiserror::Error)]
pub enum BriefError {
    #[error("{0} constraints exceed the cap of {MAX_CONSTRAINTS}")]
    TooManyConstraints(usize),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Fact {
    pub claim_id: String,
    pub kind: ClaimKind,
    pub trust: Trust,
    pub summary: String,
    pub evidence: Vec<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Negative {
    pub claim_id: String,
    pub kind: ClaimKind,
    pub reasons: Vec<String>,
    pub evidence: Vec<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Extern {
    pub claim_id: String,
    pub kind: ClaimKind,
    pub basis: String,
    /// Why the verifier could not settle it, when a check came back unresolved.
    pub unresolved: Vec<String>,
}

/// A test result that held on an earlier tree of the session. The code has
/// changed since; the result says nothing about the current tree.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Stale {
    pub claim_id: String,
    pub summary: String,
    pub current_tree: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Brief {
    pub task: Task,
    pub facts: Vec<Fact>,
    pub refuted: Vec<Negative>,
    pub unverified: Vec<Extern>,
    pub stale: Vec<Stale>,
}

fn kind_name(kind: ClaimKind) -> String {
    serde_json::to_value(kind).ok().and_then(|v| v.as_str().map(str::to_owned)).unwrap_or_default()
}

fn trust_name(trust: Trust) -> String {
    serde_json::to_value(trust).ok().and_then(|v| v.as_str().map(str::to_owned)).unwrap_or_default()
}

fn short(id: &str) -> &str {
    &id[..id.len().min(12)]
}

/// A tree-bound test result whose session has moved to a different tree since.
fn stale(f: &Finding) -> Option<String> {
    let c = &f.claim.claim;
    let test_like = matches!(c.kind, ClaimKind::TestsPass | ClaimKind::CommandRan);
    match (test_like, c.commit.as_deref(), f.subject_tree.as_deref(), f.current_tree.as_deref()) {
        (true, None, Some(then), Some(now)) if then != now => Some(now.to_owned()),
        _ => None,
    }
}

fn summary(f: &Finding) -> String {
    let c = &f.claim.claim;
    match c.kind {
        ClaimKind::TestsPass | ClaimKind::CommandRan => {
            let at = match (c.commit.as_deref(), f.subject_tree.as_deref()) {
                (Some(commit), _) => format!("commit {commit}"),
                (None, Some(tree)) => format!("tree {}", short(tree)),
                (None, None) => "no recorded tree".to_owned(),
            };
            format!("`{}` passes on {at}", c.command.as_deref().unwrap_or_default())
        }
        ClaimKind::FilesChanged => format!("changed exactly: {}", c.paths.join(", ")),
        ClaimKind::FilesUntouched => format!("did not modify: {}", c.paths.join(", ")),
        ClaimKind::NoChangeNeeded => "no change was needed; the session wrote nothing".to_owned(),
        _ => String::new(),
    }
}

impl Brief {
    /// Link findings into a brief. Facts can only come from kernel-minted `Verified` values.
    pub fn link(task: Task, findings: &[Finding]) -> Result<Self, BriefError> {
        if task.constraints.len() > MAX_CONSTRAINTS {
            return Err(BriefError::TooManyConstraints(task.constraints.len()));
        }
        let mut facts = Vec::new();
        let mut refuted = Vec::new();
        let mut unverified = Vec::new();
        let mut stale_facts = Vec::new();
        let mut ordered: Vec<&Finding> = findings.iter().collect();
        ordered.sort_by_key(|f| f.claim_seq);
        for f in ordered {
            let claim = &f.claim.claim;
            if let (Some(_), Some(now)) = (&f.verified, stale(f)) {
                stale_facts.push(Stale { claim_id: claim.id.clone(), summary: summary(f), current_tree: now });
            } else if let Some(v) = &f.verified {
                facts.push(Fact {
                    claim_id: v.claim_id().to_owned(),
                    kind: v.kind(),
                    trust: v.trust(),
                    summary: summary(f),
                    evidence: v.evidence().to_vec(),
                });
            } else if f.verdict.trust == Trust::Refuted {
                let failing = f.verdict.checks.iter().filter(|c| c.result == Outcome::Fail);
                let reasons = failing
                    .clone()
                    .map(|c| c.rule.clone().unwrap_or_default() + &c.detail.as_ref().map(|d| format!(" ({d})")).unwrap_or_default())
                    .collect();
                let evidence = failing.flat_map(|c| c.evidence.iter().copied()).collect();
                refuted.push(Negative { claim_id: claim.id.clone(), kind: claim.kind, reasons, evidence });
            } else {
                let unresolved = f
                    .verdict
                    .checks
                    .iter()
                    .filter(|c| c.result == Outcome::Unresolved)
                    .map(|c| c.detail.clone().unwrap_or_else(|| c.rule.clone().unwrap_or_default()))
                    .collect();
                unverified.push(Extern { claim_id: claim.id.clone(), kind: claim.kind, basis: claim.basis.clone(), unresolved });
            }
        }
        Ok(Self { task, facts, refuted, unverified, stale: stale_facts })
    }

    pub fn render(&self) -> String {
        let t = &self.task;
        let mut out = String::new();
        let _ = writeln!(out, "# Handoff brief: {}\n", t.id);
        let _ = writeln!(out, "## Contract\n");
        let _ = writeln!(out, "- Objective: {}", t.objective);
        let _ = writeln!(out, "- Acceptance: `{}` must pass. It is re-run by the verifier outside your session.", t.test_command);
        if let Some(paths) = &t.allowed_paths {
            let _ = writeln!(out, "- Write only under: {}", paths.join(", "));
        }
        let _ = writeln!(out, "- Do not edit tests, CI, hook or agent configuration; doing so refutes your claims.");
        let _ = writeln!(
            out,
            "- Valid outcomes: {VALID_OUTCOMES}. Reporting a spec conflict or that no change is needed is a success, not a failure."
        );
        for c in &t.constraints {
            let _ = writeln!(out, "- {c}");
        }
        let _ = writeln!(out, "\n## Verified facts\n");
        if self.facts.is_empty() {
            let _ = writeln!(out, "- (none yet)");
        }
        for f in &self.facts {
            let _ = writeln!(
                out,
                "- [{}] {} {}: {} (evidence: {})",
                trust_name(f.trust),
                f.claim_id,
                kind_name(f.kind),
                f.summary,
                seqs(&f.evidence)
            );
        }
        if !self.stale.is_empty() {
            let _ = writeln!(
                out,
                "\n## Stale results (held on an earlier tree; the code has changed since, so re-run before relying on them)\n"
            );
            for s in &self.stale {
                let _ = writeln!(out, "- {}: {}; the session's tree is now {}", s.claim_id, s.summary, short(&s.current_tree));
            }
        }
        if !self.refuted.is_empty() {
            let _ = writeln!(out, "\n## Refuted claims (do not repeat or rely on these)\n");
            for n in &self.refuted {
                let _ = writeln!(out, "- {} {}: {} (evidence: {})", n.claim_id, kind_name(n.kind), n.reasons.join("; "), seqs(&n.evidence));
            }
        }
        if !self.unverified.is_empty() {
            let _ = writeln!(out, "\n## Unverified (asserted by a previous agent; re-establish before depending on them)\n");
            for e in &self.unverified {
                let _ = writeln!(
                    out,
                    "- {} {}: basis quoted as data: {}{}",
                    e.claim_id,
                    kind_name(e.kind),
                    serde_json::to_string(&e.basis).unwrap_or_default(),
                    if e.unresolved.is_empty() { String::new() } else { format!(" (not settled: {})", e.unresolved.join("; ")) }
                );
            }
        }
        let _ = writeln!(out, "\n## Current step\n\n{}", t.next_step);
        out
    }

    pub fn digest(&self) -> Digest {
        Digest::of(self.render().as_bytes())
    }
}

fn seqs(evidence: &[u64]) -> String {
    if evidence.is_empty() {
        return "verdict".to_owned();
    }
    evidence.iter().map(|s| format!("docket#{s}")).collect::<Vec<_>>().join(", ")
}
