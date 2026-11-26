//! bailiff: keeps order in the room. It runs as the harness's own hook
//! command and adds no methodology of its own; it reads the plan the agent
//! keeps and checks it against what actually happened.
//!
//! - `PreToolUse`: deny writes to protected paths before they happen.
//! - `PostToolUse` on a native plan tool: checkpoint the tree, record the plan,
//!   check every step that just became completed.
//! - `TaskCompleted` (Claude Code): the same step check, able to keep the task open.
//! - `Stop`: protected writes, cheat signatures and the task's acceptance
//!   command on a clean copy of the current tree, plus plan conformance.
//!
//! Hard failures block and explain. Soft findings (a completion with no
//! recorded activity, unplanned work, steps out of order) are delivered once
//! as observations. Every message is framed as an independent check of the
//! working tree, not a critique of the conversation. In Self-Correction Bench
//! (Tsui, arXiv:2507.02778), 14 open-source non-reasoning models failed to
//! correct 64.5% of errors attributed to themselves that they corrected when the
//! same errors came as external input; the size of that gap for frontier coding
//! agents is unmeasured. Fail-fast: the same failure fingerprint twice in a row escalates to
//! a request for a root-cause report, and the next stop is allowed.

pub mod dialect;

use std::path::{Path, PathBuf};

use bedrock::git::Git;
use bedrock::ir::{Body, Event, Obs, PlanStep, StepStatus};
use brief::Task;
use cell::Sandbox;
use crossexam::policy::{Compiled, Policy};
use crossexam::{Reexec, Source, signatures};
use dialect::{Dialect, EventKind, Input, Verdict};
use docket::{Docket, Record};
use exhibit::{Open, Recorder};
use seal::Identity;
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub use dialect::render;

const HEADER: &str = "hc verifier (an independent check of the working tree, not a review of this conversation):";

