//! agenda: reads the plan the agent already keeps; never prescribes one.
//!
//! Supported native shapes (field names from each vendor's documentation or source):
//!
//! | Source | Tool | Shape |
//! |---|---|---|
//! | Claude Code | `TodoWrite` | `{todos: [{content, status, activeForm}]}`, whole-list replace |
//! | Claude Code | `TaskCreate` / `TaskUpdate` | `{subject, description}` → `{task: {id}}`; `{taskId, status}` |
//! | Codex | `update_plan` | `{explanation?, plan: [{step, status}]}`, whole-list replace |
//! | Gemini CLI | `write_todos` | `{todos: [{description, status}]}`, whole-list replace |
//! | ACP | `plan` session update | `{entries: [{content, priority, status}]}`, whole-list replace |
//! | Antigravity and others | Markdown task artifacts | `- [ ]`, `- [/]`, `- [x]`, `- [-]` checklist lines |
//! | Codex ExecPlan files | `PLANS.md` `## Progress` | `- [x] (2025-10-01 13:00Z) step` (OpenAI Cookbook, "Using `PLANS.md`") |
//!
//! Whole-list sources carry no step ids, so `Tracker` derives stable keys the
//! way patience diff (Bram Cohen) anchors lines: steps whose normalized text is
//! unique in both versions match first; repeated texts then match in order;
//! a remaining step matches the most similar remaining previous step when its
//! token-set Jaccard similarity is at least one half (ties go to the nearest
//! position). Anything left is a new step, and a previous step with no match
//! is kept as cancelled so its history is not lost.

use std::collections::{BTreeMap, BTreeSet};

use bedrock::Digest;
use bedrock::ir::{PlanStep, StepStatus};
use serde_json::Value;

/// A plan as read from one native update, before step keys are assigned.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Update {
    /// The full list (`TodoWrite`, `update_plan`, `write_todos`, ACP, Markdown).
    Replace(Vec<(Option<String>, String, StepStatus)>),
    /// One new step with its native id (`TaskCreate` and its tool result).
    Create { id: String, text: String },
    /// A status change on a native id (`TaskUpdate`).
    SetStatus { id: String, status: StepStatus },
    /// A native deletion (`TaskUpdate` with status `deleted`).
    Delete { id: String },
}

pub fn status(raw: &str) -> Option<StepStatus> {
    Some(match raw.to_ascii_lowercase().replace(['-', ' '], "_").as_str() {
        "pending" | "open" | "todo" | "not_started" => StepStatus::Pending,
        "in_progress" | "inprogress" | "active" | "doing" => StepStatus::InProgress,
        "completed" | "complete" | "done" | "closed" => StepStatus::Completed,
        "cancelled" | "canceled" | "skipped" => StepStatus::Cancelled,
        "blocked" => StepStatus::Blocked,
        _ => return None,
    })
}

fn list(items: Option<&Value>, text_field: &str) -> Option<Update> {
    let steps = items?
        .as_array()?
        .iter()
        .filter_map(|item| {
            let text = item.get(text_field)?.as_str()?.trim().to_owned();
            let status = status(item.get("status")?.as_str()?)?;
            Some((None, text, status))
        })
        .collect();
    Some(Update::Replace(steps))
}

/// Parse a native plan tool call. `response` is the tool result when the harness supplies it.
pub fn from_tool(tool: &str, input: &Value, response: Option<&Value>) -> Option<Update> {
    match tool {
        "TodoWrite" => list(input.get("todos"), "content"),
        "update_plan" => list(input.get("plan"), "step"),
        "write_todos" => list(input.get("todos"), "description"),
        "TaskCreate" => {
            let id = response?.pointer("/task/id").and_then(value_str)?;
            let text = input.get("subject").and_then(Value::as_str)?.trim().to_owned();
            Some(Update::Create { id, text })
        }
        "TaskUpdate" => {
            let id = ["taskId", "task_id", "id"].iter().find_map(|k| input.get(*k).and_then(value_str))?;
            let raw = input.get("status").and_then(Value::as_str)?;
            if raw == "deleted" { Some(Update::Delete { id }) } else { Some(Update::SetStatus { id, status: status(raw)? }) }
        }
        _ => None,
    }
}

