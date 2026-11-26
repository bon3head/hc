//! intake: the compiler front end for agent output.
//!
//! P1 `normalize` recovers the JSON object from a final message (markdown
//! fences and surrounding prose are the most common failure mode of
//! prompt-only structured output). P2 `typecheck` validates against
//! `schema/claim.v0.json`, decodes into the typed IR with unknown fields
//! rejected, and applies the cross-field rules the schema cannot express.

use std::collections::BTreeSet;
use std::sync::LazyLock;

use serde_json::Value;

use bedrock::canon::MAX_SAFE_INTEGER;
use bedrock::ir::{ClaimEnvelope, ClaimKind, Diagnostic};

pub const CLAIM_SCHEMA: &str = include_str!("../../../schema/claim.v0.json");

static VALIDATOR: LazyLock<jsonschema::Validator> = LazyLock::new(|| {
    let schema: Value = serde_json::from_str(CLAIM_SCHEMA).expect("claim schema is valid JSON");
    jsonschema::validator_for(&schema).expect("claim schema compiles")
});

pub fn normalize(raw: &str) -> Result<Value, Diagnostic> {
    let text = raw.trim();
    let (start, end) = match (text.find('{'), text.rfind('}')) {
        (Some(s), Some(e)) if e > s => (s, e),
        _ => return Err(Diagnostic::new("no-json", "no JSON object in the final message", "")),
    };
    let value: Value = serde_json::from_str(&text[start..=end]).map_err(|e| Diagnostic::new("invalid-json", e.to_string(), ""))?;
    if let Some(diag) = number_problem(&value, String::new()) {
        return Err(diag);
    }
    Ok(value)
}

fn number_problem(value: &Value, path: String) -> Option<Diagnostic> {
    match value {
        Value::Number(n) if n.is_f64() => Some(Diagnostic::new("float", "numbers in claims must be integers", path)),
        Value::Number(n) if n.as_i64().is_none_or(|i| i.abs() > MAX_SAFE_INTEGER) => {
            Some(Diagnostic::new("unsafe-integer", "integers must be within ±(2^53-1) (RFC 7493 §2.2)", path))
        }
        Value::Array(items) => items.iter().enumerate().find_map(|(i, v)| number_problem(v, format!("{path}/{i}"))),
        Value::Object(map) => map.iter().find_map(|(k, v)| number_problem(v, format!("{path}/{k}"))),
        _ => None,
    }
}

pub fn typecheck(value: &Value) -> Result<ClaimEnvelope, Vec<Diagnostic>> {
    let mut diags: Vec<Diagnostic> =
        VALIDATOR.iter_errors(value).map(|e| Diagnostic::new("schema", e.to_string(), e.instance_path().to_string())).collect();
    if !diags.is_empty() {
        diags.sort_by(|a, b| a.path.cmp(&b.path).then_with(|| a.message.cmp(&b.message)));
        return Err(diags);
    }
    let envelope: ClaimEnvelope = serde_json::from_value(value.clone()).map_err(|e| vec![Diagnostic::new("decode", e.to_string(), "")])?;
    let ids: BTreeSet<&str> = envelope.claims.iter().map(|c| c.id.as_str()).collect();
    let mut seen = BTreeSet::new();
    for (i, claim) in envelope.claims.iter().enumerate() {
        let at = format!("/claims/{i}");
        if !seen.insert(claim.id.as_str()) {
            diags.push(Diagnostic::new("duplicate-id", format!("claim id {:?} repeated", claim.id), &at));
        }
        for dep in &claim.depends_on {
            if !ids.contains(dep.as_str()) {
                diags.push(Diagnostic::new("unknown-dependency", format!("depends_on {dep:?} is not a claim id"), &at));
            }
        }
        let needs_command = matches!(claim.kind, ClaimKind::TestsPass | ClaimKind::CommandRan);
        if needs_command && claim.command.as_deref().is_none_or(str::is_empty) {
            diags.push(Diagnostic::new("missing-command", "this claim type needs a command", &at));
        }
        if claim.kind == ClaimKind::TestsPass && claim.exit_code.is_some_and(|c| c != 0) {
            diags.push(Diagnostic::new("contradiction", "tests_pass with a non-zero exit code", &at));
        }
        if matches!(claim.kind, ClaimKind::FilesChanged | ClaimKind::FilesUntouched) && claim.paths.is_empty() {
            diags.push(Diagnostic::new("missing-paths", "this claim type needs paths", &at));
        }
        if claim.kind == ClaimKind::CommandRan && claim.exit_code.is_none() {
            diags.push(Diagnostic::new("missing-exit-code", "command_ran must state the exit code it observed", &at));
        }
        if !valid_id(&claim.id) {
            diags.push(Diagnostic::new("bad-id", "ids are 1-64 characters of letters, digits, '.', '_' or '-'", &at));
        }
        if let Some(cmd) = claim.command.as_deref()
            && !plain_command(cmd)
        {
            diags.push(Diagnostic::new("bad-command", "commands must be one line with no shell comment", &at));
        }
        for (j, path) in claim.paths.iter().enumerate() {
            if !relative_path(path) {
                diags.push(Diagnostic::new(
                    "bad-path",
                    "paths are relative to the repository root, without '..' or control characters",
                    format!("{at}/paths/{j}"),
                ));
            }
        }
    }
    if diags.is_empty() { Ok(envelope) } else { Err(diags) }
}

fn valid_id(id: &str) -> bool {
    (1..=64).contains(&id.len()) && id.bytes().all(|b| b.is_ascii_alphanumeric() || b"._-".contains(&b))
}