#[derive(Debug, thiserror::Error)]
pub enum BailiffError {
    #[error(transparent)]
    Docket(#[from] docket::DocketError),
    #[error(transparent)]
    Git(#[from] bedrock::git::GitError),
    #[error("invalid policy")]
    Policy(#[from] globset::Error),
    #[error("i/o error")]
    Io(#[from] std::io::Error),
    #[error("unreadable hook state")]
    Json(#[from] serde_json::Error),
}

pub struct Env<'a> {
    pub docket: &'a Docket,
    pub harness: &'a Identity,
    pub state_dir: PathBuf,
    pub sandbox: Sandbox,
    pub clock: &'a dyn Fn() -> String,
}

impl std::fmt::Debug for Env<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Env").field("state_dir", &self.state_dir).finish_non_exhaustive()
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct State {
    open: Option<Open>,
    plan: Vec<PlanStep>,
    queued: Vec<String>,
    delivered: Vec<String>,
    last_failure: Option<String>,
    escalated: bool,
}

const WRITE_TOOLS: [&str; 10] = [
    "Write",
    "Edit",
    "MultiEdit",
    "NotebookEdit",
    "apply_patch",
    "write_to_file",
    "replace_file_content",
    "multi_replace_file_content",
    "write_file",
    "replace",
];
const PATH_KEYS: [&str; 7] = ["file_path", "notebook_path", "path", "TargetFile", "target_file", "AbsolutePath", "file"];

/// Paths a write tool call names, relative to `cwd`. Patch bodies (`*** Update File: p`) are scanned too.
pub fn written_paths(tool: &str, input: &Value, cwd: &Path) -> Vec<String> {
    if !WRITE_TOOLS.contains(&tool) {
        return Vec::new();
    }
    let mut paths = Vec::new();
    let mut strings = Vec::new();
    collect_strings(input, &mut strings);
    for key in PATH_KEYS {
        if let Some(p) = input.get(key).and_then(Value::as_str) {
            paths.push(p.to_owned());
        }
    }
    for text in strings {
        for line in text.lines() {
            for marker in ["*** Update File: ", "*** Add File: ", "*** Delete File: ", "*** Move to: "] {
                if let Some(p) = line.strip_prefix(marker) {
                    paths.push(p.trim().to_owned());
                }
            }
        }
    }
    let base = cwd.canonicalize().unwrap_or_else(|_| cwd.to_path_buf());
    let mut out: Vec<String> = paths
        .into_iter()
        .map(|p| {
            let path = PathBuf::from(&p);
            path.strip_prefix(&base).or_else(|_| path.strip_prefix(cwd)).map_or(p.clone(), |r| r.to_string_lossy().into_owned())
        })
        .map(|p| p.trim_start_matches("./").to_owned())
        .collect();
    out.sort();
    out.dedup();
    out
}

fn collect_strings(v: &Value, out: &mut Vec<String>) {
    match v {
        Value::String(s) => out.push(s.clone()),
        Value::Array(a) => a.iter().for_each(|x| collect_strings(x, out)),
        Value::Object(m) => m.values().for_each(|x| collect_strings(x, out)),
        _ => {}
    }
}

fn load_task(cwd: &Path) -> Option<Task> {
    let text = std::fs::read_to_string(cwd.join(".handoff/task.json")).ok()?;
    serde_json::from_str(&text).ok()
}

fn compiled_policy(task: Option<&Task>) -> Result<(Policy, Compiled), globset::Error> {
    let mut policy = Policy::new(task.map(|t| vec![t.test_command.clone()]).unwrap_or_default());
    if let Some(t) = task {
        policy.allowed.clone_from(&t.allowed_paths);
    }
    let compiled = policy.compile()?;
    Ok((policy, compiled))
}

/// A plan update carried by this tool call, if any.
fn plan_update(input: &Input) -> Option<(String, agenda::Update)> {
    let tool = input.tool.as_deref()?;
    if let Some(update) = agenda::from_tool(tool, &input.tool_input, input.tool_response.as_ref()) {
        return Some((tool.to_owned(), update));
    }
    if tool == "hc_plan" {
        let markdown = input.tool_input.get("markdown")?.as_str()?;
        let update = agenda::from_markdown(markdown);
        return matches!(&update, agenda::Update::Replace(items) if !items.is_empty()).then(|| ("mcp".to_owned(), update));
    }
    if tool == "ExitPlanMode" {
        let plan = input.tool_response.as_ref().and_then(|r| r.get("plan")).or_else(|| input.tool_input.get("plan"))?.as_str()?;
        let update = agenda::from_markdown(plan);
        return matches!(&update, agenda::Update::Replace(items) if !items.is_empty()).then(|| (tool.to_owned(), update));
    }
    for path in written_paths(tool, &input.tool_input, &input.cwd) {
        let full = if Path::new(&path).is_absolute() { PathBuf::from(&path) } else { input.cwd.join(&path) };
        let in_artifacts = input.artifact_dir.as_ref().is_some_and(|d| full.starts_with(d));
        let is_artifact = input.tool_input.get("IsArtifact").and_then(Value::as_bool).unwrap_or(false);
        let name = full.file_name().and_then(|n| n.to_str()).unwrap_or_default().to_owned();
        let text = std::fs::read_to_string(&full).ok()?;
        let update = if name.eq_ignore_ascii_case("PLANS.md") {
            agenda::from_exec_plan(&text)
        } else if (in_artifacts || is_artifact) && Path::new(&name).extension().is_some_and(|e| e.eq_ignore_ascii_case("md")) {
            agenda::from_markdown(&text)
        } else {
            continue;
        };
        if matches!(&update, agenda::Update::Replace(items) if !items.is_empty()) {
            return Some((format!("file:{path}"), update));
        }
    }
    None
}

struct Gate<'a, 'e> {
    env: &'a Env<'e>,
    session: String,
}

impl Gate<'_, '_> {
    fn records(&self) -> Result<Vec<Record>, BailiffError> {
        Ok(self.env.docket.records()?.into_iter().filter(|r| r.body.session() == self.session).collect())
    }

    /// Protected writes and cheat signatures among `writes` (docket records of `file_write`).
    fn hard(&self, writes: &[&Record], compiled: &Compiled) -> Result<Vec<String>, BailiffError> {
        let git = Git::bare(self.env.docket.git_dir());
        let mut failures = Vec::new();
        for r in writes {
            let Body::Obs(Obs { event: Event::FileWrite { path, after, added_lines, .. }, .. }) = &r.body else { continue };
            if compiled.is_protected(path) {
                failures.push(format!("{path} was modified; it is protected (tests, CI, hook or agent configuration)."));
            }
            if let (Some(blob), true) = (after, signatures::is_python(path)) {
                let source = git.run(&["cat-file", "blob", blob])?;
                for hit in signatures::scan_python(&source, added_lines, compiled.is_test(path)) {
                    failures.push(format!(
                        "{path}:{} adds a `{}` pattern, a known way to make checks pass without doing the work.",
                        hit.line, hit.rule
                    ));
                }
            }
        }
        Ok(failures)
    }
}

fn file_writes(records: &[Record]) -> Vec<&Record> {
    records.iter().filter(|r| matches!(&r.body, Body::Obs(Obs { event: Event::FileWrite { .. }, .. }))).collect()
}

fn message(items: &[String]) -> String {
    let mut text = HEADER.to_owned();
    for i in items {
        text.push_str("\n- ");
        text.push_str(i);
    }
    text
}

fn state_path(env: &Env<'_>, dialect: Dialect, session: &str) -> PathBuf {
    let slug: String = session.chars().map(|c| if c.is_ascii_alphanumeric() || c == '-' { c } else { '_' }).collect();
    env.state_dir.join(format!("{dialect:?}-{slug}.json").to_lowercase())
}

/// Handle one hook invocation and return what the harness should do.
pub fn handle(env: &Env<'_>, dialect: Dialect, input: &Input) -> Result<Verdict, BailiffError> {
    let task = load_task(&input.cwd);
    let (_policy, compiled) = compiled_policy(task.as_ref())?;

    if input.kind == EventKind::PreToolUse {
        let tool = input.tool.as_deref().unwrap_or_default();
        let blocked: Vec<String> =
            written_paths(tool, &input.tool_input, &input.cwd).into_iter().filter(|p| compiled.is_protected(p)).collect();
        return Ok(if blocked.is_empty() {
            Verdict::Allow
        } else {
            Verdict::Deny(format!(
                "{HEADER}\n- {} is protected (tests, CI, hook or agent configuration). Writing it would refute any completion claim. If the task really requires changing it, report a spec_conflict instead.",
                blocked.join(", ")
            ))
        });
    }

    let session = format!("{dialect:?}-{}", input.session).to_lowercase();
    let path = state_path(env, dialect, &input.session);
    let mut state: State = std::fs::read_to_string(&path).ok().and_then(|t| serde_json::from_str(&t).ok()).unwrap_or_default();
    let force = Policy::new(vec![]).protected;
    let recorder = Recorder { docket: env.docket, harness: env.harness, force_paths: &force };
    let ts = (env.clock)();
    if state.open.is_none() {
        let attestation = roster::Registry::builtin().detect().unwrap_or_else(|| roster::declared("unregistered"));
        let task_id = task.as_ref().map_or_else(|| "adhoc".to_owned(), |t| t.id.clone());
        state.open = Some(recorder.open(&ts, &session, &task_id, &input.cwd, attestation, env.sandbox.isolation)?);
    }
    let gate = Gate { env, session: session.clone() };

    let verdict = match input.kind {
        EventKind::PreToolUse => unreachable!("handled above"),
        EventKind::SessionStart => {
            let brief = std::fs::read_to_string(input.cwd.join(".handoff/brief.md")).ok();
            brief.map_or(Verdict::Allow, Verdict::Observe)
        }
        EventKind::PostInvocation => {
            if state.queued.is_empty() {
                Verdict::Allow
            } else {
                let items = std::mem::take(&mut state.queued);
                state.delivered.extend(items.iter().cloned());
                Verdict::Observe(message(&items))
            }
        }
        EventKind::PostToolUse => post_tool_use(&gate, &recorder, &mut state, input, &compiled, &ts, dialect)?,
        EventKind::TaskCompleted => task_completed(&gate, &recorder, &mut state, input, &compiled, &ts, task.as_ref())?,
        EventKind::Stop => stop(&gate, &recorder, &mut state, input, &compiled, &ts, task.as_ref())?,
    };
    std::fs::create_dir_all(&env.state_dir)?;
    std::fs::write(&path, serde_json::to_string(&state)?)?;
    Ok(verdict)
}

/// The hook session currently open on `repo`, most recently active first, so a
/// bare `hc claim` binds to the session whose writes were recorded.
pub fn open_session_for(env: &Env<'_>, repo: &Path) -> Option<String> {
    let repo = repo.canonicalize().ok()?;
    let mut found: Vec<(std::time::SystemTime, String)> = std::fs::read_dir(&env.state_dir)
        .ok()?
        .flatten()
        .filter_map(|e| {
            let modified = e.metadata().ok()?.modified().ok()?;
            let state: State = serde_json::from_str(&std::fs::read_to_string(e.path()).ok()?).ok()?;
            let open = state.open?;
            (open.worktree.canonicalize().ok()? == repo).then_some((modified, open.session))
        })
        .collect();
    found.sort();
    found.pop().map(|(_, s)| s)
}

/// For `hc claim`: if `session` (as recorded, e.g. `claude-abc`) is a hook
/// session on this machine, snapshot it first. Returns whether it was one.
pub fn checkpoint_recorded(env: &Env<'_>, session: &str, cwd: &Path) -> Result<bool, BailiffError> {
    for dialect in [Dialect::Claude, Dialect::Antigravity, Dialect::Mcp] {
        let prefix = format!("{dialect:?}-").to_lowercase();
        let Some(id) = session.strip_prefix(&prefix) else { continue };
        if !state_path(env, dialect, id).exists() {
            continue;
        }
        let input = Input {
            kind: EventKind::PostToolUse,
            session: id.to_owned(),
            cwd: cwd.to_path_buf(),
            tool: None,
            tool_input: serde_json::Value::Null,
            tool_response: None,
            task_id: None,
            continuing: false,
            artifact_dir: None,
        };
        checkpoint_now(env, dialect, &input)?;
        return Ok(true);
    }
    Ok(false)
}

/// Snapshot the session now (opening it if needed), so claims filed next are
/// judged against the tree as it is, not as it was at the last step boundary.
pub fn checkpoint_now(env: &Env<'_>, dialect: Dialect, input: &Input) -> Result<(), BailiffError> {
    let task = load_task(&input.cwd);
    let path = state_path(env, dialect, &input.session);
    let mut state: State = std::fs::read_to_string(&path).ok().and_then(|t| serde_json::from_str(&t).ok()).unwrap_or_default();
    let force = Policy::new(vec![]).protected;
    let recorder = Recorder { docket: env.docket, harness: env.harness, force_paths: &force };
    let ts = (env.clock)();
    if state.open.is_none() {
        let session = format!("{dialect:?}-{}", input.session).to_lowercase();
        let attestation = roster::Registry::builtin().detect().unwrap_or_else(|| roster::declared("unregistered"));
        let task_id = task.as_ref().map_or_else(|| "adhoc".to_owned(), |t| t.id.clone());
        state.open = Some(recorder.open(&ts, &session, &task_id, &input.cwd, attestation, env.sandbox.isolation)?);
    }
    checkpoint(&recorder, &mut state, &ts)?;
    std::fs::create_dir_all(&env.state_dir)?;
    std::fs::write(&path, serde_json::to_string(&state)?)?;
    Ok(())
}

fn checkpoint(recorder: &Recorder<'_>, state: &mut State, ts: &str) -> Result<(), BailiffError> {
    if let Some(open) = state.open.as_mut() {
        recorder.checkpoint(ts, open)?;
    }
    Ok(())
}

fn post_tool_use(
    gate: &Gate<'_, '_>,
    recorder: &Recorder<'_>,
    state: &mut State,
    input: &Input,
    compiled: &Compiled,
    ts: &str,
    dialect: Dialect,
) -> Result<Verdict, BailiffError> {
    let Some((source, update)) = plan_update(input) else {
        // Any file-writing tool moves the tree; snapshot it so claims filed later bind to the code as it is.
        if WRITE_TOOLS.contains(&input.tool.as_deref().unwrap_or_default()) {
            checkpoint(recorder, state, ts)?;
        }
        return Ok(Verdict::Allow);
    };
    let before_seq = gate.env.docket.count()?;
    checkpoint(recorder, state, ts)?;
    let mut tracker = agenda::Tracker::restore(std::mem::take(&mut state.plan));
    let before = tracker.steps().to_vec();
    let after = tracker.apply(update).to_vec();
    state.plan.clone_from(&after);
    if after == before {
        return Ok(Verdict::Allow);
    }
    recorder.plan(ts, &gate.session, &source, after.clone())?;
    let completed: Vec<String> =
        agenda::transitions(&before, &after).into_iter().filter(|(_, _, to)| *to == StepStatus::Completed).map(|(k, _, _)| k).collect();
    if completed.is_empty() {
        return Ok(Verdict::Allow);
    }
    let records = gate.records()?;
    let fresh: Vec<&Record> = file_writes(&records).into_iter().filter(|r| r.seq >= before_seq).collect();
    let hard = gate.hard(&fresh, compiled)?;
    let report = conform::align(&records, &gate.session);
    let soft: Vec<String> = report
        .plan_only()
        .filter(|s| completed.contains(&s.key))
        .map(|s| format!("Step \"{}\" is marked completed, but no file changes were recorded while it was in progress.", s.text))
        .filter(|o| !state.delivered.contains(o))
        .collect();
    let verdict = match (hard.is_empty(), soft.is_empty()) {
        (false, _) => Verdict::Block(message(&[hard, soft.clone()].concat())),
        (true, false) => Verdict::Observe(message(&soft)),
        (true, true) => Verdict::Allow,
    };
    state.delivered.extend(soft);
    if dialect == Dialect::Antigravity
        && let Verdict::Block(t) | Verdict::Observe(t) = &verdict
    {
        state.queued.push(t.trim_start_matches(HEADER).trim().to_owned());
    }
    Ok(verdict)
}

fn task_completed(
    gate: &Gate<'_, '_>,
    recorder: &Recorder<'_>,
    state: &mut State,
    input: &Input,
    compiled: &Compiled,
    ts: &str,
    task: Option<&Task>,
) -> Result<Verdict, BailiffError> {
    let before_seq = gate.env.docket.count()?;
    checkpoint(recorder, state, ts)?;
    let records = gate.records()?;
    let fresh: Vec<&Record> = file_writes(&records).into_iter().filter(|r| r.seq >= before_seq).collect();
    let mut hard = gate.hard(&fresh, compiled)?;
    if task.is_some_and(|t| t.step_tests) {
        hard.extend(acceptance(gate, state, task)?);
    }
    let key = input.task_id.as_ref().map(|id| format!("native:{id}"));
    if let Some(key) = key {
        let mut hypothetical = records.clone();
        let mut plan = state.plan.clone();
        for s in plan.iter_mut().filter(|s| s.key == key) {
            s.status = StepStatus::Completed;
        }
        hypothetical.push(Record {
            seq: u64::MAX,
            commit: String::new(),
            signer: String::new(),
            body: Body::Obs(Obs {
                session: gate.session.clone(),
                ts: ts.to_owned(),
                event: Event::PlanUpdate { source: "hypothetical".into(), steps: plan },
            }),
        });
        for s in conform::align(&hypothetical, &gate.session).plan_only().filter(|s| s.key == key) {
            state.queued.push(format!("Step \"{}\" was completed with no file changes recorded while it was in progress.", s.text));
        }
    }
    Ok(if hard.is_empty() { Verdict::Allow } else { Verdict::Block(message(&hard)) })
}

fn acceptance(gate: &Gate<'_, '_>, state: &State, task: Option<&Task>) -> Result<Vec<String>, BailiffError> {
    let (Some(task), Some(open)) = (task, state.open.as_ref()) else { return Ok(Vec::new()) };
    let Some(argv) = shlex_split(&task.test_command) else { return Ok(Vec::new()) };
    let reexec = Reexec { sandbox: gate.env.sandbox.clone(), runs: 1 };
    let run = reexec.run(Source::Tree { docket_git: &gate.env.docket.git_dir(), tree: &open.tree }, &argv)?;
    Ok(run
        .exit_codes
        .iter()
        .find(|c| **c != 0)
        .map(|code| vec![format!("The acceptance command `{}` exits {code} on a clean copy of the current tree.", task.test_command)])
        .unwrap_or_default())
}

fn shlex_split(command: &str) -> Option<Vec<String>> {
    let argv: Vec<String> = command.split_whitespace().map(str::to_owned).collect();
    (!argv.is_empty()).then_some(argv)
}

fn stop(
    gate: &Gate<'_, '_>,
    recorder: &Recorder<'_>,
    state: &mut State,
    input: &Input,
    compiled: &Compiled,
    ts: &str,
    task: Option<&Task>,
) -> Result<Verdict, BailiffError> {
    checkpoint(recorder, state, ts)?;
    let records = gate.records()?;
    let mut failures = gate.hard(&file_writes(&records), compiled)?;
    failures.extend(acceptance(gate, state, task)?);
    let fingerprint = (!failures.is_empty()).then(|| conform::fingerprint(&failures));
    if state.escalated {
        if fingerprint.is_some() && fingerprint == state.last_failure {
            return Ok(Verdict::Allow);
        }
        state.escalated = false;
    }
    let report = conform::align(&records, &gate.session);
    let mut notes: Vec<String> = std::mem::take(&mut state.queued);
    notes.extend(report.observations());
    notes.retain(|n| !state.delivered.contains(n));
    notes.dedup();

    if let Some(fp) = fingerprint {
        if state.last_failure.as_deref() == Some(fp.as_str()) {
            state.escalated = true;
            let mut items = failures.clone();
            items.push(
                "This is the same failure as the previous attempt. Stop attempting fixes. Report the root cause instead: \
                 state what blocks the task and why, as a `blocked` or `spec_conflict` claim (`hc claim`) or in your final message."
                    .to_owned(),
            );
            return Ok(Verdict::Block(message(&items)));
        }
        state.last_failure = Some(fp);
        state.delivered.extend(notes.iter().cloned());
        return Ok(Verdict::Block(message(&[failures, notes].concat())));
    }
    state.last_failure = None;
    if notes.is_empty() || input.continuing {
        return Ok(Verdict::Allow);
    }
    state.delivered.extend(notes.iter().cloned());
    Ok(Verdict::Observe(message(&notes)))
}

#[cfg(test)]
mod tests;
