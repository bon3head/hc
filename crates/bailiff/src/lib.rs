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

use std::collections::BTreeMap;
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
    #[error("could not compile the brief")]
    Kernel(#[from] crossexam::KernelError),
    #[error("could not compile the brief")]
    Brief(#[from] brief::BriefError),
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
/// `p` (absolute, or relative to `base`) as a path relative to `root`, with `.` and `..`
/// resolved lexically, so `src/../tests/x.py` is matched as `tests/x.py`. A path that
/// leaves the root is returned absolute and never matches a repository pattern.
fn repo_relative(root: &Path, base: &Path, p: &str) -> String {
    let joined = if Path::new(p).is_absolute() { PathBuf::from(p) } else { base.join(p) };
    let mut parts: Vec<std::path::Component<'_>> = Vec::new();
    for c in joined.components() {
        match c {
            std::path::Component::CurDir => {}
            std::path::Component::ParentDir => {
                parts.pop();
            }
            other => parts.push(other),
        }
    }
    let normal: PathBuf = parts.iter().collect();
    normal.strip_prefix(root).map_or_else(|_| normal.to_string_lossy().into_owned(), |r| r.to_string_lossy().into_owned())
}

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
    let root = repo_root(&base);
    let mut out: Vec<String> = paths.into_iter().map(|p| repo_relative(&root, &base, &p)).collect();
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

/// The contract recorded by `hc init`; the repository's task file only when none was
/// recorded (a repository set up before baselines existed).
fn load_task(env: &Env<'_>, cwd: &Path) -> Option<Task> {
    if let Some(contract) = exhibit::recorded_contract(&env.docket.git_dir()) {
        return serde_json::from_value(contract).ok();
    }
    let text = std::fs::read_to_string(cwd.join(".handoff/task.json")).ok()?;
    serde_json::from_str(&text).ok()
}

fn compiled_policy(task: Option<&Task>) -> Result<(Policy, Compiled), globset::Error> {
    let mut policy = Policy::new(task.map(|t| vec![t.test_command.clone()]).unwrap_or_default());
    if let Some(t) = task {
        policy.allowed.clone_from(&t.allowed_paths);
        if t.tests_writable {
            policy = policy.with_writable_tests();
        }
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
    /// What the session must still undo before it may stop: protected paths whose content
    /// differs from the baseline now, and known cheat patterns in each file's latest
    /// content. A protected file restored to its accepted content no longer blocks.
    fn hard(&self, records: &[Record], since: u64, compiled: &Compiled) -> Result<Vec<String>, BailiffError> {
        let git = Git::bare(self.env.docket.git_dir());
        let tree_of = |want_baseline: bool| {
            records.iter().rev().find_map(|r| match &r.body {
                Body::Obs(Obs { event: Event::Baseline { tree, .. }, .. }) if want_baseline => Some(tree.clone()),
                Body::Obs(Obs { event: Event::SessionStart { tree, .. }, .. }) if !want_baseline => Some(tree.clone()),
                _ => None,
            })
        };
        let base = tree_of(true).or_else(|| tree_of(false));
        let now = self.env.docket.last_tree()?;
        let blob = |tree: Option<&String>, path: &str| -> Result<Option<String>, BailiffError> {
            Ok(match tree {
                Some(t) => git.try_run(&["rev-parse", "--verify", "-q", &format!("{t}:{path}")])?,
                None => None,
            })
        };
        // The latest write per path is the one that describes the file now.
        let mut latest: BTreeMap<&str, &Record> = BTreeMap::new();
        for r in file_writes(records).into_iter().filter(|r| r.seq >= since) {
            if let Body::Obs(Obs { event: Event::FileWrite { path, .. }, .. }) = &r.body {
                latest.insert(path.as_str(), r);
            }
        }
        let mut failures = Vec::new();
        for r in latest.into_values() {
            let Body::Obs(Obs { event: Event::FileWrite { path, after, added_lines, .. }, .. }) = &r.body else { continue };
            let untracked = path.starts_with(".git/");
            if !untracked && now.is_some() && blob(base.as_ref(), path)? == blob(now.as_ref(), path)? {
                continue;
            }
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

/// Writes recorded since the latest baseline: earlier ones were accepted by the owner.
fn file_writes(records: &[Record]) -> Vec<&Record> {
    let since = records.iter().rev().find(|r| matches!(&r.body, Body::Obs(Obs { event: Event::Baseline { .. }, .. }))).map_or(0, |r| r.seq);
    records.iter().filter(|r| r.seq > since && matches!(&r.body, Body::Obs(Obs { event: Event::FileWrite { .. }, .. }))).collect()
}

fn message(items: &[String]) -> String {
    let mut text = HEADER.to_owned();
    for i in items {
        text.push_str("\n- ");
        text.push_str(i);
    }
    text
}

/// The brief for `task`, compiled from the ledger exactly as `hc brief` does (read-only:
/// verdicts are not recorded from a hook). Nothing from the working tree is injected, so
/// an agent cannot plant text for the next session. `None` before any claim exists.
fn compiled_brief(env: &Env<'_>, task: &Task, cwd: &Path) -> Result<Option<String>, BailiffError> {
    let records = env.docket.records()?;
    if !records.iter().any(|r| matches!(&r.body, Body::Claim(c) if c.task == task.id)) {
        return Ok(None);
    }
    let kernel = crossexam::Kernel::for_task(
        &task.test_command,
        task.allowed_paths.clone(),
        task.tests_writable,
        env.docket.git_dir(),
        cwd.to_path_buf(),
        env.sandbox.clone(),
    );
    let mut findings: Vec<_> = kernel.examine(&records)?.into_iter().filter(|f| f.claim.task == task.id).collect();
    kernel.observe_now(&mut findings)?;
    let mut brief = brief::Brief::link(task.clone(), &findings)?;
    brief.changes = kernel.drift(&records, &findings)?;
    brief.observations = brief::observations(records.iter().map(|r| (r.seq, &r.body)));
    Ok(Some(brief.render()))
}

/// A shell command or file read that looks at repository history beyond the current
/// branch: `git log --all` (or `--branches`, `--remotes`, `--reflog`, `-g`), `git
/// reflog`, `git show <sha>`, `git cat-file`, `git branch -a`, `git for-each-ref`,
/// `git fsck`, or a path under `.git/objects`, `.git/refs` or `.git/logs`. Returns the
/// command (or path) to record. Recorded as an observation only; it refutes nothing.
pub fn history_read(tool: &str, input: &Value) -> Option<String> {
    const GIT_PATHS: [&str; 4] = [".git/objects", ".git/refs", ".git/logs", ".git/packed-refs"];
    let touches_git_dir = |s: &str| GIT_PATHS.iter().any(|g| s.contains(g));
    if tool != "Bash" {
        let path = ["file_path", "path", "pattern"].iter().find_map(|k| input.get(*k).and_then(Value::as_str))?;
        return touches_git_dir(path).then(|| path.to_owned());
    }
    let command = input.get("command").and_then(Value::as_str)?;
    if touches_git_dir(command) {
        return Some(command.to_owned());
    }
    let hit = command.split(['\n', ';', '|', '&', '(', ')']).any(|segment| {
        let words: Vec<&str> = segment.split_whitespace().collect();
        let Some(at) = words.iter().position(|w| *w == "git" || w.ends_with("/git")) else { return false };
        let mut rest = words[at + 1..].iter().copied();
        let sub = loop {
            match rest.next() {
                Some("-C" | "-c" | "--git-dir" | "--work-tree") => {
                    rest.next();
                }
                Some(w) if w.starts_with('-') => {}
                other => break other,
            }
        };
        let args: Vec<&str> = rest.collect();
        let any = |flags: &[&str]| args.iter().any(|a| flags.contains(a));
        let is_sha = |a: &&str| (7..=40).contains(&a.len()) && a.chars().all(|c| c.is_ascii_hexdigit());
        match sub {
            Some("reflog" | "cat-file" | "fsck" | "for-each-ref" | "show-ref") => true,
            Some("log" | "rev-list" | "shortlog" | "whatchanged") => {
                any(&["--all", "--branches", "--remotes", "--reflog", "-g", "--walk-reflogs"])
            }
            Some("show") => args.iter().any(is_sha),
            Some("branch") => any(&["-a", "--all", "-r", "--remotes"]),
            _ => false,
        }
    });
    hit.then(|| command.to_owned())
}

/// The repository root: the nearest ancestor holding `.git`, so a hook fired from a
/// subdirectory still finds the task, the ledger and the right protected paths.
pub fn repo_root(dir: &Path) -> PathBuf {
    let dir = dir.canonicalize().unwrap_or_else(|_| dir.to_path_buf());
    dir.ancestors().find(|d| d.join(".git").exists()).unwrap_or(&dir).to_path_buf()
}

/// The recorded session name: dialect prefix in lower case, the harness's id as given,
/// so claims filed under the same id (MCP's `mcp-<id>`, `hc claim --session`) match.
pub fn session_name(dialect: Dialect, id: &str) -> String {
    format!("{}-{id}", format!("{dialect:?}").to_lowercase())
}

fn state_path(env: &Env<'_>, dialect: Dialect, session: &str) -> PathBuf {
    let slug: String = session.chars().map(|c| if c.is_ascii_alphanumeric() || c == '-' { c } else { '_' }).collect();
    // The id keeps its case: `Work-1` and `work-1` are different sessions, as they are in the ledger.
    env.state_dir.join(format!("{}-{slug}.json", format!("{dialect:?}").to_lowercase()))
}

/// Handle one hook invocation and return what the harness should do.
pub fn handle(env: &Env<'_>, dialect: Dialect, input: &Input) -> Result<Verdict, BailiffError> {
    let task = load_task(env, &input.cwd);
    let (_policy, compiled) = compiled_policy(task.as_ref())?;
    let capture = task.as_ref().is_some_and(|t| t.capture);

    if input.kind == EventKind::PreToolUse {
        let tool = input.tool.as_deref().unwrap_or_default();
        if let Some(read) = history_read(tool, &input.tool_input) {
            let force = Policy::new(vec![]).protected;
            let recorder = Recorder { docket: env.docket, harness: env.harness, force_paths: &force };
            recorder.history_read(&(env.clock)(), &session_name(dialect, &input.session), &read)?;
        }
        if capture || !WRITE_TOOLS.contains(&tool) {
            return Ok(Verdict::Allow);
        }
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

    let session = session_name(dialect, &input.session);
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
        EventKind::SessionStart => match task.as_ref() {
            Some(t) => compiled_brief(env, t, &input.cwd)?.map_or(Verdict::Allow, Verdict::Observe),
            None => Verdict::Allow,
        },
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
    // Capture only: everything above has been recorded; nothing reaches the agent.
    Ok(if capture { Verdict::Allow } else { verdict })
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
    let task = load_task(env, &input.cwd);
    let path = state_path(env, dialect, &input.session);
    let mut state: State = std::fs::read_to_string(&path).ok().and_then(|t| serde_json::from_str(&t).ok()).unwrap_or_default();
    let force = Policy::new(vec![]).protected;
    let recorder = Recorder { docket: env.docket, harness: env.harness, force_paths: &force };
    let ts = (env.clock)();
    if state.open.is_none() {
        let session = session_name(dialect, &input.session);
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
    let hard = gate.hard(&records, before_seq, compiled)?;
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
    let mut hard = gate.hard(&records, before_seq, compiled)?;
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

/// Whether `records` (one session) hold a `spec_conflict` or `blocked` claim for `task`
/// filed after the session's last recorded write, so it describes the tree as it is.
/// `hc claim` checkpoints before recording, so writes made before the claim precede it.
fn reported_outcome(records: &[Record], task: &str) -> bool {
    let last_write =
        records.iter().rev().find(|r| matches!(&r.body, Body::Obs(Obs { event: Event::FileWrite { .. }, .. }))).map_or(0, |r| r.seq);
    records.iter().any(|r| {
        r.seq > last_write
            && matches!(&r.body, Body::Claim(c) if c.task == task
                && matches!(c.claim.status, bedrock::ir::Status::SpecConflict | bedrock::ir::Status::Blocked))
    })
}

/// Split as the kernel does (P4 and P8 use `shlex`), so the Stop gate runs the same argv.
fn shlex_split(command: &str) -> Option<Vec<String>> {
    shlex::split(command).filter(|argv| !argv.is_empty())
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
    let tampering = gate.hard(&records, 0, compiled)?;
    if !tampering.is_empty() {
        // Never released by the fail-fast escape below: repeating "tests were edited" is not
        // a fix loop, and letting the third Stop through would end the session on tampered code.
        return Ok(Verdict::Block(message(&tampering)));
    }
    let mut failures = acceptance(gate, state, task)?;
    if !failures.is_empty() && task.is_some_and(|t| reported_outcome(&records, &t.id)) {
        // The session already reported why the suite cannot pass (a valid outcome per the
        // contract). Blocking again would push it toward forcing a pass; the claim stays
        // unverified in the brief for the owner to settle.
        failures.clear();
    }
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
