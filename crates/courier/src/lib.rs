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
    out.push_str("Write `basis` before deciding `status`. Claim only what you did; every claim is checked against what the harness observed.\n");
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
        (Some(start), Some(end)) if end > start => format!("{}{block}{}", &current[..start], current[end + END.len()..].trim_start_matches('\n')),
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
            Task { id: "T-1".into(), objective: "o".into(), test_command: "make test".into(), allowed_paths: None, constraints: vec![], next_step: "go".into() },
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
