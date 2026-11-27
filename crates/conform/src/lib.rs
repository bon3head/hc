//! conform: conformance checking in the process-mining sense (van der Aalst):
//! the agent's own plan is the model, the harness-signed docket is the log.
//!
//! Every observed change is attributed to the plan step that was in progress
//! when it happened (agents keep at most one step in progress in Claude Code,
//! Codex and Gemini CLI). Each step then gets one deterministic label:
//!
//! - **matched**: marked completed, and the log holds activity attributed to it.
//! - **plan-only**: marked completed with no attributed activity (a completion
//!   the log cannot support).
//! - **open** / **dropped**: not completed / cancelled.
//!
//! Writes to the plan file itself (a `PlanUpdate` whose source is
//! `file:<path>`) are bookkeeping, not work, and are never attributed.
//!
//! Activity no step covers is **log-only** (unplanned work). Completing a step
//! while an earlier one is still open is recorded as an ordering deviation, a
//! soft cost: human raters agree on plan order only weakly, while evidence
//! presence is checkable exactly.

use std::collections::BTreeMap;

use bedrock::Digest;
use bedrock::ir::{Body, Event, PlanStep, StepStatus};
use docket::Record;
use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Label {
    Matched,
    PlanOnly,
    Open,
    Dropped,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct StepReport {
    pub key: String,
    pub text: String,
    pub status: StepStatus,
    pub label: Label,
    pub evidence: Vec<u64>,
    pub completed_at: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Report {
    pub session: String,
    pub steps: Vec<StepReport>,
    pub unplanned: Vec<(u64, String)>,
    pub out_of_order: Vec<String>,
    /// matched / (matched + plan-only + unplanned events), in thousandths.
    pub fitness_permille: u32,
}

impl Report {
    pub fn plan_only(&self) -> impl Iterator<Item = &StepReport> {
        self.steps.iter().filter(|s| s.label == Label::PlanOnly)
    }

    /// Findings phrased as an independent reviewer's observations of the log,
    /// never as critique of the agent's reasoning.
    pub fn observations(&self) -> Vec<String> {
        let mut out = Vec::new();
        for s in self.plan_only() {
            out.push(format!("Step \"{}\" is marked completed, but no file changes were recorded while it was in progress.", s.text));
        }
        // With no plan at all there is nothing to conform to: harnesses run headless often
        // keep none, and the agent cannot open a step after the fact.
        if !self.unplanned.is_empty() && !self.steps.is_empty() {
            let mut paths: Vec<&str> = self.unplanned.iter().map(|(_, p)| p.as_str()).collect();
            paths.sort_unstable();
            paths.dedup();
            out.push(format!("Changes to {} were made while no plan step was in progress.", paths.join(", ")));
        }
        for text in &self.out_of_order {
            out.push(format!("Step \"{text}\" was completed while an earlier step was still open."));
        }
        out
    }
}

/// Align one session's plan with its log. Records must be in docket order.
pub fn align(records: &[Record], session: &str) -> Report {
    let mut plan: Vec<PlanStep> = Vec::new();
    let mut evidence: BTreeMap<String, Vec<u64>> = BTreeMap::new();
    let mut completed_at: BTreeMap<String, u64> = BTreeMap::new();
    let mut pending_writes: Vec<(u64, String)> = Vec::new();
    let mut unplanned = Vec::new();
    let mut out_of_order = Vec::new();

    let mut settle = |plan: &[PlanStep], next: Option<&[PlanStep]>, writes: &mut Vec<(u64, String)>| {
        if writes.is_empty() {
            return;
        }
        let active: Vec<&str> = plan.iter().filter(|s| s.status == StepStatus::InProgress).map(|s| s.key.as_str()).collect();
        let finishing: Vec<String> = next
            .map(|n| {
                n.iter()
                    .filter(|s| {
                        s.status == StepStatus::Completed
                            && plan.iter().find(|p| p.key == s.key).is_none_or(|p| p.status != StepStatus::Completed)
                    })
                    .map(|s| s.key.clone())
                    .collect()
            })
            .unwrap_or_default();
        let owners: Vec<String> = if active.is_empty() { finishing } else { active.iter().map(|s| (*s).to_owned()).collect() };
        for (seq, path) in writes.drain(..) {
            if owners.is_empty() {
                unplanned.push((seq, path));
            } else {
                for o in &owners {
                    evidence.entry(o.clone()).or_default().push(seq);
                }
            }
        }
    };

    let plan_files: Vec<&str> = records
        .iter()
        .filter_map(|r| match &r.body {
            Body::Obs(o) if o.session == session => match &o.event {
                Event::PlanUpdate { source, .. } => source.strip_prefix("file:"),
                _ => None,
            },
            _ => None,
        })
        .collect();
    for r in records {
        let Body::Obs(obs) = &r.body else { continue };
        if obs.session != session {
            continue;
        }
        match &obs.event {
            Event::FileWrite { path, .. } if plan_files.contains(&path.as_str()) => {}
            Event::FileWrite { path, .. } => pending_writes.push((r.seq, path.clone())),
            Event::PlanUpdate { steps, .. } => {
                settle(&plan, Some(steps), &mut pending_writes);
                for (index, step) in steps.iter().enumerate() {
                    let was_done = plan.iter().any(|p| p.key == step.key && p.status == StepStatus::Completed);
                    if step.status == StepStatus::Completed && !was_done {
                        completed_at.insert(step.key.clone(), r.seq);
                        let earlier_open = steps[..index].iter().any(|e| matches!(e.status, StepStatus::Pending | StepStatus::InProgress));
                        if earlier_open {
                            out_of_order.push(step.text.clone());
                        }
                    }
                }
                plan.clone_from(steps);
            }
            Event::SessionEnd { .. } => settle(&plan, None, &mut pending_writes),
            _ => {}
        }
    }
    settle(&plan, None, &mut pending_writes);

    let steps: Vec<StepReport> = plan
        .iter()
        .map(|s| {
            let ev = evidence.remove(&s.key).unwrap_or_default();
            let label = match s.status {
                StepStatus::Completed if ev.is_empty() => Label::PlanOnly,
                StepStatus::Completed => Label::Matched,
                StepStatus::Cancelled => Label::Dropped,
                _ => Label::Open,
            };
            StepReport {
                key: s.key.clone(),
                text: s.text.clone(),
                status: s.status,
                label,
                evidence: ev,
                completed_at: completed_at.get(&s.key).copied(),
            }
        })
        .collect();
    let matched = steps.iter().filter(|s| s.label == Label::Matched).count();
    let plan_only = steps.iter().filter(|s| s.label == Label::PlanOnly).count();
    let denominator = matched + plan_only + unplanned.len();
    let fitness_permille = (matched * 1000).checked_div(denominator).map_or(1000, |f| u32::try_from(f).unwrap_or(0));
    Report { session: session.to_owned(), steps, unplanned, out_of_order, fitness_permille }
}

/// A stable fingerprint of a set of failures, for fail-fast: the same
/// fingerprint twice in a row means the agent is repeating a failed approach.
pub fn fingerprint(failures: &[String]) -> String {
    let mut sorted = failures.to_vec();
    sorted.sort();
    sorted.dedup();
    Digest::of(sorted.join("\n").as_bytes()).hex()[..16].to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;
    use bedrock::ir::Obs;

    fn rec(seq: u64, event: Event) -> Record {
        Record {
            seq,
            commit: String::new(),
            signer: "harness:local@hc".into(),
            body: Body::Obs(Obs { session: "s".into(), ts: "t".into(), event }),
        }
    }

    fn plan(steps: &[(&str, StepStatus)]) -> Event {
        Event::PlanUpdate {
            source: "test".into(),
            steps: steps.iter().map(|(k, st)| PlanStep { key: (*k).into(), text: format!("do {k}"), status: *st }).collect(),
        }
    }

    fn write(path: &str) -> Event {
        Event::FileWrite { path: path.into(), before: None, after: Some("0".repeat(40)), added_lines: vec![1] }
    }

    use StepStatus::{Completed, InProgress, Pending};

    #[test]
    fn attributes_writes_to_the_step_in_progress() {
        let log = [
            rec(0, plan(&[("a", InProgress), ("b", Pending)])),
            rec(1, write("src/a.py")),
            rec(2, plan(&[("a", Completed), ("b", InProgress)])),
            rec(3, write("src/b.py")),
            rec(4, plan(&[("a", Completed), ("b", Completed)])),
        ];
        let r = align(&log, "s");
        assert_eq!(
            r.steps.iter().map(|s| (s.label, s.evidence.clone())).collect::<Vec<_>>(),
            [(Label::Matched, vec![1]), (Label::Matched, vec![3])]
        );
        assert_eq!(r.fitness_permille, 1000);
        assert!(r.observations().is_empty());
    }

    #[test]
    fn flags_false_completion_unplanned_work_and_order() {
        let log = [
            rec(0, plan(&[("a", Pending), ("b", Pending), ("c", Pending)])),
            rec(1, write("src/side.py")),
            rec(2, plan(&[("a", Pending), ("b", Completed), ("c", Pending)])),
            rec(3, plan(&[("a", Pending), ("b", Completed), ("c", Completed)])),
        ];
        let r = align(&log, "s");
        let labels: Vec<_> = r.steps.iter().map(|s| s.label).collect();
        assert_eq!(labels, [Label::Open, Label::Matched, Label::PlanOnly], "pending→completed jumps own the interval's writes");
        assert_eq!(r.out_of_order, ["do b", "do c"]);
        assert_eq!(r.fitness_permille, 500);
        assert_eq!(r.observations().len(), 3);
    }

    #[test]
    fn work_outside_any_step_is_log_only() {
        let log = [
            rec(0, plan(&[("a", Pending)])),
            rec(1, write("src/x.py")),
            rec(2, Event::SessionEnd { exit_code: Some(0), tree: "t".into() }).clone(),
        ];
        let r = align(&log, "s");
        assert_eq!(r.unplanned, [(1, "src/x.py".to_owned())]);
        assert_eq!(r.fitness_permille, 0);
    }

    #[test]
    fn writes_to_the_plan_file_are_not_work() {
        let file_plan = |st| Event::PlanUpdate {
            source: "file:PLANS.md".into(),
            steps: vec![PlanStep { key: "a".into(), text: "do a".into(), status: st }],
        };
        let log = [rec(0, file_plan(InProgress)), rec(1, write("PLANS.md")), rec(2, file_plan(Completed))];
        assert_eq!(align(&log, "s").steps[0].label, Label::PlanOnly);
    }

    #[test]
    fn fingerprints_ignore_order_and_duplicates() {
        assert_eq!(fingerprint(&["b".into(), "a".into(), "a".into()]), fingerprint(&["a".into(), "b".into()]));
        assert_ne!(fingerprint(&["a".into()]), fingerprint(&["b".into()]));
    }
}