fn value_str(v: &Value) -> Option<String> {
    v.as_str().map(str::to_owned).or_else(|| v.as_u64().map(|n| n.to_string()))
}

/// ACP `session/update` with `sessionUpdate: "plan"`.
pub fn from_acp(update: &Value) -> Option<Update> {
    list(update.get("entries"), "content")
}

/// Markdown checklist lines: `[ ]` pending, `[/]` in progress, `[x]` completed, `[-]` cancelled.
pub fn from_markdown(text: &str) -> Update {
    let steps = text
        .lines()
        .filter_map(|line| {
            let rest = line.trim_start().strip_prefix("- [").or_else(|| line.trim_start().strip_prefix("* ["))?;
            let (mark, item) = rest.split_once(']')?;
            let status = match mark.trim() {
                "" => StepStatus::Pending,
                "/" | "~" => StepStatus::InProgress,
                "x" | "X" => StepStatus::Completed,
                "-" => StepStatus::Cancelled,
                _ => return None,
            };
            Some((None, item.trim().to_owned(), status))
        })
        .collect();
    Update::Replace(steps)
}

/// The `## Progress` checklist of a Codex ExecPlan. Timestamps in the leading
/// parenthesis are dropped so ticking a box does not change the step's text.
pub fn from_exec_plan(text: &str) -> Update {
    let mut in_progress = false;
    let mut section = String::new();
    for line in text.lines() {
        if let Some(heading) = line.strip_prefix("## ") {
            in_progress = heading.trim().eq_ignore_ascii_case("progress");
            continue;
        }
        if in_progress {
            section.push_str(line);
            section.push('\n');
        }
    }
    let Update::Replace(items) = from_markdown(&section) else { unreachable!("from_markdown returns Replace") };
    Update::Replace(
        items
            .into_iter()
            .map(|(id, text, status)| {
                let stripped = match text.strip_prefix('(').and_then(|rest| rest.split_once(')')) {
                    Some((stamp, rest)) if stamp.chars().any(|c| c.is_ascii_digit()) => rest.trim().to_owned(),
                    _ => text,
                };
                (id, stripped, status)
            })
            .collect(),
    )
}

fn normalize(text: &str) -> String {
    text.split_whitespace().map(str::to_lowercase).collect::<Vec<_>>().join(" ")
}

fn tokens(text: &str) -> BTreeSet<String> {
    normalize(text).split(' ').filter(|t| t.len() > 2).map(str::to_owned).collect()
}

/// Token-set Jaccard similarity in thousandths.
fn jaccard_permille(a: &str, b: &str) -> usize {
    let (ta, tb) = (tokens(a), tokens(b));
    let union = ta.union(&tb).count();
    (ta.intersection(&tb).count() * 1000).checked_div(union).unwrap_or(0)
}

/// Folds native updates into a keyed plan.
#[derive(Debug, Clone, Default)]
pub struct Tracker {
    steps: Vec<PlanStep>,
    native: BTreeMap<String, usize>,
}

impl Tracker {
    pub fn steps(&self) -> &[PlanStep] {
        &self.steps
    }

    pub fn restore(steps: Vec<PlanStep>) -> Self {
        let native = steps.iter().enumerate().filter_map(|(i, s)| s.key.strip_prefix("native:").map(|id| (id.to_owned(), i))).collect();
        Self { steps, native }
    }

    pub fn apply(&mut self, update: Update) -> &[PlanStep] {
        match update {
            Update::Create { id, text } => {
                self.native.insert(id.clone(), self.steps.len());
                self.steps.push(PlanStep { key: format!("native:{id}"), text, status: StepStatus::Pending });
            }
            Update::SetStatus { id, status } => {
                if let Some(&i) = self.native.get(&id) {
                    self.steps[i].status = status;
                }
            }
            Update::Delete { id } => {
                if let Some(i) = self.native.remove(&id) {
                    self.steps[i].status = StepStatus::Cancelled;
                }
            }
            Update::Replace(items) => self.replace(items),
        }
        &self.steps
    }

