//! End-to-end hook sessions with payloads shaped exactly as the vendor hook
//! references document them, against real git repositories and real test runs.

use std::cell::Cell;
use std::path::{Path, PathBuf};

use serde_json::{Value, json};

use super::*;
use crate::dialect::{Dialect, EventKind, Verdict, read};
use bedrock::ir::Role;
use seal::Keyring;

struct World {
    _tmp: tempfile::TempDir,
    repo: PathBuf,
    docket: Docket,
    harness: Identity,
    state: PathBuf,
    tick: Cell<u32>,
}

const TEST: &str = "python3 -m unittest discover -s tests -t .";

fn write(repo: &Path, rel: &str, text: &str) {
    let p = repo.join(rel);
    std::fs::create_dir_all(p.parent().unwrap()).unwrap();
    std::fs::write(p, text).unwrap();
}

fn world() -> World {
    let tmp = tempfile::tempdir().unwrap();
    let repo = tmp.path().join("repo");
    write(&repo, "src/__init__.py", "");
    write(&repo, "src/calc.py", "def add(a, b):\n    return a - b\n");
    write(&repo, "tests/__init__.py", "");
    write(
        &repo,
        "tests/test_calc.py",
        "import unittest\nfrom src.calc import add\n\n\nclass T(unittest.TestCase):\n    def test_add(self):\n        self.assertEqual(add(2, 3), 5)\n",
    );
    write(&repo, ".gitignore", "__pycache__/\n");
    let task = json!({"id": "fix-add", "objective": "Make add correct.", "test_command": TEST, "allowed_paths": ["src/**"], "next_step": "Fix it."});
    write(&repo, ".handoff/task.json", &task.to_string());
    Git::default().run(&["init", "-q", repo.to_str().unwrap()]).unwrap();
    let keys = Keyring::new(tmp.path().join("keys"));
    let harness = keys.ensure(Role::Harness, "local").unwrap();
    let docket = Docket::open(&tmp.path().join("docket.git"), keys).unwrap();
    World { state: tmp.path().join("state"), _tmp: tmp, repo, docket, harness, tick: Cell::new(0) }
}

impl World {
    fn hook(&self, dialect: Dialect, event: &str, payload: Value) -> Verdict {
        let clock = || {
            self.tick.set(self.tick.get() + 1);
            format!("2026-01-01T00:{:02}:{:02}Z", self.tick.get() / 60, self.tick.get() % 60)
        };
        let env = Env {
            docket: &self.docket,
            harness: &self.harness,
            state_dir: self.state.clone(),
            sandbox: cell::Sandbox::none(Path::new("/nonexistent")),
            clock: &clock,
        };
        let mut raw = payload;
        match dialect {
            Dialect::Claude | Dialect::Mcp => {
                raw["session_id"] = json!("sess-1");
                raw["cwd"] = json!(self.repo);
                raw["hook_event_name"] = json!(event);
            }
            Dialect::Antigravity => {
                raw["conversationId"] = json!("conv-1");
                raw["workspacePaths"] = json!([self.repo]);
            }
        }
        handle(&env, dialect, &read(dialect, EventKind::parse(event).unwrap(), &raw)).unwrap()
    }

    fn todo(&self, items: &[(&str, &str)]) -> Verdict {
        let todos: Vec<Value> = items.iter().map(|(c, s)| json!({"content": c, "status": s, "activeForm": c})).collect();
        self.hook(Dialect::Claude, "PostToolUse", json!({"tool_name": "TodoWrite", "tool_input": {"todos": todos}, "tool_response": {}}))
    }

    fn stop(&self) -> Verdict {
        self.hook(Dialect::Claude, "Stop", json!({"stop_hook_active": false, "last_assistant_message": "done"}))
    }

    /// File a claim as `hc claim` does: checkpoint the hook session, then record.
    fn claim(&self, kind: bedrock::ir::ClaimKind, status: bedrock::ir::Status) {
        let clock = || "2026-01-01T01:00:00Z".to_owned();
        let env = Env {
            docket: &self.docket,
            harness: &self.harness,
            state_dir: self.state.clone(),
            sandbox: cell::Sandbox::none(Path::new("/nonexistent")),
            clock: &clock,
        };
        assert!(checkpoint_recorded(&env, "claude-sess-1", &self.repo).unwrap());
        let agent = self.docket.keys().ensure(Role::Agent, "claude-code").unwrap();
        let claim = bedrock::ir::Claim {
            id: "c1".into(),
            kind,
            basis: "the test contradicts the spec".into(),
            command: Some(TEST.into()),
            exit_code: Some(1),
            passed: None,
            failed: None,
            paths: vec![],
            commit: None,
            depends_on: vec![],
            status,
        };
        let attestation = bedrock::ir::AgentAttestation {
            agent_id: "claude-code".into(),
            level: bedrock::ir::AttestationLevel::Declared,
            executable: None,
            pid: None,
        };
        let envelope = bedrock::ir::ClaimEnvelope { task: "fix-add".into(), claims: vec![claim] };
        exhibit::record_claims(&self.docket, &agent, &clock(), "claude-sess-1", &attestation, &envelope).unwrap();
    }

