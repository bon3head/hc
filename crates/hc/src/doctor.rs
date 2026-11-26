//! `hc doctor`: can hc work on this machine and in this repository?
//!
//! Conventions shared by `flutter doctor`, `brew doctor`, `npm doctor`,
//! `expo-doctor` and `claude doctor`: grouped checks marked ✓ / ! / ✗; every
//! problem names the command that fixes it; warnings (works, degraded) are kept
//! apart from errors (will not work); terse by default, `-v` for detail,
//! `--json` for scripts; a nonzero exit only when something is broken; and the
//! doctor runs even when the tool's own state is what is broken. Checks exercise
//! the real code path where they can (sign and verify a docket entry, start the
//! sandbox) instead of only reading version strings.

use std::io::IsTerminal as _;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

use anyhow::Result;
use bedrock::ir::{Event, Obs, PREDICATE_OBS, Role};
use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Level {
    Ok,
    Info,
    Warn,
    Error,
}

#[derive(Debug, Serialize)]
pub struct Check {
    pub section: &'static str,
    pub name: String,
    pub level: Level,
    pub detail: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fix: Option<String>,
}

#[derive(Default)]
struct Report(Vec<Check>);

impl Report {
    fn add(&mut self, section: &'static str, name: impl Into<String>, level: Level, detail: impl Into<String>, fix: Option<String>) {
        self.0.push(Check { section, name: name.into(), level, detail: detail.into(), fix });
    }
    fn ok(&mut self, s: &'static str, n: impl Into<String>, d: impl Into<String>) {
        self.add(s, n, Level::Ok, d, None);
    }
    fn info(&mut self, s: &'static str, n: impl Into<String>, d: impl Into<String>) {
        self.add(s, n, Level::Info, d, None);
    }
    fn warn(&mut self, s: &'static str, n: impl Into<String>, d: impl Into<String>, fix: impl Into<String>) {
        self.add(s, n, Level::Warn, d, Some(fix.into()));
    }
    fn error(&mut self, s: &'static str, n: impl Into<String>, d: impl Into<String>, fix: impl Into<String>) {
        self.add(s, n, Level::Error, d, Some(fix.into()));
    }
}

pub fn which(exe: &str) -> Option<PathBuf> {
    if exe.contains('/') {
        let p = PathBuf::from(exe);
        return p.is_file().then_some(p);
    }
    std::env::split_paths(&std::env::var_os("PATH")?).map(|d| d.join(exe)).find(|p| p.is_file())
}

fn output(cmd: &mut Command) -> Option<String> {
    let out = cmd.output().ok()?;
    out.status.success().then(|| String::from_utf8_lossy(&out.stdout).trim().to_owned())
}

/// `git version 2.43.0` or `git version 2.39.3 (Apple Git-146)` → (2, 43).
pub fn parse_git_version(text: &str) -> Option<(u32, u32)> {
    let mut parts = text.split_whitespace().nth(2)?.split('.');
    Some((parts.next()?.parse().ok()?, parts.next()?.parse().ok()?))
}

fn install_hint(pkg: &str) -> String {
    match std::env::consts::OS {
        "macos" => format!("brew install {pkg}"),
        _ => format!("install {pkg} with your package manager (apt install {pkg}, dnf install {pkg}, pacman -S {pkg})"),
    }
}

