//! courier: delivers briefs to where agents and people read them.
//!
//! - `prompt`: the brief plus the reporting contract, for `hc run` launches.
//! - `protocol_block`: a marked section for an agent's instructions file
//!   (AGENTS.md, CLAUDE.md, GEMINI.md), replaced in place on every write.
//! - `notion`: a page per brief through the Notion API `2025-09-03`, created
//!   under a data source with Notion-flavored Markdown (`markdown_version: v2`).
//!
//! Every copy is a view. Trust is decided only from the docket.

use std::path::Path;

use brief::Brief;
use serde_json::{Value, json};

pub const BEGIN: &str = "<!-- hc:begin -->";
pub const END: &str = "<!-- hc:end -->";
pub const NOTION_VERSION: &str = "2025-09-03";

pub fn prompt(brief: &Brief, schema_enforced: bool) -> String {
    let mut out = brief.render();
    out.push_str("\n## Reporting\n\nYour final message is your testimony. ");
    if schema_enforced {
        out.push_str("It is constrained to the claim schema by your CLI. ");
    } else {
        out.push_str("Make it a single JSON object matching this schema and nothing else:\n\n```json\n");
        out.push_str(intake::CLAIM_SCHEMA.trim());
        out.push_str("\n```\n\n");
    }
    out.push_str(
        "Write `basis` before deciding `status`. Claim only what you did; every claim is checked against what the harness observed.\n",
    );
    out
}

pub fn protocol_block(test_command: &str) -> String {
    format!(
        "{BEGIN}\n## Handoff protocol (hc)\n\n\
         - Start from the current brief: run `hc brief`. Facts there were verified; unverified items are not.\n\
         - Finish by reporting claims: write JSON matching `hc schema` and pipe it to `hc claim`.\n\
         - The verifier re-runs `{test_command}` outside your session. Editing tests, CI, hooks or agent configuration refutes your claims.\n\
         - `spec_conflict` and `no_change_needed` are successful outcomes. Use them instead of forcing a pass.\n\
         {END}\n"
    )
}

/// Insert or replace the hc block in an instructions file, leaving the rest untouched.
pub fn write_protocol_block(file: &Path, test_command: &str) -> std::io::Result<bool> {
    let block = protocol_block(test_command);
    let current = std::fs::read_to_string(file).unwrap_or_default();
    let next = match (current.find(BEGIN), current.find(END)) {
        (Some(start), Some(end)) if end > start => {
            format!("{}{block}{}", &current[..start], current[end + END.len()..].trim_start_matches('\n'))
        }
        _ if current.is_empty() => block,
        _ => format!("{}\n\n{block}", current.trim_end()),
    };
    if next == current {
        return Ok(false);
    }
    std::fs::write(file, next)?;
    Ok(true)
}

pub mod notion {
    use super::{Brief, NOTION_VERSION, Value, json};

    #[derive(Debug, thiserror::Error)]
    pub enum NotionError {
        #[error("http: {0}")]
        Http(#[from] ureq::Error),
    }

    /// Request body for `POST /v1/pages` creating one page per brief.
    pub fn page_body(data_source_id: &str, title_property: &str, brief: &Brief) -> Value {
        let digest = brief.digest().to_string();
        let short = &digest["sha256:".len().."sha256:".len() + 12];
        json!({
            "parent": {"type": "data_source_id", "data_source_id": data_source_id},
            "properties": {
                title_property: {"title": [{"type": "text", "text": {"content": format!("{} brief {short}", brief.task.id)}}]}
            },
            "markdown": brief.render(),
            "markdown_version": "v2"
        })
    }