    fn report(&self, session: &str) -> conform::Report {
        conform::align(&self.docket.records().unwrap(), session)
    }
}

fn text(v: &Verdict) -> &str {
    match v {
        Verdict::Allow => "",
        Verdict::Deny(t) | Verdict::Observe(t) | Verdict::Block(t) => t,
    }
}

#[test]
fn honest_session_is_silent_and_fully_conformant() {
    let w = world();
    assert_eq!(w.hook(Dialect::Claude, "SessionStart", json!({"source": "startup"})), Verdict::Allow);
    assert_eq!(w.todo(&[("Fix add", "in_progress"), ("Document add", "pending")]), Verdict::Allow);
    write(&w.repo, "src/calc.py", "def add(a, b):\n    return a + b\n");
    assert_eq!(w.todo(&[("Fix add", "completed"), ("Document add", "in_progress")]), Verdict::Allow);
    write(&w.repo, "src/calc.py", "def add(a, b):\n    \"\"\"Sum.\"\"\"\n    return a + b\n");
    assert_eq!(w.todo(&[("Fix add", "completed"), ("Document add", "completed")]), Verdict::Allow);
    assert_eq!(w.stop(), Verdict::Allow);
    let r = w.report("claude-sess-1");
    assert_eq!(r.fitness_permille, 1000);
    assert!(r.steps.iter().all(|s| s.label == conform::Label::Matched));
}

#[test]
fn completion_without_activity_is_observed_once() {
    let w = world();
    w.todo(&[("Fix add", "in_progress")]);
    let v = w.todo(&[("Fix add", "completed")]);
    assert!(matches!(v, Verdict::Observe(_)), "{v:?}");
    assert!(text(&v).contains("no file changes were recorded"));
    assert!(text(&v).starts_with(HEADER), "framed as an external observation");
    write(&w.repo, "src/calc.py", "def add(a, b):\n    return a + b\n");
    assert_eq!(w.stop(), Verdict::Observe(message(&["Changes to src/calc.py were made while no plan step was in progress.".into()])));
    assert_eq!(w.stop(), Verdict::Allow, "observations are delivered once");
}

#[test]
fn protected_writes_are_denied_before_they_happen() {
    let w = world();
    let deny = w.hook(
        Dialect::Claude,
        "PreToolUse",
        json!({"tool_name": "Edit", "tool_input": {"file_path": w.repo.join("tests/test_calc.py"), "old_string": "5", "new_string": "-1"}}),
    );
    assert!(matches!(deny, Verdict::Deny(_)) && text(&deny).contains("tests/test_calc.py"));
    let patch = "*** Begin Patch\n*** Update File: .codex/hooks.json\n@@\n-x\n+y\n*** End Patch";
    assert!(matches!(
        w.hook(Dialect::Claude, "PreToolUse", json!({"tool_name": "apply_patch", "tool_input": {"input": patch}})),
        Verdict::Deny(_)
    ));
    assert_eq!(
        w.hook(Dialect::Claude, "PreToolUse", json!({"tool_name": "Write", "tool_input": {"file_path": "src/calc.py", "content": ""}})),
        Verdict::Allow
    );
}

#[test]
fn a_test_edit_made_through_the_shell_blocks_the_step() {
    let w = world();
    w.todo(&[("Make tests pass", "in_progress")]);
    write(&w.repo, "tests/test_calc.py", "import unittest\n\n\nclass T(unittest.TestCase):\n    def test_add(self):\n        pass\n");
    let v = w.todo(&[("Make tests pass", "completed")]);
    assert!(matches!(v, Verdict::Block(_)), "{v:?}");
    assert!(text(&v).contains("tests/test_calc.py was modified"));
}

#[test]
fn same_failure_twice_escalates_to_root_cause_then_lets_go() {
    let w = world();
    w.todo(&[("Fix add", "in_progress")]);
    let first = w.stop();
    assert!(matches!(first, Verdict::Block(_)) && text(&first).contains("exits 1 on a clean copy"), "{first:?}");
    let second = w.stop();
    assert!(text(&second).contains("same failure as the previous attempt"), "{second:?}");
    assert_eq!(w.stop(), Verdict::Allow, "fail-fast never loops");
    write(&w.repo, "tests/test_calc.py", "import unittest\n");
    let fresh = w.stop();
    assert!(
        matches!(fresh, Verdict::Block(_)) && text(&fresh).contains("tests/test_calc.py was modified"),
        "a new failure is not masked by an old escalation: {fresh:?}"
    );
}

