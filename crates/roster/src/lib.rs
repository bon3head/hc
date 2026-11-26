//! roster: knows who's who.
//!
//! Agents are data (`agents.json`, extended or overridden by `<hc home>/roster.json`).
//! Adding an agent means adding an entry: its executable names, the
//! instructions file it reads, and optionally how to launch it headless and
//! where its final structured message lands.
//!
//! Identity is attested, never self-declared: either `hc run` launched the
//! process (`Launcher`), or `hc` finds a registered executable among its own
//! process ancestors (`Ancestry`), read from the kernel's process table.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use bedrock::ir::{AgentAttestation, AttestationLevel};
use serde::{Deserialize, Serialize};

const BUILTIN: &str = include_str!("../agents.json");

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum OutputSpec {
    StdoutJsonField { field: String },
    File,
    Stdout,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Launch {
    pub argv: Vec<String>,
    pub output: OutputSpec,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentDef {
    pub id: String,
    pub vendor: String,
    pub executables: Vec<String>,
    pub instructions_file: String,
    pub launch: Option<Launch>,
}

#[derive(Debug, thiserror::Error)]
pub enum RosterError {
    #[error("roster file {path}: {source}")]
    Parse { path: PathBuf, source: serde_json::Error },
    #[error("i/o error")]
    Io(#[from] std::io::Error),
    #[error("unknown agent {0:?}")]
    Unknown(String),
    #[error("agent {0:?} has no headless launch template")]
    NoLaunch(String),
}

#[derive(Debug, Clone)]
pub struct Registry {
    agents: BTreeMap<String, AgentDef>,
}

impl Registry {
    pub fn builtin() -> Self {
        let defs: Vec<AgentDef> = serde_json::from_str(BUILTIN).expect("builtin roster parses");
        Self { agents: defs.into_iter().map(|d| (d.id.clone(), d)).collect() }
    }

    /// Built-in entries, overridden or extended by `path` when it exists.
    pub fn load(path: &Path) -> Result<Self, RosterError> {
        let mut reg = Self::builtin();
        if path.exists() {
            let text = std::fs::read_to_string(path)?;
            let defs: Vec<AgentDef> = serde_json::from_str(&text).map_err(|source| RosterError::Parse { path: path.into(), source })?;
            for def in defs {
                reg.agents.insert(def.id.clone(), def);
            }
        }
        Ok(reg)
    }

    pub fn get(&self, id: &str) -> Result<&AgentDef, RosterError> {
        self.agents.get(id).ok_or_else(|| RosterError::Unknown(id.into()))
    }

    pub fn iter(&self) -> impl Iterator<Item = &AgentDef> {
        self.agents.values()
    }

    fn by_executable(&self, name: &str) -> Option<&AgentDef> {
        self.agents.values().find(|d| d.executables.iter().any(|e| e == name))
    }

    /// Walk the process ancestry from `pid` and return the nearest registered agent.
    pub fn detect_from(&self, pid: u32) -> Option<AgentAttestation> {
        let mut current = Some(pid);
        for _ in 0..64 {
            let p = current?;
            let info = proc_info(p)?;
            if let Some(def) = info.names.iter().find_map(|n| self.by_executable(n)) {
                return Some(AgentAttestation {
                    agent_id: def.id.clone(),
                    level: AttestationLevel::Ancestry,
                    executable: info.exe,
                    pid: Some(p),
                });
            }
            current = info.ppid.filter(|pp| *pp != 0 && *pp != p);
        }
        None
    }

    pub fn detect(&self) -> Option<AgentAttestation> {
        self.detect_from(std::process::id())
    }
}

pub fn launcher_attestation(def: &AgentDef, executable: Option<String>, pid: Option<u32>) -> AgentAttestation {
    AgentAttestation { agent_id: def.id.clone(), level: AttestationLevel::Launcher, executable, pid }
}

pub fn declared(agent_id: &str) -> AgentAttestation {
    AgentAttestation { agent_id: agent_id.to_owned(), level: AttestationLevel::Declared, executable: None, pid: None }
}

/// Fill `{prompt}`, `{schema_path}`, `{schema_json}` and `{out_path}` in a launch template.
pub fn render_argv(template: &[String], vars: &BTreeMap<&str, String>) -> Vec<String> {
    template.iter().map(|arg| vars.iter().fold(arg.clone(), |acc, (k, v)| acc.replace(&format!("{{{k}}}"), v))).collect()
}

struct ProcInfo {
    ppid: Option<u32>,
    exe: Option<String>,
    names: Vec<String>,
}

fn basename(s: &str) -> String {
    Path::new(s).file_name().map_or_else(|| s.to_owned(), |n| n.to_string_lossy().into_owned())
}

/// Interpreters whose first argument is the actual program (node, bun, python wrappers).
const INTERPRETERS: [&str; 6] = ["node", "bun", "deno", "python", "python3", "env"];

fn candidate_names(exe: Option<&str>, argv: &[String]) -> Vec<String> {
    let mut names = Vec::new();
    if let Some(e) = exe {
        names.push(basename(e));
    }
    if let Some(first) = argv.first() {
        names.push(basename(first));
        if INTERPRETERS.contains(&basename(first).as_str())
            && let Some(script) = argv.iter().skip(1).find(|a| !a.starts_with('-'))
        {
            let name = basename(script);
            names.push(name.trim_end_matches(".js").trim_end_matches(".mjs").to_owned());
        }
    }
    names
}

#[cfg(target_os = "linux")]
fn proc_info(pid: u32) -> Option<ProcInfo> {
    let stat = std::fs::read_to_string(format!("/proc/{pid}/stat")).ok()?;
    let after_comm = &stat[stat.rfind(')')? + 2..];
    let parent = after_comm.split(' ').nth(1).and_then(|s| s.parse().ok());
    let exe = std::fs::read_link(format!("/proc/{pid}/exe")).ok().map(|p| p.to_string_lossy().into_owned());
    let argv: Vec<String> = std::fs::read(format!("/proc/{pid}/cmdline"))
        .ok()
        .map(|b| b.split(|c| *c == 0).filter(|s| !s.is_empty()).map(|s| String::from_utf8_lossy(s).into_owned()).collect())
        .unwrap_or_default();
    Some(ProcInfo { ppid: parent, names: candidate_names(exe.as_deref(), &argv), exe })
}

#[cfg(not(target_os = "linux"))]
fn proc_info(pid: u32) -> Option<ProcInfo> {
    let out = std::process::Command::new("ps").args(["-o", "ppid=", "-o", "args=", "-p", &pid.to_string()]).output().ok()?;
    let line = String::from_utf8_lossy(&out.stdout).trim().to_owned();
    let (ppid, args) = line.split_once(char::is_whitespace)?;
    let argv: Vec<String> = args.split_whitespace().map(str::to_owned).collect();
    Some(ProcInfo { ppid: ppid.trim().parse().ok(), exe: argv.first().cloned(), names: candidate_names(None, &argv) })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builtin_roster_is_well_formed() {
        let reg = Registry::builtin();
        for def in reg.iter() {
            if let Some(launch) = &def.launch {
                assert!(launch.argv.iter().any(|a| a.contains("{prompt}")), "{} must take the prompt", def.id);
                if launch.output == OutputSpec::File {
                    assert!(launch.argv.iter().any(|a| a.contains("{out_path}")), "{} writes a file", def.id);
                }
            }
        }
        assert!(reg.get("codex").unwrap().launch.is_some());
    }

    #[test]
    fn user_entries_extend_and_override() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("roster.json");
        std::fs::write(&path, r#"[{"id":"neon","vendor":"local","executables":["neon"],"instructions_file":"AGENTS.md","launch":null},
                                  {"id":"codex","vendor":"openai","executables":["codex","codex-dev"],"instructions_file":"AGENTS.md","launch":null}]"#).unwrap();
        let reg = Registry::load(&path).unwrap();
        assert_eq!(reg.get("neon").unwrap().vendor, "local");
        assert!(reg.by_executable("codex-dev").is_some());
        assert!(reg.get("codex").unwrap().launch.is_none());
    }

    #[test]
    fn interpreter_wrapped_clis_are_recognised() {
        let names = candidate_names(Some("/usr/bin/node"), &["node".into(), "--no-warnings".into(), "/opt/lib/codex.js".into()]);
        assert!(names.contains(&"codex".to_owned()));
    }

    #[test]
    fn detects_a_registered_ancestor() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("roster.json");
        let me = proc_info(std::process::id()).unwrap();
        let parent_name = me.ppid.and_then(proc_info).and_then(|p| p.names.into_iter().next()).unwrap();
        std::fs::write(
            &path,
            format!(r#"[{{"id":"probe","vendor":"test","executables":["{parent_name}"],"instructions_file":"AGENTS.md","launch":null}}]"#),
        )
        .unwrap();
        let found = Registry::load(&path).unwrap().detect().unwrap();
        assert_eq!(found.level, AttestationLevel::Ancestry);
        assert_eq!(found.pid, me.ppid);
    }

    #[test]
    fn renders_templates() {
        let vars = BTreeMap::from([("prompt", "do it".to_owned()), ("out_path", "/o.json".to_owned())]);
        let argv = render_argv(&["codex".into(), "-o".into(), "{out_path}".into(), "{prompt}".into()], &vars);
        assert_eq!(argv, ["codex", "-o", "/o.json", "do it"]);
    }
}
