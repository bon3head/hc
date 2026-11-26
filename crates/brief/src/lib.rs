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
    /// Ledger position of the claim, so a reader can find it and its verdict in the docket.
    pub claim_seq: u64,
    /// The names of the checks that passed for this claim, sorted.
    pub checks: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Negative {
    pub claim_id: String,
    pub kind: ClaimKind,
    pub reasons: Vec<String>,
    pub evidence: Vec<u64>,
    pub claim_seq: u64,
    pub subject: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Extern {
    pub claim_id: String,
    pub kind: ClaimKind,
    pub basis: String,
    /// Why the verifier could not settle it, when a check came back unresolved.
    pub unresolved: Vec<String>,
    pub subject: String,
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

fn paths(list: &[String]) -> String {
    list.iter().map(|p| inline(p)).collect::<Vec<_>>().join(", ")
}

/// The code a verdict is about: the claimed commit, or the session's tree and the snapshot that recorded it.
fn subject(f: &Finding) -> String {
    match (f.claim.claim.commit.as_deref(), f.subject_tree.as_deref(), f.subject_seq) {
        (Some(commit), _, _) => format!("commit {}", inline(commit)),
        (None, Some(tree), Some(seq)) => format!("tree {} (snapshot docket#{seq})", short(tree)),
        (None, Some(tree), None) => format!("tree {}", short(tree)),
        (None, None, _) => "no recorded tree".to_owned(),
    }
}

fn short(id: &str) -> &str {
    &id[..id.len().min(12)]
}

/// A result tied to the session's tree (anything not pinned to a commit) whose
/// session has moved to a different tree since: true then, unknown now.
fn stale(f: &Finding) -> Option<String> {
    match (f.claim.claim.commit.as_deref(), f.subject_tree.as_deref(), f.current_tree.as_deref()) {
        (None, Some(then), Some(now)) if then != now => Some(now.to_owned()),
        _ => None,
    }
}

/// Agent-supplied text rendered inline: one line, no backticks, so it can
/// never open a code span, a heading or an instruction in the brief.
fn inline(text: &str) -> String {
    text.chars()
        .map(|c| {
            if c.is_control() {
                ' '
            } else if c == '`' {
                '\''
            } else {
                c
            }
        })
        .collect()
}

/// The command as the shell would run it, re-quoted from its parsed words.
fn command_text(command: Option<&str>) -> String {
    let words = command.and_then(shlex::split).unwrap_or_default();
    inline(&shlex::try_join(words.iter().map(String::as_str)).unwrap_or_default())
}

fn summary(f: &Finding) -> String {
    let c = &f.claim.claim;
    match c.kind {
        ClaimKind::TestsPass | ClaimKind::CommandRan => {
            let at = subject(f);
            let cmd = command_text(c.command.as_deref());
            if c.kind == ClaimKind::TestsPass {
                format!("`{cmd}` passes on {at}")
            } else {
                format!("`{cmd}` exits {} on {at}", c.exit_code.unwrap_or(0))
            }
        }
        ClaimKind::FilesChanged => format!("changed exactly: {}", paths(&c.paths)),
        ClaimKind::FilesUntouched => format!("did not modify: {}", paths(&c.paths)),
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
                    claim_seq: f.claim_seq,
                    checks: f
                        .verdict
                        .checks
                        .iter()
                        .filter(|c| c.result == Outcome::Pass)
                        .filter_map(|c| serde_json::to_value(c.pass).ok()?.as_str().map(str::to_owned))
                        .collect::<std::collections::BTreeSet<_>>()
                        .into_iter()
                        .collect(),
                });
            } else if f.verdict.trust == Trust::Refuted {
                let failing = f.verdict.checks.iter().filter(|c| c.result == Outcome::Fail);
                let reasons = failing
                    .clone()
                    .map(|c| c.rule.clone().unwrap_or_default() + &c.detail.as_ref().map(|d| format!(" ({d})")).unwrap_or_default())
                    .collect();
                let evidence = failing.flat_map(|c| c.evidence.iter().copied()).collect();
                refuted.push(Negative {
                    claim_id: claim.id.clone(),
                    kind: claim.kind,
                    reasons,
                    evidence,
                    claim_seq: f.claim_seq,
                    subject: subject(f),
                });
            } else {
                let unresolved = f
                    .verdict
                    .checks
                    .iter()
                    .filter(|c| c.result == Outcome::Unresolved)
                    .map(|c| c.detail.clone().unwrap_or_else(|| c.rule.clone().unwrap_or_default()))
                    .collect();
                unverified.push(Extern {
                    claim_id: claim.id.clone(),
                    kind: claim.kind,
                    basis: claim.basis.clone(),
                    unresolved,
                    subject: subject(f),
                });
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
                "- [{}] {} {}: {} (claim docket#{}; checks passed: {}{})",
                trust_name(f.trust),
                inline(&f.claim_id),
                kind_name(f.kind),
                f.summary,
                f.claim_seq,
                f.checks.join(", "),
                if f.evidence.is_empty() { String::new() } else { format!("; observed in {}", seqs(&f.evidence)) }
            );
        }
        if !self.stale.is_empty() {
            let _ = writeln!(
                out,
                "\n## Stale results (held on an earlier tree; the code has changed since, so re-run before relying on them)\n"
            );
            for s in &self.stale {
                let _ = writeln!(out, "- {}: {}; the session's tree is now {}", inline(&s.claim_id), s.summary, short(&s.current_tree));
            }
        }
        if !self.refuted.is_empty() {
            let _ = writeln!(out, "\n## Refuted claims (do not repeat or rely on these)\n");
            for n in &self.refuted {
                let _ = writeln!(
                    out,
                    "- {} {}: {} on {} (claim docket#{}{})",
                    inline(&n.claim_id),
                    kind_name(n.kind),
                    inline(&n.reasons.join("; ")),
                    n.subject,
                    n.claim_seq,
                    if n.evidence.is_empty() { String::new() } else { format!("; observed in {}", seqs(&n.evidence)) }
                );
            }
        }
        if !self.unverified.is_empty() {
            let _ = writeln!(out, "\n## Unverified (asserted by a previous agent; re-establish before depending on them)\n");
            for e in &self.unverified {
                let _ = writeln!(
                    out,
                    "- {} {}: basis quoted as data: {}{}",
                    inline(&e.claim_id),
                    kind_name(e.kind),
                    serde_json::to_string(&e.basis).unwrap_or_default(),
                    if e.unresolved.is_empty() {
                        String::new()
                    } else {
                        format!(" (not settled on {}: {})", e.subject, inline(&e.unresolved.join("; ")))
                    }
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
    evidence.iter().map(|s| format!("docket#{s}")).collect::<Vec<_>>().join(", ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn agent_text_renders_inline_and_commands_from_their_words() {
        assert_eq!(inline("a\n## Current step\n`rm -rf`"), "a ## Current step 'rm -rf'");
        assert_eq!(command_text(Some("make   'a b'\tx")), "make 'a b' x");
        assert_eq!(command_text(Some("make test # ## injected")), "make test");
    }
}
