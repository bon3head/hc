//! Can a local model file claims hc accepts? Each scenario describes a
//! finished session in plain words; the model must turn it into a claim
//! envelope. Two modes are compared: the schema enforced at decode time
//! (Ollama `format` set to the claim schema) and prompt-only (the schema in
//! the prompt). Scored deterministically: valid on first try (front end P1+P2)
//! and semantically right (the expected claim kinds with the right fields).
//!
//! Request and response shapes follow Ollama's `/api/chat` reference
//! ("Chat request (Structured outputs)").

use std::time::Instant;

use bedrock::ir::{ClaimEnvelope, ClaimKind};
use serde::Serialize;
use serde_json::{Value, json};

#[derive(Debug)]
pub struct Scenario {
    pub id: &'static str,
    pub story: &'static str,
    pub expect: &'static [ClaimKind],
    pub command: Option<&'static str>,
    pub commit: Option<&'static str>,
}

const CMD: &str = "python3 -m unittest discover -s tests -t .";

pub const SCENARIOS: [Scenario; 6] = [
    Scenario {
        id: "fixed-and-committed",
        story: "You changed the operator in src/calc.py from minus to plus, committed it as 4f2c9e1, then ran `python3 -m unittest discover -s tests -t .` and all 3 tests passed. You did not touch any other file.",
        expect: &[ClaimKind::TestsPass, ClaimKind::FilesChanged],
        command: Some(CMD),
        commit: Some("4f2c9e1"),
    },
    Scenario {
        id: "uncommitted",
        story: "You fixed src/parser.py and src/lexer.py but did not commit. Running `python3 -m unittest discover -s tests -t .` passed 12 tests, 0 failed.",
        expect: &[ClaimKind::TestsPass, ClaimKind::FilesChanged],
        command: Some(CMD),
        commit: None,
    },
    Scenario {
        id: "spec-conflict",
        story: "The task says mul must multiply, but tests/test_calc.py asserts mul(1, 7) == 8. You did not change anything because the test contradicts the specification.",
        expect: &[ClaimKind::SpecConflict],
        command: None,
        commit: None,
    },
    Scenario {
        id: "no-change",
        story: "You inspected src/calc.py and the add function is already correct; the suite already passes. You changed nothing.",
        expect: &[ClaimKind::NoChangeNeeded],
        command: None,
        commit: None,
    },
    Scenario {
        id: "blocked",
        story: "Installing dependencies needs credentials for a private package index that you do not have, so you could not run or change anything.",
        expect: &[ClaimKind::Blocked],
        command: None,
        commit: None,
    },
    Scenario {
        id: "untouched-tests",
        story: "You fixed src/calc.py, committed it as 9ab31c0, and deliberately left tests/test_calc.py unmodified. The suite `python3 -m unittest discover -s tests -t .` passes.",
        expect: &[ClaimKind::TestsPass, ClaimKind::FilesUntouched],
        command: Some(CMD),
        commit: Some("9ab31c0"),
    },
];

pub fn prompt(s: &Scenario, constrained: bool) -> String {
    let mut p = format!(
        "You are a coding agent reporting what you did in a session on task \"probe\". Report only what the session description states.\n\nSession: {}\n\nReturn one JSON object with fields \"task\" (\"probe\") and \"claims\". Every claim has all fields: id, type, basis, command, exit_code, passed, failed, paths, commit, depends_on, status. Unused fields are null or []. Write basis before status.",
        s.story
    );
    if !constrained {
        p.push_str("\n\nThe JSON must match this schema and nothing else may be output:\n");
        p.push_str(intake::CLAIM_SCHEMA);
    }
    p
}

pub fn request_body(model: &str, s: &Scenario, constrained: bool, seed: u64) -> Value {
    let mut body = json!({
        "model": model,
        "messages": [{"role": "user", "content": prompt(s, constrained)}],
        "stream": false,
        "options": {"temperature": 0, "seed": seed}
    });
    if constrained {
        body["format"] = serde_json::from_str(intake::CLAIM_SCHEMA).expect("schema parses");
    }
    body
}

