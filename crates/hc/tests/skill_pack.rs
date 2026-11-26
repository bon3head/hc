//! Conformance of `integration/skills/*` to the Agent Skills specification
//! (agentskills.io/specification), mirroring the checks in the reference
//! validator `skills-ref` (`validator.py`: name, description, compatibility
//! limits and allowed frontmatter fields), plus Anthropic's authoring rules:
//! no reserved words or XML tags in metadata, body under 500 lines,
//! references one level deep, contents list for references over 100 lines.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

const ALLOWED_FIELDS: [&str; 6] = ["name", "description", "license", "compatibility", "metadata", "allowed-tools"];

fn skills_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../integration/skills")
}

fn frontmatter(text: &str) -> (BTreeMap<String, String>, &str) {
    let rest = text.strip_prefix("---\n").expect("SKILL.md starts with YAML frontmatter");
    let end = rest.find("\n---\n").expect("frontmatter is closed");
    let mut fields = BTreeMap::new();
    for line in rest[..end].lines().filter(|l| !l.starts_with(' ') && l.contains(':')) {
        let (k, v) = line.split_once(':').unwrap();
        fields.insert(k.trim().to_owned(), v.trim().to_owned());
    }
    (fields, &rest[end + 5..])
}

fn links(markdown: &str) -> Vec<String> {
    markdown
        .match_indices("](")
        .filter_map(|(i, _)| markdown[i + 2..].split_once(')').map(|(target, _)| target.to_owned()))
        .filter(|t| !t.starts_with("http"))
        .collect()
}

#[test]
fn skills_conform_to_the_agent_skills_spec() {
    let mut count = 0;
    for entry in std::fs::read_dir(skills_dir()).unwrap() {
        let dir = entry.unwrap().path();
        let dir_name = dir.file_name().unwrap().to_string_lossy().into_owned();
        let text = std::fs::read_to_string(dir.join("SKILL.md")).unwrap();
        let (fields, body) = frontmatter(&text);
        for key in fields.keys() {
            assert!(ALLOWED_FIELDS.contains(&key.as_str()), "{dir_name}: unexpected field {key}");
        }

        let name = &fields["name"];
        assert_eq!(name, &dir_name, "name must match the directory");
        assert!((1..=64).contains(&name.len()));
        assert!(name.bytes().all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-'));
        assert!(!name.starts_with('-') && !name.ends_with('-') && !name.contains("--"));
        assert!(!name.contains("anthropic") && !name.contains("claude"), "reserved word in {name}");

        let description = &fields["description"];
        assert!((1..=1024).contains(&description.len()), "{name}: description length");
        assert!(!description.contains('<'), "{name}: no XML tags in description");
        assert!(!description.starts_with("I ") && !description.starts_with("You "), "{name}: third person");
        if let Some(c) = fields.get("compatibility") {
            assert!((1..=500).contains(&c.len()));
        }

        assert!(body.lines().count() < 500, "{name}: body under 500 lines");
        for target in links(body) {
            let path = dir.join(&target);
            assert!(path.exists(), "{name}: broken link {target}");
            let reference = std::fs::read_to_string(&path).unwrap();
            assert!(
                links(&reference).iter().all(|t| Path::new(t).extension().is_none_or(|e| !e.eq_ignore_ascii_case("md"))),
                "{name}: {target} links to another reference; keep references one level deep"
            );
            if reference.lines().count() > 100 {
                assert!(reference.to_lowercase().contains("contents"), "{name}: {target} needs a table of contents");
            }
        }
        count += 1;
    }
    assert_eq!(count, 4);
}
