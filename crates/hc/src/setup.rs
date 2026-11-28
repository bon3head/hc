//! `hc init`: everything a repository needs, in one idempotent step.

use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use brief::Task;

/// Harnesses with a native hook dialect: (hook dialect, roster id, executable, repo marker).
pub const HARNESSES: [(&str, &str, &str, &str); 3] = [
    ("claude", "claude-code", "claude", ".claude"),
    ("codex", "codex", "codex", ".codex"),
    ("antigravity", "antigravity", "agy", ".agents"),
];

pub struct Plan {
    pub test_command: String,
    pub test_source: &'static str,
    pub harnesses: Vec<&'static str>,
    pub roster_ids: Vec<&'static str>,
}

/// The test command a repository already declares, and the file that declares it.
pub fn detect_test_command(repo: &Path) -> Option<(String, &'static str)> {
    let has = |f: &str| repo.join(f).exists();
    if has("Cargo.toml") {
        return Some(("cargo test".into(), "Cargo.toml"));
    }
    if let Ok(text) = std::fs::read_to_string(repo.join("package.json"))
        && serde_json::from_str::<serde_json::Value>(&text).ok().is_some_and(|v| v["scripts"]["test"].is_string())
    {
        return Some(("npm test".into(), "package.json"));
    }
    if has("go.mod") {
        return Some(("go test ./...".into(), "go.mod"));
    }
    for f in ["pytest.ini", "pyproject.toml", "setup.cfg", "tox.ini"] {
        if has(f) {
            return Some(("python3 -m pytest".into(), f));
        }
    }
    if std::fs::read_dir(repo.join("tests")).is_ok_and(|d| {
        d.flatten().any(|e| e.file_name().to_string_lossy().starts_with("test_") && e.path().extension().is_some_and(|x| x == "py"))
    }) {
        return Some(("python3 -m unittest discover -s tests -t .".into(), "tests/"));
    }
    if std::fs::read_to_string(repo.join("Makefile")).is_ok_and(|m| m.lines().any(|l| l.starts_with("test:"))) {
        return Some(("make test".into(), "Makefile"));
    }
    None
}

fn on_path(exe: &str) -> bool {
    std::env::var_os("PATH").is_some_and(|p| std::env::split_paths(&p).any(|d| d.join(exe).is_file()))
}

/// Harnesses installed on this machine or already configured in the repository.
pub fn detect_harnesses(repo: &Path) -> Vec<(&'static str, &'static str)> {
    HARNESSES.iter().filter(|(_, _, exe, marker)| on_path(exe) || repo.join(marker).is_dir()).map(|&(d, id, _, _)| (d, id)).collect()
}

pub fn resolve(repo: &Path, test: Option<String>, only: &[String], existing: Option<&Task>) -> Result<Plan> {
    let (test_command, test_source) = match (test, existing) {
        (Some(t), _) => (t, "--test"),
        (None, Some(task)) => (task.test_command.clone(), ".handoff/task.json"),
        (None, None) => detect_test_command(repo).context("no test command found; pass one with --test \"<command>\"")?,
    };
    let found = if only.is_empty() {
        detect_harnesses(repo)
    } else {
        only.iter()
            .map(|name| {
                HARNESSES
                    .iter()
                    .find(|h| h.0 == name || h.1 == name)
                    .map(|&(d, id, _, _)| (d, id))
                    .with_context(|| format!("unknown harness {name:?} (claude, codex, antigravity)"))
            })
            .collect::<Result<_>>()?
    };
    Ok(Plan { test_command, test_source, harnesses: found.iter().map(|h| h.0).collect(), roster_ids: found.iter().map(|h| h.1).collect() })
}

/// Write `.handoff/task.json` when absent. When present, a given objective, a test
/// command passed explicitly with `--test`, and `--capture`, replace the stored ones;
/// everything else is kept. Returns the path and whether the file was created.
pub fn write_task(repo: &Path, objective: Option<&str>, test_command: &str, test_given: bool, capture: bool) -> Result<(PathBuf, bool)> {
    let path = repo.join(".handoff/task.json");
    if path.exists() {
        if objective.is_some() || test_given || capture {
            let mut task: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(&path)?)?;
            let Some(map) = task.as_object_mut() else { bail!("{} is not a JSON object", path.display()) };
            if let Some(objective) = objective {
                map.insert("objective".into(), objective.into());
            }
            if test_given {
                map.insert("test_command".into(), test_command.into());
            }
            if capture {
                map.insert("capture".into(), true.into());
            }
            std::fs::write(&path, serde_json::to_string_pretty(&task)? + "\n")?;
        }
        return Ok((path, false));
    }
    let id = repo.canonicalize()?.file_name().map_or_else(|| "task".into(), |n| n.to_string_lossy().into_owned());
    let q = |s: &str| serde_json::Value::from(s).to_string();
    let text = format!(
        "{{\n  \"id\": {},\n  \"objective\": {},\n  \"test_command\": {},\n  \"next_step\": \"\"{}\n}}\n",
        q(&id),
        q(objective.unwrap_or("")),
        q(test_command),
        if capture { ",\n  \"capture\": true" } else { "" }
    );
    std::fs::create_dir_all(path.parent().expect("has parent"))?;
    std::fs::write(&path, text)?;
    Ok((path, true))
}