fn system(r: &mut Report) {
    const S: &str = "System";
    let (os, arch) = (std::env::consts::OS, std::env::consts::ARCH);
    match os {
        "linux" => r.ok(S, "platform", format!("{os} {arch}")),
        "macos" => r.warn(
            S,
            "platform",
            format!("{os} {arch}: agent identity falls back to declared names and `hc run` is not isolated"),
            "full support needs Linux; run hc there (or in a Linux VM or container) for process-tree attestation and bubblewrap",
        ),
        _ => r.error(S, "platform", format!("{os} {arch} is not supported"), "run hc inside WSL 2 or a Linux container"),
    }

    // SSH signing arrived in Git 2.34 (Documentation/RelNotes/2.34.0.adoc), which also notes that
    // ssh-keygen from OpenSSH 8.7 is broken for it; the sign-and-verify round trip below catches that case.
    match which("git").and_then(|_| output(Command::new("git").arg("--version"))) {
        None => r.error(S, "git", "not found on PATH", install_hint("git")),
        Some(v) => match parse_git_version(&v) {
            Some(ver) if ver >= (2, 34) => r.ok(S, "git", format!("{}.{}", ver.0, ver.1)),
            Some(ver) => r.error(
                S,
                "git",
                format!("{}.{} cannot sign commits with SSH keys (needs 2.34+)", ver.0, ver.1),
                "upgrade git to 2.34 or newer",
            ),
            None => r.warn(S, "git", format!("unrecognised version string {v:?}"), "check that `git --version` works"),
        },
    }
    if which("ssh-keygen").is_none() {
        r.error(S, "ssh-keygen", "not found on PATH", install_hint("openssh-client"));
    }
    if which("git").is_some() && which("ssh-keygen").is_some() {
        match signing_round_trip() {
            Ok(()) => r.ok(S, "signed docket", "a test entry was signed and verified"),
            Err(e) => r.error(
                S,
                "signed docket",
                format!("signing round trip failed: {e:#}"),
                "run `hc doctor -v` and check git and OpenSSH versions",
            ),
        }
    }

    if os == "linux" {
        match which("bwrap") {
            None => r.warn(S, "bubblewrap", "not installed; `hc run` agents will not be isolated", install_hint("bubblewrap")),
            Some(_) if output(Command::new("bwrap").args(["--dev-bind", "/", "/", "--", "true"])).is_some() => {
                r.ok(S, "bubblewrap", "sandbox starts");
            }
            Some(_) => r.warn(
                S,
                "bubblewrap",
                "installed but cannot create a namespace (unprivileged user namespaces are restricted)",
                "allow them, e.g. `sudo sysctl kernel.apparmor_restrict_unprivileged_userns=0` on Ubuntu 24.04+, or run without isolation",
            ),
        }
        if std::fs::read_to_string("/proc/self/stat").is_ok() {
            r.ok(S, "process tree", "agents are identified by ancestry");
        } else {
            r.warn(
                S,
                "process tree",
                "/proc is not readable; agents fall back to declared names",
                "mount /proc (containers: do not mask it)",
            );
        }
    }
}

/// Sign and verify one entry in a throwaway docket: exercises ssh-keygen, git's
/// SSH signing and the allowed-signers check exactly as real sessions do.
fn signing_round_trip() -> Result<()> {
    let dir = tempfile::tempdir()?;
    let keys = seal::Keyring::new(dir.path().join("keys"));
    let harness = keys.ensure(Role::Harness, "doctor")?;
    let docket = docket::Docket::open(&dir.path().join("docket.git"), keys)?;
    let obs = Obs {
        session: "doctor".into(),
        ts: "2026-01-01T00:00:00Z".into(),
        event: Event::SessionEnd { exit_code: Some(0), tree: "4b825dc642cb6eb9a060e54bf8d69288fbee4904".into() },
    };
    docket.append(&harness, "2026-01-01T00:00:00Z", PREDICATE_OBS, vec![], &obs)?;
    anyhow::ensure!(docket.records()?.len() == 1, "the signed entry did not verify");
    Ok(())
}

fn home_root() -> Option<PathBuf> {
    std::env::var_os("HC_HOME").map(PathBuf::from).or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".hc")))
}

fn home(r: &mut Report, verbose: bool) -> Option<chambers::Home> {
    const S: &str = "Home";
    let Some(root) = home_root() else {
        r.error(S, "location", "neither HC_HOME nor HOME is set", "export HOME or HC_HOME");
        return None;
    };
    if !root.exists() {
        r.info(S, "location", format!("{} (created by `hc init`)", root.display()));
        return None;
    }
    let probe = root.join(".doctor-write-test");
    if std::fs::write(&probe, b"").and_then(|()| std::fs::remove_file(&probe)).is_err() {
        r.error(
            S,
            "writable",
            format!("cannot write to {}", root.display()),
            format!("fix ownership: sudo chown -R \"$USER\" {}", root.display()),
        );
        return None;
    }
    r.ok(S, "location", root.display().to_string());
    let home = chambers::Home::locate().ok()?;
    let keyring = home.keyring();
    for role in [Role::Harness, Role::Kernel] {
        let principal = seal::principal(role, "local");
        let key = keyring.dir().join(principal.replace(['@', ':'], "_")).join("id_ed25519");
        if !key.exists() {
            r.info(S, principal, "no key yet (created by `hc init`)");
            continue;
        }
        match key_signs(&key, &principal, &keyring.allowed_signers()) {
            Ok(()) => r.ok(S, principal, if verbose { key.display().to_string() } else { "signs and verifies".into() }),
            Err(e) => r.error(S, principal, e, "remove the key directory and run `hc init` to regenerate it"),
        }
    }
    Some(home)
}