pub fn content(response: &Value) -> Option<&str> {
    response.pointer("/message/content").and_then(Value::as_str)
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Score {
    pub valid: bool,
    pub semantic: bool,
    pub first_diagnostic: Option<String>,
}

pub fn score(raw: &str, s: &Scenario) -> Score {
    match intake::compile(raw) {
        Err(diags) => {
            Score { valid: false, semantic: false, first_diagnostic: diags.first().map(|d| format!("{}: {}", d.code, d.message)) }
        }
        Ok(envelope) => Score { valid: true, semantic: semantic(&envelope, s), first_diagnostic: None },
    }
}

fn semantic(e: &ClaimEnvelope, s: &Scenario) -> bool {
    s.expect.iter().all(|kind| e.claims.iter().any(|c| c.kind == *kind))
        && e.claims
            .iter()
            .filter(|c| c.kind == ClaimKind::TestsPass)
            .all(|c| c.command.as_deref() == s.command && c.commit.as_deref() == s.commit)
}

#[derive(Debug, Clone, Serialize)]
pub struct ModeResult {
    pub model: String,
    pub constrained: bool,
    pub trials: usize,
    pub valid: usize,
    pub semantic: usize,
    pub median_ms: u128,
    pub failures: Vec<(String, String)>,
}

/// Run every scenario `repeats` times per model and mode against an Ollama server.
pub fn run(base_url: &str, models: &[String], repeats: u64) -> Result<Vec<ModeResult>, ureq::Error> {
    let url = format!("{}/api/chat", base_url.trim_end_matches('/'));
    let mut results = Vec::new();
    for model in models {
        for constrained in [true, false] {
            let mut r =
                ModeResult { model: model.clone(), constrained, trials: 0, valid: 0, semantic: 0, median_ms: 0, failures: Vec::new() };
            let mut times = Vec::new();
            for seed in 0..repeats {
                for s in &SCENARIOS {
                    let started = Instant::now();
                    let response: Value = ureq::post(&url).send_json(request_body(model, s, constrained, seed))?.body_mut().read_json()?;
                    times.push(started.elapsed().as_millis());
                    let sc = score(content(&response).unwrap_or_default(), s);
                    r.trials += 1;
                    r.valid += usize::from(sc.valid);
                    r.semantic += usize::from(sc.semantic);
                    if !sc.semantic {
                        r.failures.push((s.id.to_owned(), sc.first_diagnostic.unwrap_or_else(|| "wrong claims".into())));
                    }
                }
            }
            times.sort_unstable();
            r.median_ms = times.get(times.len() / 2).copied().unwrap_or(0);
            results.push(r);
        }
    }
    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn claim(kind: &str, command: Option<&str>, commit: Option<&str>, paths: &[&str], status: &str) -> Value {
        json!({"id": kind, "type": kind, "basis": "b", "command": command, "exit_code": command.map(|_| 0), "passed": null, "failed": null,
               "paths": paths, "commit": commit, "depends_on": [], "status": status})
    }

    #[test]
    fn constrained_requests_carry_the_schema_and_prompt_only_ones_embed_it() {
        let c = request_body("m", &SCENARIOS[0], true, 1);
        assert_eq!(c["format"]["additionalProperties"], json!(false));
        assert!(!c["messages"][0]["content"].as_str().unwrap().contains("$defs"));
        let p = request_body("m", &SCENARIOS[0], false, 1);
        assert!(p.get("format").is_none());
        assert!(p["messages"][0]["content"].as_str().unwrap().contains("$defs"));
        assert_eq!(p["options"]["temperature"], 0);
    }

    #[test]
    fn scoring_separates_validity_from_meaning() {
        let s = &SCENARIOS[0];
        let good = json!({"task": "probe", "claims": [claim("tests_pass", Some(CMD), Some("4f2c9e1"), &[], "done"), claim("files_changed", None, None, &["src/calc.py"], "done")]});
        assert_eq!(score(&good.to_string(), s), Score { valid: true, semantic: true, first_diagnostic: None });
        let wrong_commit = json!({"task": "probe", "claims": [claim("tests_pass", Some(CMD), None, &[], "done"), claim("files_changed", None, None, &["src/calc.py"], "done")]});
        assert!(!score(&wrong_commit.to_string(), s).semantic);
        let fenced = format!("```json\n{good}\n```");
        assert!(score(&fenced, s).valid, "fences are recovered by P1");
        assert!(!score("{\"task\": \"probe\"}", s).valid);
        let response = json!({"message": {"role": "assistant", "content": good.to_string()}});
        assert!(score(content(&response).unwrap(), s).semantic);
    }
}