/// Copy the owner's held-out probes from `src` into hc's home and name the check in the
/// task file. `src` must lie outside the repository: probes the agent can read are not held
/// out. Returns the number of files copied.
pub fn install_holdout(home: &Path, repo: &Path, task_path: &Path, src: &Path, command: &str) -> Result<usize> {
    let src = src.canonicalize().with_context(|| format!("holdout directory {}", src.display()))?;
    if src.starts_with(repo.canonicalize()?) {
        bail!("{} is inside the repository, where the agent can read it; keep held-out probes elsewhere", src.display());
    }
    if !command.contains("{holdout}") {
        bail!("--holdout-command must name the probes as {{holdout}}, e.g. \"python3 -I {{holdout}}/probes.py\"");
    }
    let mut task: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(task_path)?)?;
    let id = task.get("id").and_then(|v| v.as_str()).context("task has no id")?.to_owned();
    let dest = crossexam::holdout_dir(home, &id);
    if dest.exists() {
        std::fs::remove_dir_all(&dest)?;
    }
    let copied = copy_tree(&src, &dest)?;
    task.as_object_mut().context("task is not a JSON object")?.insert("holdout".into(), command.into());
    std::fs::write(task_path, serde_json::to_string_pretty(&task)? + "\n")?;
    Ok(copied)
}

fn copy_tree(src: &Path, dest: &Path) -> Result<usize> {
    std::fs::create_dir_all(dest)?;
    let mut n = 0;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let to = dest.join(entry.file_name());
        let kind = entry.file_type()?;
        if kind.is_dir() {
            n += copy_tree(&entry.path(), &to)?;
        } else if kind.is_file() {
            std::fs::copy(entry.path(), &to)?;
            n += 1;
        }
    }
    Ok(n)
}

pub fn install_hooks(repo: &Path, dialect: &str, bin: &str) -> Result<PathBuf> {
    let path = repo.join(courier::hooks::path(dialect).expect("known dialect"));
    let existing = std::fs::read_to_string(&path).ok().and_then(|t| serde_json::from_str(&t).ok()).unwrap_or_default();
    let merged = courier::hooks::merge(dialect, existing, bin);
    std::fs::create_dir_all(path.parent().expect("config has a parent"))?;
    std::fs::write(&path, serde_json::to_string_pretty(&merged)? + "\n")?;
    Ok(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_declared_test_commands() {
        let cases: [(&str, &str, &str); 5] = [
            ("Cargo.toml", "[package]", "cargo test"),
            ("package.json", r#"{"scripts":{"test":"vitest"}}"#, "npm test"),
            ("go.mod", "module x", "go test ./..."),
            ("pyproject.toml", "[project]", "python3 -m pytest"),
            ("Makefile", "build:\n\ttrue\ntest:\n\ttrue\n", "make test"),
        ];
        for (file, body, want) in cases {
            let dir = tempfile::tempdir().unwrap();
            std::fs::write(dir.path().join(file), body).unwrap();
            assert_eq!(detect_test_command(dir.path()).unwrap().0, want, "{file}");
        }
        let empty = tempfile::tempdir().unwrap();
        std::fs::write(empty.path().join("package.json"), r#"{"scripts":{}}"#).unwrap();
        assert!(detect_test_command(empty.path()).is_none());
    }

    #[test]
    fn repo_markers_select_harnesses_and_explicit_names_override() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir(dir.path().join(".codex")).unwrap();
        assert!(detect_harnesses(dir.path()).contains(&("codex", "codex")));
        let plan = resolve(dir.path(), Some("make test".into()), &["claude-code".into()], None).unwrap();
        assert_eq!(plan.harnesses, ["claude"]);
        assert!(resolve(dir.path(), Some("x".into()), &["cursor".into()], None).is_err());
    }

    #[test]
    fn task_file_is_written_once() {
        let dir = tempfile::tempdir().unwrap();
        let (path, created) = write_task(dir.path(), Some("Fix the parser"), "cargo test", false, false).unwrap();
        assert!(created);
        let task: Task = serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
        assert_eq!((task.objective.as_str(), task.test_command.as_str()), ("Fix the parser", "cargo test"));
        assert!(!write_task(dir.path(), None, "other", false, false).unwrap().1);
        write_task(dir.path(), Some("Fix the lexer"), "other", false, false).unwrap();
        let task: Task = serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
        assert_eq!((task.objective.as_str(), task.test_command.as_str()), ("Fix the lexer", "cargo test"));
    }

    #[test]
    fn capture_flag_is_stored_in_the_task_and_only_set_when_asked() {
        let dir = tempfile::tempdir().unwrap();
        let (path, _) = write_task(dir.path(), Some("Fix the parser"), "cargo test", false, false).unwrap();
        let task: Task = serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
        assert!(!task.capture);
        write_task(dir.path(), None, "cargo test", false, true).unwrap();
        let task: Task = serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
        assert!(task.capture, "--capture on an existing task file sets the flag");
        assert_eq!(task.objective, "Fix the parser", "and keeps the rest");
    }
}