    fn replace(&mut self, items: Vec<(Option<String>, String, StepStatus)>) {
        let previous = std::mem::take(&mut self.steps);
        let old: Vec<String> = previous.iter().map(|p| normalize(&p.text)).collect();
        let new: Vec<String> = items.iter().map(|(_, t, _)| normalize(t)).collect();
        let mut matched: Vec<Option<usize>> = vec![None; items.len()];
        let mut used = vec![false; previous.len()];
        let count = |v: &[String], t: &str| v.iter().filter(|x| *x == t).count();

        for (i, t) in new.iter().enumerate() {
            if count(&new, t) == 1 && count(&old, t) == 1 {
                let j = old.iter().position(|o| o == t).expect("counted once");
                matched[i] = Some(j);
                used[j] = true;
            }
        }
        for (i, t) in new.iter().enumerate() {
            if matched[i].is_none()
                && let Some(j) = (0..old.len()).find(|&j| !used[j] && &old[j] == t)
            {
                matched[i] = Some(j);
                used[j] = true;
            }
        }
        for i in 0..new.len() {
            if matched[i].is_some() {
                continue;
            }
            let best = (0..old.len())
                .filter(|&j| !used[j])
                .map(|j| (jaccard_permille(&old[j], &new[i]), std::cmp::Reverse(j.abs_diff(i)), j))
                .filter(|(score, _, _)| *score >= 500)
                .max();
            if let Some((_, _, j)) = best {
                matched[i] = Some(j);
                used[j] = true;
            }
        }

        let mut next = Vec::with_capacity(items.len());
        for (i, (id, text, status)) in items.into_iter().enumerate() {
            let key = match (id, matched[i]) {
                (Some(native), _) => format!("native:{native}"),
                (None, Some(j)) => previous[j].key.clone(),
                (None, None) => {
                    let mut n = 0;
                    loop {
                        let key = format!("step:{}", &Digest::of(format!("{}#{n}", new[i]).as_bytes()).hex()[..12]);
                        if !next.iter().any(|s: &PlanStep| s.key == key) && !previous.iter().any(|p| p.key == key) {
                            break key;
                        }
                        n += 1;
                    }
                }
            };
            next.push(PlanStep { key, text, status });
        }
        for (j, p) in previous.into_iter().enumerate() {
            if !used[j] && !next.iter().any(|n| n.key == p.key) {
                next.push(PlanStep { status: StepStatus::Cancelled, ..p });
            }
        }
        self.steps = next;
    }
}