/// The key signs, and `allowed_signers` accepts the signature for its principal.
fn key_signs(key: &Path, principal: &str, allowed: &Path) -> Result<(), String> {
    let dir = tempfile::tempdir().map_err(|e| e.to_string())?;
    let msg = dir.path().join("msg");
    std::fs::write(&msg, b"hc doctor").map_err(|e| e.to_string())?;
    let sign = Command::new("ssh-keygen").args(["-Y", "sign", "-n", "git", "-f"]).arg(key).arg(&msg).output().map_err(|e| e.to_string())?;
    if !sign.status.success() {
        return Err(format!("cannot sign: {}", String::from_utf8_lossy(&sign.stderr).lines().next().unwrap_or("ssh-keygen failed")));
    }
    let verify = Command::new("ssh-keygen")
        .args(["-Y", "verify", "-n", "git", "-I", principal, "-f"])
        .arg(allowed)
        .arg("-s")
        .arg(msg.with_extension("sig"))
        .stdin(std::fs::File::open(&msg).map_err(|e| e.to_string())?)
        .output()
        .map_err(|e| e.to_string())?;
    if verify.status.success() { Ok(()) } else { Err("allowed_signers does not accept this key".into()) }
}

fn repository(r: &mut Report, repo: &Path, home: Option<&chambers::Home>) {
    const S: &str = "Repository";
    let inside =
        Command::new("git").args(["rev-parse", "--is-inside-work-tree"]).current_dir(repo).output().is_ok_and(|o| o.status.success());
    if !inside {
        r.error(S, "git work tree", format!("{} is not inside a git repository", repo.display()), "git init");
        return;
    }
    r.ok(S, "git work tree", repo.canonicalize().unwrap_or_else(|_| repo.to_path_buf()).display().to_string());

    let task_path = repo.join(".handoff/task.json");
    let task = match std::fs::read_to_string(&task_path) {
        Err(_) => {
            r.warn(S, "task", "no .handoff/task.json", "hc init \"<what the agent should do>\"");
            None
        }
        Ok(text) => match serde_json::from_str::<brief::Task>(&text) {
            Err(e) => {
                r.error(S, "task", format!(".handoff/task.json: {e}"), "fix the JSON, or delete it and run `hc init`");
                None
            }
            Ok(t) if t.objective.is_empty() => {
                r.warn(S, "task", "objective is empty", "hc init \"<what the agent should do>\"");
                Some(t)
            }
            Ok(t) => {
                r.ok(S, "task", t.objective.clone());
                Some(t)
            }
        },
    };

    if let Some(task) = &task {
        let first = shlex::split(&task.test_command).and_then(|v| v.into_iter().find(|w| !w.contains('=')));
        match first {
            None => r.error(S, "test command", format!("cannot parse {:?}", task.test_command), "set test_command in .handoff/task.json"),
            Some(exe) => match which(&exe).or_else(|| exe.contains('/').then(|| repo.join(&exe)).filter(|p| p.is_file())) {
                Some(path) => r.ok(S, "test command", format!("{}  ({})", task.test_command, path.display())),
                None => r.error(
                    S,
                    "test command",
                    format!("`{exe}` is not on PATH"),
                    format!("install {exe}, or change test_command in .handoff/task.json"),
                ),
            },
        }
        let block = courier::protocol_block(&task.test_command);
        let notes = repo.join("AGENTS.md");
        if !std::fs::read_to_string(&notes).is_ok_and(|t| t.contains(&block)) {
            r.warn(S, "agent notes", "AGENTS.md is missing the hc protocol or names an old test command", "hc init");
        }
    }

    for (dialect, _, exe, _) in crate::setup::HARNESSES {
        hooks(r, repo, dialect, which(exe).is_some());
    }

    if let Some(home) = home {
        let path = home.docket_path(repo);
        if path.as_ref().is_ok_and(|p| p.exists()) {
            match home.docket(repo).and_then(|d| Ok(d.records()?)) {
                Ok(recs) => r.ok(S, "docket", format!("{} entries, every signature valid", recs.len())),
                Err(e) => r.error(S, "docket", format!("{e:#}"), "the ledger was altered outside hc; keep it for inspection and report it"),
            }
        }
    }
}

fn hooks(r: &mut Report, repo: &Path, dialect: &str, installed: bool) {
    const S: &str = "Repository";
    let rel = courier::hooks::path(dialect).expect("known dialect");
    let name = format!("{dialect} hooks");
    let text = std::fs::read_to_string(repo.join(rel)).ok();
    let config = match text.as_deref().map(serde_json::from_str::<serde_json::Value>) {
        Some(Err(e)) => return r.error(S, name, format!("{rel} is not valid JSON: {e}"), format!("fix {rel}, then run `hc init`")),
        Some(Ok(v)) => v,
        None => serde_json::Value::Null,
    };
    let mut commands = Vec::new();
    hook_commands(&config, &mut commands);
    match (commands.first(), installed) {
        (None, true) => r.warn(S, name, format!("{dialect} is installed but {rel} has no hc hooks"), format!("hc init --for {dialect}")),
        (None, false) => {}
        (Some(cmd), _) => {
            let bin = cmd.split_whitespace().next().unwrap_or("hc");
            match which(bin) {
                Some(path) => r.ok(S, name, format!("{rel}  ({})", path.display())),
                None => r.error(
                    S,
                    name,
                    format!("{rel} runs `{bin}`, which is not on PATH; the harness cannot reach hc"),
                    "put hc on PATH (cargo install --path crates/hc), then run `hc init` again",
                ),
            }
        }
    }
}