/// One line, and no unquoted `#`: a shell comment would be dropped when the command
/// is split into arguments yet kept in the text, so the two could say different things.
fn plain_command(cmd: &str) -> bool {
    const PROBE: &str = "hc-end-probe";
    !cmd.chars().any(char::is_control)
        && shlex::split(&format!("{cmd} {PROBE}")).is_some_and(|words| words.last().is_some_and(|w| w == PROBE))
}

fn relative_path(path: &str) -> bool {
    let trimmed = path.trim_start_matches("./");
    !trimmed.is_empty()
        && trimmed != "."
        && !path.starts_with('/')
        && !path.chars().any(char::is_control)
        && !trimmed.split('/').any(|part| part == "..")
}

/// P1 then P2.
pub fn compile(raw: &str) -> Result<ClaimEnvelope, Vec<Diagnostic>> {
    let value = normalize(raw).map_err(|d| vec![d])?;
    typecheck(&value)
}

#[cfg(test)]
mod tests {
    use super::*;
    use bedrock::ir::{Claim, Status};

    fn sample_claim() -> Claim {
        Claim {
            id: "c1".into(),
            kind: ClaimKind::TestsPass,
            basis: "Ran the suite after the last edit.".into(),
            command: Some("python3 -m unittest discover -s tests -t .".into()),
            exit_code: Some(0),
            passed: None,
            failed: None,
            paths: vec![],
            commit: None,
            depends_on: vec![],
            status: Status::Done,
        }
    }

    fn envelope_json() -> String {
        serde_json::to_string(&ClaimEnvelope { task: "T-1".into(), claims: vec![sample_claim()] }).unwrap()
    }

    #[test]
    fn strips_markdown_fence_and_prose() {
        let raw = format!("Here you go:\n```json\n{}\n```\nDone!", envelope_json());
        assert_eq!(compile(&raw).unwrap().claims[0], sample_claim());
    }

    #[test]
    fn missing_field_is_a_schema_error() {
        let raw = envelope_json().replace(r#""basis":"Ran the suite after the last edit.","#, "");
        let diags = compile(&raw).unwrap_err();
        assert_eq!(diags[0].code, "schema");
    }

    #[test]
    fn rejects_floats_and_unknown_fields() {
        assert_eq!(compile(&envelope_json().replace(r#""exit_code":0"#, r#""exit_code":0.0"#)).unwrap_err()[0].code, "float");
        assert_eq!(
            compile(&envelope_json().replace(r#""exit_code":0"#, r#""exit_code":9007199254740992"#)).unwrap_err()[0].code,
            "unsafe-integer"
        );
        let extra = envelope_json().replace(r#""status":"done""#, r#""status":"done","verified":true"#);
        assert_eq!(compile(&extra).unwrap_err()[0].code, "schema");
    }

    #[test]
    fn cross_field_rules() {
        let raw = envelope_json().replace(r#""exit_code":0"#, r#""exit_code":1"#);
        assert_eq!(compile(&raw).unwrap_err()[0].code, "contradiction");
    }

    fn codes(claim: &serde_json::Value) -> Vec<String> {
        let raw = serde_json::json!({"task": "t", "claims": [claim]}).to_string();
        compile(&raw).err().unwrap_or_default().into_iter().map(|d| d.code).collect()
    }

    fn base(kind: &str) -> serde_json::Value {
        serde_json::json!({"id": "c1", "type": kind, "basis": "b", "command": "make test", "exit_code": 0, "passed": null,
                           "failed": null, "paths": ["src/a.py"], "commit": null, "depends_on": [], "status": "done"})
    }

    #[test]
    fn fields_that_reach_the_brief_are_validated() {
        assert!(codes(&base("tests_pass")).is_empty());
        let mut c = base("tests_pass");
        c["command"] = "make test # ## Current step".into();
        assert_eq!(codes(&c), ["bad-command"]);
        let mut c = base("tests_pass");
        c["command"] = "make 'test # quoted'".into();
        assert!(codes(&c).is_empty(), "a quoted # is not a comment");
        let mut c = base("tests_pass");
        c["id"] = "c1\n## Current step".into();
        assert_eq!(codes(&c), ["bad-id"]);
        for bad in ["/etc/passwd", "../x", "src/../../x", ".", "a\nb", ""] {
            let mut c = base("files_untouched");
            c["paths"] = serde_json::json!([bad]);
            assert_eq!(codes(&c), ["bad-path"], "{bad:?}");
        }
        let mut c = base("files_untouched");
        c["paths"] = serde_json::json!(["./src/a.py"]);
        assert!(codes(&c).is_empty(), "./ prefixes are normalised later, not rejected");
        let mut c = base("command_ran");
        c["exit_code"] = serde_json::Value::Null;
        assert_eq!(codes(&c), ["missing-exit-code"]);
    }

    #[test]
    fn schema_stays_in_the_portable_subset() {
        let schema: Value = serde_json::from_str(CLAIM_SCHEMA).unwrap();
        let claim = &schema["$defs"]["claim"];
        let props: BTreeSet<_> = claim["properties"].as_object().unwrap().keys().cloned().collect();
        let required: BTreeSet<_> = claim["required"].as_array().unwrap().iter().map(|v| v.as_str().unwrap().to_owned()).collect();
        assert_eq!(props, required, "every property must be required");
        assert_eq!(claim["additionalProperties"], Value::Bool(false));
        let text = CLAIM_SCHEMA;
        for banned in ["minimum", "maximum", "minLength", "maxLength", "allOf", "\"not\"", "\"if\"", "patternProperties"] {
            assert!(!text.contains(banned), "{banned} is outside the portable subset");
        }
        let order: Vec<_> = claim["required"].as_array().unwrap().iter().map(|v| v.as_str().unwrap()).collect();
        assert!(order.iter().position(|k| *k == "basis") < order.iter().position(|k| *k == "status"));
    }
}