#[test]
fn a_fix_clears_the_failure_fingerprint() {
    let w = world();
    w.todo(&[("Fix add", "in_progress")]);
    assert!(matches!(w.stop(), Verdict::Block(_)));
    write(&w.repo, "src/calc.py", "def add(a, b):\n    return a + b\n");
    w.todo(&[("Fix add", "completed")]);
    assert_eq!(w.stop(), Verdict::Allow);
}

#[test]
fn antigravity_artifact_plans_are_read_and_findings_injected_later() {
    let w = world();
    let artifacts = w.repo.join(".artifacts");
    let task_md = artifacts.join("task.md");
    write(&w.repo, ".artifacts/task.md", "- [/] Fix add\n");
    let post = |path: &Path| json!({"toolCall": {"name": "write_to_file", "args": {"TargetFile": path, "IsArtifact": true}}, "stepIdx": 3, "artifactDirectoryPath": artifacts});
    assert_eq!(w.hook(Dialect::Antigravity, "PostToolUse", post(&task_md)), Verdict::Allow);
    write(&w.repo, ".artifacts/task.md", "- [x] Fix add\n");
    let silent = w.hook(Dialect::Antigravity, "PostToolUse", post(&task_md));
    assert_eq!(render(Dialect::Antigravity, EventKind::PostToolUse, &silent).0, "{}", "PostToolUse cannot inject in Antigravity");
    let injected = w.hook(Dialect::Antigravity, "PostInvocation", json!({"invocationNum": 2, "initialNumSteps": 4}));
    assert!(text(&injected).contains("Fix add"), "{injected:?}");
    let report = w.report("antigravity-conv-1");
    assert_eq!(report.steps.len(), 1);
    assert_eq!(report.steps[0].label, conform::Label::PlanOnly, "the artifact write itself is not work");
}

#[test]
fn claude_task_completed_can_keep_a_step_open() {
    let w = world();
    w.hook(Dialect::Claude, "PostToolUse", json!({"tool_name": "TaskCreate", "tool_input": {"subject": "Fix add", "description": "d"}, "tool_response": {"task": {"id": "1", "subject": "Fix add"}}}));
    w.hook(
        Dialect::Claude,
        "PostToolUse",
        json!({"tool_name": "TaskUpdate", "tool_input": {"taskId": "1", "status": "in_progress"}, "tool_response": {}}),
    );
    write(&w.repo, ".claude/settings.json", "{\"hooks\": {}}\n");
    let v = w.hook(Dialect::Claude, "TaskCompleted", json!({"task_id": "1", "task_subject": "Fix add"}));
    assert!(matches!(v, Verdict::Block(_)), "{v:?}");
    let (_, stderr, code) = render(Dialect::Claude, EventKind::TaskCompleted, &v);
    assert_eq!(code, 2);
    assert!(stderr.contains(".claude/settings.json"));
}

#[test]
fn a_reported_spec_conflict_lets_a_failing_suite_stop() {
    use bedrock::ir::{ClaimKind, Status};
    let w = world();
    assert_eq!(w.hook(Dialect::Claude, "SessionStart", json!({"source": "startup"})), Verdict::Allow);
    write(&w.repo, "src/calc.py", "def add(a, b):\n    return a * b\n");
    w.claim(ClaimKind::SpecConflict, Status::SpecConflict);
    assert_eq!(w.stop(), Verdict::Allow, "a valid outcome is not pushed toward forcing a pass");
    write(&w.repo, "src/calc.py", "def add(a, b):\n    return a ** b\n");
    let after_edit = w.stop();
    assert!(
        matches!(after_edit, Verdict::Block(_)) && text(&after_edit).contains("exits 1 on a clean copy"),
        "a claim filed before later edits does not describe the tree: {after_edit:?}"
    );
}

#[test]
fn a_reported_conflict_does_not_excuse_tampering() {
    use bedrock::ir::{ClaimKind, Status};
    let w = world();
    w.hook(Dialect::Claude, "SessionStart", json!({"source": "startup"}));
    write(&w.repo, "tests/test_calc.py", "import unittest\n");
    w.claim(ClaimKind::Blocked, Status::Blocked);
    let v = w.stop();
    assert!(matches!(v, Verdict::Block(_)) && text(&v).contains("tests/test_calc.py was modified"), "{v:?}");
}

#[test]
fn writes_without_any_plan_are_not_reported_as_unplanned() {
    let w = world();
    w.hook(Dialect::Claude, "SessionStart", json!({"source": "startup"}));
    write(&w.repo, "src/calc.py", "def add(a, b):\n    return a + b\n");
    assert_eq!(w.stop(), Verdict::Allow, "no plan step was ever opened, so there is nothing to conform to");
}
