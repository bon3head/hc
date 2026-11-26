//! Native hook dialects. Field names are taken from each vendor's hook
//! reference (see `docs/research-native-plans.md`):
//!
//! - Claude Code and Codex share one dialect: snake_case stdin
//!   (`session_id`, `cwd`, `tool_name`, `tool_input`, `tool_response`,
//!   `stop_hook_active`, `task_id`, `task_subject`), `hookSpecificOutput`
//!   with `additionalContext` / `permissionDecision`, top-level
//!   `decision: "block"` with `reason`, and exit code 2 with stderr.
//! - Antigravity uses camelCase stdin (`conversationId`, `workspacePaths`,
//!   `toolCall {name, args}`, `artifactDirectoryPath`), `decision`
//!   (`deny` / `continue`) with `reason`, and `injectSteps` on
//!   `SessionStart` and `PostInvocation`. Its `PostToolUse` cannot inject.

use std::path::PathBuf;

use serde_json::{Value, json};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Dialect {
    /// Claude Code and Codex.
    Claude,
    Antigravity,
    /// Tool calls through `clerk`, for surfaces without hooks. Verdicts come back as text.
    Mcp,
}

impl Dialect {
    pub fn parse(name: &str) -> Option<Self> {
        match name {
            "claude" | "claude-code" | "codex" => Some(Self::Claude),
            "antigravity" | "agy" => Some(Self::Antigravity),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventKind {
    SessionStart,
    PreToolUse,
    PostToolUse,
    TaskCompleted,
    Stop,
    PostInvocation,
}

impl EventKind {
    pub fn parse(name: &str) -> Option<Self> {
        Some(match name {
            "SessionStart" => Self::SessionStart,
            "PreToolUse" => Self::PreToolUse,
            "PostToolUse" => Self::PostToolUse,
            "TaskCompleted" => Self::TaskCompleted,
            "Stop" | "SubagentStop" => Self::Stop,
            "PostInvocation" => Self::PostInvocation,
            _ => return None,
        })
    }
}

/// One hook invocation, normalized across dialects.
#[derive(Debug, Clone)]
pub struct Input {
    pub kind: EventKind,
    pub session: String,
    pub cwd: PathBuf,
    pub tool: Option<String>,
    pub tool_input: Value,
    pub tool_response: Option<Value>,
    pub task_id: Option<String>,
    pub continuing: bool,
    pub artifact_dir: Option<PathBuf>,
}

fn s(v: &Value, key: &str) -> Option<String> {
    v.get(key).and_then(Value::as_str).map(str::to_owned)
}

pub fn read(dialect: Dialect, kind: EventKind, raw: &Value) -> Input {
    match dialect {
        Dialect::Claude | Dialect::Mcp => Input {
            kind,
            session: s(raw, "session_id").unwrap_or_else(|| "unknown".into()),
            cwd: s(raw, "cwd").map_or_else(|| PathBuf::from("."), PathBuf::from),
            tool: s(raw, "tool_name"),
            tool_input: raw.get("tool_input").cloned().unwrap_or(Value::Null),
            tool_response: raw.get("tool_response").cloned(),
            task_id: s(raw, "task_id"),
            continuing: raw.get("stop_hook_active").and_then(Value::as_bool).unwrap_or(false),
            artifact_dir: None,
        },
        Dialect::Antigravity => {
            let call = raw.get("toolCall").cloned().unwrap_or(Value::Null);
            Input {
                kind,
                session: s(raw, "conversationId").unwrap_or_else(|| "unknown".into()),
                cwd: raw.pointer("/workspacePaths/0").and_then(Value::as_str).map_or_else(|| PathBuf::from("."), PathBuf::from),
                tool: s(&call, "name"),
                tool_input: call.get("args").cloned().unwrap_or(Value::Null),
                tool_response: None,
                task_id: None,
                continuing: false,
                artifact_dir: s(raw, "artifactDirectoryPath").map(PathBuf::from),
            }
        }
    }
}

/// What the verifier wants the harness to do.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Verdict {
    /// Proceed; say nothing.
    Allow,
    /// Refuse a tool call before it runs.
    Deny(String),
    /// Proceed, and put this observation in front of the model.
    Observe(String),
    /// Refuse to let the step complete or the session stop, with the reason.
    Block(String),
}

/// Native rendering: (stdout, stderr, exit code).
pub fn render(dialect: Dialect, kind: EventKind, verdict: &Verdict) -> (String, String, i32) {
    let event = match kind {
        EventKind::SessionStart => "SessionStart",
        EventKind::PreToolUse => "PreToolUse",
        EventKind::PostToolUse => "PostToolUse",
        EventKind::TaskCompleted => "TaskCompleted",
        EventKind::Stop => "Stop",
        EventKind::PostInvocation => "PostInvocation",
    };
    let out = |v: Value| (v.to_string(), String::new(), 0);
    match (dialect, kind, verdict) {
        (Dialect::Mcp, _, v) => match v {
            Verdict::Allow => ("ok: no findings".to_owned(), String::new(), 0),
            Verdict::Deny(t) | Verdict::Observe(t) | Verdict::Block(t) => (t.clone(), String::new(), 0),
        },
        (_, _, Verdict::Allow) => match dialect {
            Dialect::Antigravity => out(json!({})),
            Dialect::Claude | Dialect::Mcp => (String::new(), String::new(), 0),
        },
        (Dialect::Claude, EventKind::TaskCompleted, Verdict::Block(r) | Verdict::Deny(r)) => (String::new(), r.clone(), 2),
        (Dialect::Claude, _, Verdict::Deny(r)) => {
            out(json!({"hookSpecificOutput": {"hookEventName": event, "permissionDecision": "deny", "permissionDecisionReason": r}}))
        }
        (Dialect::Claude, _, Verdict::Block(r)) => out(json!({"decision": "block", "reason": r})),
        (Dialect::Claude, EventKind::TaskCompleted, Verdict::Observe(_)) => (String::new(), String::new(), 0),
        (Dialect::Claude, _, Verdict::Observe(t)) => out(json!({"hookSpecificOutput": {"hookEventName": event, "additionalContext": t}})),
        (Dialect::Antigravity, EventKind::PreToolUse, Verdict::Deny(r) | Verdict::Block(r)) => {
            out(json!({"decision": "deny", "reason": r}))
        }
        (Dialect::Antigravity, EventKind::Stop, Verdict::Block(r) | Verdict::Observe(r)) => {
            out(json!({"decision": "continue", "reason": r}))
        }
        (Dialect::Antigravity, EventKind::SessionStart | EventKind::PostInvocation, Verdict::Observe(t) | Verdict::Block(t)) => {
            out(json!({"injectSteps": [{"systemMessage": {"systemMessage": t}}]}))
        }
        (Dialect::Antigravity, _, _) => out(json!({})),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn claude_and_codex_renderings() {
        let (o, _, c) = render(Dialect::Claude, EventKind::PreToolUse, &Verdict::Deny("no".into()));
        assert_eq!((serde_json::from_str::<Value>(&o).unwrap()["hookSpecificOutput"]["permissionDecision"].clone(), c), (json!("deny"), 0));
        let (o, _, _) = render(Dialect::Claude, EventKind::PostToolUse, &Verdict::Observe("seen".into()));
        assert_eq!(serde_json::from_str::<Value>(&o).unwrap()["hookSpecificOutput"]["additionalContext"], "seen");
        assert_eq!(render(Dialect::Claude, EventKind::TaskCompleted, &Verdict::Block("tests".into())), (String::new(), "tests".into(), 2));
        let (o, _, _) = render(Dialect::Claude, EventKind::Stop, &Verdict::Block("again".into()));
        assert_eq!(serde_json::from_str::<Value>(&o).unwrap(), json!({"decision": "block", "reason": "again"}));
    }

    #[test]
    fn antigravity_renderings() {
        let r = |k, v| serde_json::from_str::<Value>(&render(Dialect::Antigravity, k, &v).0).unwrap();
        assert_eq!(r(EventKind::PreToolUse, Verdict::Deny("x".into())), json!({"decision": "deny", "reason": "x"}));
        assert_eq!(r(EventKind::Stop, Verdict::Block("y".into())), json!({"decision": "continue", "reason": "y"}));
        assert_eq!(r(EventKind::PostInvocation, Verdict::Observe("z".into()))["injectSteps"][0]["systemMessage"]["systemMessage"], "z");
        assert_eq!(r(EventKind::PostToolUse, Verdict::Observe("lost".into())), json!({}));
    }
}