/// Steps whose status changed between two plans, keyed by step key.
pub fn transitions(before: &[PlanStep], after: &[PlanStep]) -> Vec<(String, Option<StepStatus>, StepStatus)> {
    let old: BTreeMap<&str, StepStatus> = before.iter().map(|s| (s.key.as_str(), s.status)).collect();
    after
        .iter()
        .filter_map(|s| {
            let from = old.get(s.key.as_str()).copied();
            (from != Some(s.status)).then(|| (s.key.clone(), from, s.status))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn texts(t: &Tracker) -> Vec<(String, StepStatus)> {
        t.steps().iter().map(|s| (s.text.clone(), s.status)).collect()
    }

    #[test]
    fn parses_every_native_shape() {
        let claude = json!({"todos": [{"content": "Write parser", "status": "in_progress", "activeForm": "Writing parser"}]});
        let codex = json!({"explanation": "x", "plan": [{"step": "Write parser", "status": "completed"}]});
        let gemini = json!({"todos": [{"description": "Write parser", "status": "blocked"}]});
        let acp = json!({"entries": [{"content": "Write parser", "priority": "high", "status": "pending"}]});
        assert_eq!(
            from_tool("TodoWrite", &claude, None),
            Some(Update::Replace(vec![(None, "Write parser".into(), StepStatus::InProgress)]))
        );
        assert_eq!(
            from_tool("update_plan", &codex, None),
            Some(Update::Replace(vec![(None, "Write parser".into(), StepStatus::Completed)]))
        );
        assert_eq!(
            from_tool("write_todos", &gemini, None),
            Some(Update::Replace(vec![(None, "Write parser".into(), StepStatus::Blocked)]))
        );
        assert_eq!(from_acp(&acp), Some(Update::Replace(vec![(None, "Write parser".into(), StepStatus::Pending)])));
        assert_eq!(
            from_markdown("# Tasks\n- [x] Read spec\n- [/] Write parser\n- [ ] Add tests\n- [-] Old idea\nnotes"),
            Update::Replace(vec![
                (None, "Read spec".into(), StepStatus::Completed),
                (None, "Write parser".into(), StepStatus::InProgress),
                (None, "Add tests".into(), StepStatus::Pending),
                (None, "Old idea".into(), StepStatus::Cancelled),
            ])
        );
        assert_eq!(
            from_tool(
                "TaskCreate",
                &json!({"subject": "Write parser", "description": "d"}),
                Some(&json!({"task": {"id": "7", "subject": "Write parser"}}))
            ),
            Some(Update::Create { id: "7".into(), text: "Write parser".into() })
        );
        assert_eq!(
            from_tool("TaskUpdate", &json!({"taskId": "7", "status": "completed"}), None),
            Some(Update::SetStatus { id: "7".into(), status: StepStatus::Completed })
        );
        assert_eq!(from_tool("TaskUpdate", &json!({"task_id": "7", "status": "deleted"}), None), Some(Update::Delete { id: "7".into() }));
        assert_eq!(from_tool("Read", &json!({}), None), None);
    }

    #[test]
    fn keys_survive_reordering_rewording_and_removal() {
        let mut t = Tracker::default();
        t.apply(from_markdown("- [/] Write the tokenizer\n- [ ] Add parser tests"));
        let keys: Vec<String> = t.steps().iter().map(|s| s.key.clone()).collect();
        t.apply(from_markdown("- [ ] Add parser tests\n- [x] Write the tokenizer"));
        assert_eq!(t.steps()[1].key, keys[0], "reordered step keeps its key");
        t.apply(from_markdown("- [ ] Add parser unit tests\n- [x] Write the tokenizer"));
        assert_eq!(t.steps()[0].key, keys[1], "reworded step keeps its key");
        t.apply(from_markdown("- [x] Write the tokenizer"));
        assert_eq!(
            texts(&t),
            [("Write the tokenizer".into(), StepStatus::Completed), ("Add parser unit tests".into(), StepStatus::Cancelled)]
        );
    }

    #[test]
    fn exec_plan_progress_only_and_timestamps_dropped() {
        let doc = "# Ship parser\n\n## Purpose / Big Picture\n\n- [ ] not a step\n\n## Progress\n\n- [x] (2025-10-01 13:00Z) Write tokenizer.\n- [ ] Add parser tests (completed: lexer cases; remaining: errors).\n\n## Decision Log\n\n- Decision: x\n";
        let Update::Replace(items) = from_exec_plan(doc) else { panic!() };
        assert_eq!(
            items,
            [
                (None, "Write tokenizer.".into(), StepStatus::Completed),
                (None, "Add parser tests (completed: lexer cases; remaining: errors).".into(), StepStatus::Pending)
            ]
        );
        let mut t = Tracker::default();
        t.apply(from_exec_plan("## Progress\n- [ ] Write tokenizer.\n"));
        let key = t.steps()[0].key.clone();
        t.apply(from_exec_plan("## Progress\n- [x] (2026-11-26 05:30Z) Write tokenizer.\n"));
        assert_eq!(t.steps()[0].key, key, "ticking with a timestamp keeps identity");
    }

    #[test]
    fn duplicate_boilerplate_does_not_cross_match() {
        let mut t = Tracker::default();
        t.apply(from_markdown("- [ ] Run tests\n- [ ] Fix parser\n- [ ] Run tests"));
        let keys: Vec<String> = t.steps().iter().map(|s| s.key.clone()).collect();
        t.apply(from_markdown("- [x] Run tests\n- [/] Fix parser\n- [ ] Run tests"));
        let after: Vec<String> = t.steps().iter().map(|s| s.key.clone()).collect();
        assert_eq!(after, keys);
        assert_ne!(keys[0], keys[2], "identical texts get distinct keys");
    }

    #[test]
    fn native_ids_track_status_changes() {
        let mut t = Tracker::default();
        t.apply(Update::Create { id: "1".into(), text: "A".into() });
        t.apply(Update::Create { id: "2".into(), text: "B".into() });
        let before = t.steps().to_vec();
        t.apply(Update::SetStatus { id: "1".into(), status: StepStatus::InProgress });
        assert_eq!(transitions(&before, t.steps()), [("native:1".into(), Some(StepStatus::Pending), StepStatus::InProgress)]);
        let restored = Tracker::restore(t.steps().to_vec());
        assert_eq!(restored.native.get("2"), Some(&1));
    }
}