    pub fn create_page(token: &str, body: &Value) -> Result<Value, NotionError> {
        let mut response = ureq::post("https://api.notion.com/v1/pages")
            .header("Authorization", &format!("Bearer {token}"))
            .header("Notion-Version", NOTION_VERSION)
            .send_json(body)?;
        Ok(response.body_mut().read_json::<Value>()?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use brief::Task;

    fn sample() -> Brief {
        Brief::link(
            Task {
                id: "T-1".into(),
                objective: "o".into(),
                test_command: "make test".into(),
                allowed_paths: None,
                constraints: vec![],
                next_step: "go".into(),
                step_tests: false,
                tests_writable: false,
            },
            &[],
        )
        .unwrap()
    }

    #[test]
    fn protocol_block_is_idempotent_and_preserves_surroundings() {
        let dir = tempfile::tempdir().unwrap();
        let f = dir.path().join("AGENTS.md");
        std::fs::write(&f, "# Project\n\nRules.\n").unwrap();
        assert!(write_protocol_block(&f, "make test").unwrap());
        assert!(!write_protocol_block(&f, "make test").unwrap());
        assert!(write_protocol_block(&f, "cargo test").unwrap());
        let text = std::fs::read_to_string(&f).unwrap();
        assert!(text.starts_with("# Project\n\nRules.\n\n<!-- hc:begin -->"));
        assert_eq!(text.matches(BEGIN).count(), 1);
        assert!(text.contains("re-runs `cargo test`"));
    }

    #[test]
    fn notion_body_targets_a_data_source_with_v2_markdown() {
        let body = notion::page_body("ds_123", "Name", &sample());
        assert_eq!(body["parent"], json!({"type": "data_source_id", "data_source_id": "ds_123"}));
        assert_eq!(body["markdown_version"], "v2");
        assert!(body["properties"]["Name"]["title"][0]["text"]["content"].as_str().unwrap().starts_with("T-1 brief "));
        assert!(body["markdown"].as_str().unwrap().contains("## Contract"));
    }

    #[test]
    fn unenforced_prompts_embed_the_schema() {
        assert!(prompt(&sample(), false).contains("\"$defs\""));
        assert!(!prompt(&sample(), true).contains("\"$defs\""));
    }
}

/// Hook configuration for each harness, merged into existing files without
/// disturbing other hooks. Shapes follow each vendor's hook reference:
/// Claude Code `.claude/settings.json` and Codex `.codex/hooks.json` use
/// `{"hooks": {Event: [{"matcher", "hooks": [{"type": "command", "command"}]}]}}`;
/// Antigravity `.agents/hooks.json` uses `{Event: [...]}` where tool events take
/// matcher groups and other events take handlers directly.
pub mod hooks {
    use serde_json::{Map, Value, json};

    pub const MARKER: &str = "hc hook ";

    fn handler(bin: &str, dialect: &str, event: &str) -> Value {
        json!({"type": "command", "command": format!("{bin} hook {dialect} {event}")})
    }

    fn group(matcher: Option<&str>, h: &Value) -> Value {
        match matcher {
            Some(m) => json!({"matcher": m, "hooks": [h]}),
            None => json!({"hooks": [h]}),
        }
    }

    /// (event, matcher) pairs per dialect.
    pub fn plan(dialect: &str) -> Vec<(&'static str, Option<&'static str>)> {
        match dialect {
            "claude" => vec![
                ("SessionStart", None),
                ("PreToolUse", Some("Write|Edit|MultiEdit|NotebookEdit")),
                ("PostToolUse", Some("TodoWrite|TaskCreate|TaskUpdate|ExitPlanMode|Write|Edit|MultiEdit")),
                ("TaskCompleted", None),
                ("Stop", None),
            ],
            "codex" => vec![
                ("SessionStart", None),
                ("PreToolUse", Some("apply_patch")),
                ("PostToolUse", Some("update_plan|apply_patch")),
                ("Stop", None),
            ],
            "antigravity" => vec![
                ("SessionStart", None),
                ("PreToolUse", Some("write_to_file|replace_file_content|multi_replace_file_content")),
                ("PostToolUse", Some("write_to_file|replace_file_content|multi_replace_file_content")),
                ("PostInvocation", None),
                ("Stop", None),
            ],
            _ => Vec::new(),
        }
    }

    fn is_ours(v: &Value) -> bool {
        v.to_string().contains(MARKER)
    }

    /// Merge hc's hooks into `existing`, replacing any previous hc entries.
    pub fn merge(dialect: &str, existing: Value, bin: &str) -> Value {
        let mut root = match existing {
            Value::Object(m) => m,
            _ => Map::new(),
        };
        let wire = if dialect == "codex" { "claude" } else { dialect };
        let nested = dialect != "antigravity";
        let events = if nested {
            root.entry("hooks").or_insert_with(|| json!({})).as_object_mut().expect("hooks is an object").clone()
        } else {
            root.clone()
        };
        let mut events = events;
        for (event, matcher) in plan(dialect) {
            let list = events.entry(event).or_insert_with(|| json!([]));
            let mut kept: Vec<Value> = list.as_array().cloned().unwrap_or_default().into_iter().filter(|v| !is_ours(v)).collect();
            let h = handler(bin, wire, event);
            let entry = if !nested && matcher.is_none() { h } else { group(matcher, &h) };
            kept.push(entry);
            *list = Value::Array(kept);
        }
        if nested {
            root.insert("hooks".into(), Value::Object(events));
        } else {
            root = events;
        }
        Value::Object(root)
    }

    pub fn path(dialect: &str) -> Option<&'static str> {
        match dialect {
            "claude" => Some(".claude/settings.json"),
            "codex" => Some(".codex/hooks.json"),
            "antigravity" => Some(".agents/hooks.json"),
            _ => None,
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn merges_idempotently_and_keeps_foreign_hooks() {
            let mine = json!({"permissions": {}, "hooks": {"Stop": [{"hooks": [{"type": "command", "command": "notify-send done"}]}]}});
            let once = merge("claude", mine, "hc");
            let twice = merge("claude", once.clone(), "hc");
            assert_eq!(once, twice);
            assert_eq!(once["hooks"]["Stop"].as_array().unwrap().len(), 2);
            assert_eq!(once["hooks"]["TaskCompleted"][0]["hooks"][0]["command"], "hc hook claude TaskCompleted");
            assert_eq!(once["permissions"], json!({}));
        }

        #[test]
        fn codex_speaks_the_claude_dialect_and_antigravity_is_flat() {
            let c = merge("codex", json!({}), "hc");
            assert_eq!(c["hooks"]["PostToolUse"][0]["matcher"], "update_plan|apply_patch");
            assert_eq!(c["hooks"]["Stop"][0]["hooks"][0]["command"], "hc hook claude Stop");
            let a = merge("antigravity", json!({}), "hc");
            assert_eq!(a["PostInvocation"][0]["command"], "hc hook antigravity PostInvocation");
            assert_eq!(a["PreToolUse"][0]["matcher"], "write_to_file|replace_file_content|multi_replace_file_content");
        }
    }
}