/// Every `command` string in a hook config that invokes `hc hook`.
fn hook_commands(v: &serde_json::Value, out: &mut Vec<String>) {
    match v {
        serde_json::Value::String(s) if s.contains(courier::hooks::MARKER) => out.push(s.clone()),
        serde_json::Value::Array(items) => items.iter().for_each(|i| hook_commands(i, out)),
        serde_json::Value::Object(map) => map.values().for_each(|i| hook_commands(i, out)),
        _ => {}
    }
}

fn agents(r: &mut Report) {
    let found: Vec<&str> = crate::setup::HARNESSES.iter().filter(|h| which(h.2).is_some()).map(|h| h.1).collect();
    if found.is_empty() {
        r.info("Agents", "harnesses", "none with hooks found on PATH; MCP clients can use `hc mcp`");
    } else {
        r.ok("Agents", "harnesses", found.join(", "));
    }
}

/// The system checks `hc init` runs first, so setup stops before writing hooks that cannot work.
pub fn preflight() -> Result<()> {
    let mut r = Report::default();
    system(&mut r);
    let errors: Vec<String> =
        r.0.iter()
            .filter(|c| c.level == Level::Error)
            .map(|c| format!("  ✗ {}: {}\n    fix: {}", c.name, c.detail, c.fix.as_deref().unwrap_or("")))
            .collect();
    anyhow::ensure!(errors.is_empty(), "hc cannot run on this machine yet:\n{}", errors.join("\n"));
    Ok(())
}

pub fn run(repo: &Path, verbose: bool, json: bool) -> ExitCode {
    let mut r = Report::default();
    system(&mut r);
    let home = home(&mut r, verbose);
    repository(&mut r, repo, home.as_ref());
    agents(&mut r);
    let errors = r.0.iter().filter(|c| c.level == Level::Error).count();
    let warnings = r.0.iter().filter(|c| c.level == Level::Warn).count();

    if json {
        let out = serde_json::json!({"checks": r.0, "errors": errors, "warnings": warnings});
        println!("{}", serde_json::to_string_pretty(&out).expect("serializable"));
    } else {
        print_human(&r.0, verbose, errors, warnings);
    }
    if errors > 0 { ExitCode::FAILURE } else { ExitCode::SUCCESS }
}

fn print_human(checks: &[Check], verbose: bool, errors: usize, warnings: usize) {
    let color = std::io::stdout().is_terminal() && std::env::var_os("NO_COLOR").is_none();
    let paint = |code: &str, s: &str| if color { format!("\x1b[{code}m{s}\x1b[0m") } else { s.to_owned() };
    let width = checks.iter().map(|c| c.name.chars().count()).max().unwrap_or(0).min(24);
    let mut section = "";
    for c in checks.iter().filter(|c| verbose || c.level != Level::Info) {
        if c.section != section {
            section = c.section;
            println!("{}{}", if section == checks[0].section { "" } else { "\n" }, paint("1", section));
        }
        let mark = match c.level {
            Level::Ok => paint("32", "✓"),
            Level::Info => paint("2", "·"),
            Level::Warn => paint("33", "!"),
            Level::Error => paint("31", "✗"),
        };
        println!("  {mark} {:<width$}  {}", c.name, c.detail);
        if let Some(fix) = &c.fix {
            println!("    {} {fix}", paint("2", "fix:"));
        }
    }
    println!();
    match (errors, warnings) {
        (0, 0) => println!("{} hc is ready.", paint("32", "✓")),
        (0, w) => println!("{} hc will run, with {w} warning{}.", paint("33", "!"), if w == 1 { "" } else { "s" }),
        (e, _) => println!("{} {e} problem{} to fix; run `hc doctor` again after fixing.", paint("31", "✗"), if e == 1 { "" } else { "s" }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn git_versions_parse() {
        assert_eq!(parse_git_version("git version 2.43.0"), Some((2, 43)));
        assert_eq!(parse_git_version("git version 2.39.3 (Apple Git-146)"), Some((2, 39)));
        assert_eq!(parse_git_version("git version 2.25.1"), Some((2, 25)));
        assert_eq!(parse_git_version("nonsense"), None);
    }

    #[test]
    fn round_trip_passes_here() {
        signing_round_trip().unwrap();
    }
}
