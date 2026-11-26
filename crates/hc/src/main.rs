//! hc: the handoff compiler.
//!
//! ```text
//! hc run --task task.json --agent codex     launch, observe, cross-examine, retry with a fresh brief
//! hc claim < claims.json                     an agent reports; identity is detected from the process tree
//! hc brief --task task.json                  print the compiled brief for the next agent
//! hc onboard --test-command "..." --hooks    protocol block + native hook configs for Claude Code, Codex, Antigravity
//! hc hook <dialect> <event>                  the hook entry point the harness calls (reads its stdin JSON)
//! hc conform --session <id>                  plan conformance recomputed from the docket
//! hc assay --seed 7                          acceptance run over the seeded corpus
//! ```

use std::collections::{BTreeMap, BTreeSet};
use std::io::Read as _;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use anyhow::{Context, Result, bail};
use bedrock::ir::{ClaimEnvelope, PassId, Role, Trust};
use brief::{Brief, Task};
use cell::Sandbox;
use chambers::Home;
use clap::{Parser, Subcommand};
use crossexam::{Finding, Kernel};
use exhibit::Recorder;
use roster::OutputSpec;

#[derive(Parser)]
#[command(name = "hc", version, about = "Deterministic, vendor-neutral handoff compiler for coding agents")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Create the hc home and the harness and kernel identities.
    Init,
    /// Print the claim schema agents report against.
    Schema,
    /// List registered agents and the one detected in this process tree.
    Agents,
    /// Report claims (JSON on stdin or --file). Run by agents.
    Claim {
        #[arg(long)]
        file: Option<PathBuf>,
        #[arg(long, default_value = ".")]
        repo: PathBuf,
        #[arg(long, env = "HC_SESSION")]
        session: Option<String>,
        /// Name to record when no registered agent is found among this process's ancestors.
        #[arg(long = "as")]
        declared_as: Option<String>,
    },
    /// Cross-examine pending claims for a task and print its brief.
    Brief {
        #[arg(long)]
        task: PathBuf,
        #[arg(long, default_value = ".")]
        repo: PathBuf,
    },
    /// Launch an agent on a task, verify its claims, and retry with a fresh brief on refutation.
    Run {
        #[arg(long)]
        task: PathBuf,
        #[arg(long)]
        agent: String,
        #[arg(long, default_value = ".")]
        repo: PathBuf,
        #[arg(long, default_value_t = 3)]
        max_attempts: u32,
        /// Run without bubblewrap even when it is available.
        #[arg(long)]
        no_isolation: bool,
    },
    /// Write the hc protocol block into each registered agent's instructions file.
    Onboard {
        #[arg(long)]
        test_command: String,
        #[arg(long, default_value = ".")]
        repo: PathBuf,
        /// Limit to these agent ids (default: every registered agent).
        #[arg(long, value_delimiter = ',')]
        agents: Vec<String>,
        /// Also install native hooks for these harnesses: claude, codex, antigravity.
        #[arg(long, value_delimiter = ',', num_args = 0.., default_missing_value = "claude,codex,antigravity")]
        hooks: Option<Vec<String>>,
        /// Command used in hook configs to invoke hc.
        #[arg(long, default_value = "hc")]
        hc_bin: String,
    },
    /// Native hook entry point. Reads the harness's hook JSON on stdin.
    Hook {
        /// claude (also used for codex) or antigravity
        dialect: String,
        /// One of `SessionStart`, `PreToolUse`, `PostToolUse`, `TaskCompleted`, `Stop`, `PostInvocation`
        event: String,
    },
    /// Serve the MCP attestation tier on stdio (for surfaces without hooks).
    Mcp {
        #[arg(long, default_value = ".")]
        repo: PathBuf,
    },
    /// Plan conformance for one session, recomputed from the docket.
    Conform {
        #[arg(long)]
        session: String,
        #[arg(long, default_value = ".")]
        repo: PathBuf,
    },
    /// Acceptance run over the seeded corpus of honest and lying sessions.
    Assay {
        #[arg(long, default_value_t = 7)]
        seed: u64,
        /// Passes to disable: command, tamper, scope, signature, reexec, falsify.
        #[arg(long, value_delimiter = ',', env = "HC_DISABLE")]
        disable: Vec<String>,
        #[arg(long)]
        work: Option<PathBuf>,
        #[arg(long)]
        json: bool,
    },
    /// Measure how reliably local models (Ollama) file valid, correct claims, with and without decode-time schema enforcement.
    ProbeModel {
        #[arg(long, default_value = "http://localhost:11434")]
        ollama: String,
        #[arg(long = "model", required = true)]
        models: Vec<String>,
        #[arg(long, default_value_t = 3)]
        repeats: u64,
    },
    /// Publish the task's brief as a Notion page (`NOTION_TOKEN` must be set).
    ExportNotion {
        #[arg(long)]
        task: PathBuf,
        #[arg(long)]
        data_source: String,
        #[arg(long, default_value = "Name")]
        title_property: String,
        #[arg(long, default_value = ".")]
        repo: PathBuf,
    },
}

fn now() -> String {
    jiff::Timestamp::now().strftime("%Y-%m-%dT%H:%M:%SZ").to_string()
}

fn parse_pass(name: &str) -> Result<PassId> {
    Ok(match name.trim().to_ascii_lowercase().as_str() {
        "command" | "p4" => PassId::Command,
        "tamper" | "p6" => PassId::Tamper,
        "scope" | "p6s" => PassId::Scope,
        "signature" | "p7" => PassId::Signature,
        "reexec" | "p8" => PassId::Reexec,
        "falsify" | "p9" => PassId::Falsify,
        other => bail!("unknown pass {other:?}; expected command, tamper, scope, signature, reexec or falsify"),
    })
}

fn load_task(path: &Path) -> Result<Task> {
    let text = std::fs::read_to_string(path).with_context(|| format!("reading task {}", path.display()))?;
    serde_json::from_str(&text).with_context(|| format!("parsing task {}", path.display()))
}

fn kernel_for(home: &Home, repo: &Path, task: &Task, isolate: bool) -> Result<Kernel> {
    let sandbox = if isolate { Sandbox::detect(home.root()) } else { Sandbox::none(home.root()) };
    Ok(Kernel::for_task(
        &task.test_command,
        task.allowed_paths.clone(),
        task.tests_writable,
        home.docket(repo)?.git_dir(),
        repo.to_path_buf(),
        sandbox,
    ))
}

fn adjudicate(home: &Home, repo: &Path, task: &Task, kernel: &Kernel) -> Result<Vec<Finding>> {
    Ok(kernel.adjudicate(&home.docket(repo)?, &home.identity(Role::Kernel, "local")?, &now(), &task.id)?)
}

fn print_findings(findings: &[Finding]) {
    for f in findings {
        let trust = serde_json::to_value(f.verdict.trust).unwrap_or_default();
        let failed: Vec<String> = f.verdict.checks.iter().filter_map(|c| c.rule.clone()).collect();
        let suffix = if failed.is_empty() { String::new() } else { format!(" ({})", failed.join(", ")) };
        eprintln!("[hc] {} {}: {}{suffix}", f.claim.session, f.claim.claim.id, trust.as_str().unwrap_or_default());
    }
}

fn cmd_claim(home: &Home, file: Option<&Path>, repo: &Path, session: Option<String>, declared_as: Option<&str>) -> Result<ExitCode> {
    let raw = if let Some(path) = file {
        std::fs::read_to_string(path)?
    } else {
        let mut s = String::new();
        std::io::stdin().read_to_string(&mut s)?;
        s
    };
    let envelope = match intake::compile(&raw) {
        Ok(e) => e,
        Err(diags) => {
            for d in diags {
                eprintln!("[hc] {} at {}: {}", d.code, if d.path.is_empty() { "/" } else { &d.path }, d.message);
            }
            eprintln!("[hc] fix the claims and run `hc claim` again; `hc schema` prints the schema");
            return Ok(ExitCode::from(2));
        }
    };
    if let Ok(outbox) = std::env::var("HC_OUTBOX") {
        std::fs::write(Path::new(&outbox).join("claims.json"), raw)?;
        eprintln!("[hc] {} claims queued for the harness", envelope.claims.len());
        return Ok(ExitCode::SUCCESS);
    }
    let registry = home.registry()?;
    let attestation = registry.detect().unwrap_or_else(|| roster::declared(declared_as.unwrap_or("unregistered")));
    let agent = home.identity(Role::Agent, &attestation.agent_id)?;
    let session = session.unwrap_or_else(|| format!("adhoc-{}", jiff::Timestamp::now().as_second()));
    let docket = home.docket(repo)?;
    exhibit::record_claims(&docket, &agent, &now(), &session, &attestation, &envelope)?;
    eprintln!("[hc] recorded {} claims from {} ({:?})", envelope.claims.len(), attestation.agent_id, attestation.level);
    Ok(ExitCode::SUCCESS)
}

fn extract_output(spec: &OutputSpec, stdout: &str, out_path: &Path) -> Option<String> {
    match spec {
        OutputSpec::StdoutJsonField { field } => {
            let value: serde_json::Value = serde_json::from_str(stdout.trim()).ok()?;
            value.get(field).filter(|v| !v.is_null()).map(ToString::to_string)
        }
        OutputSpec::File => std::fs::read_to_string(out_path).ok(),
        OutputSpec::Stdout => Some(stdout.to_owned()),
    }
}

fn cmd_run(home: &Home, task_path: &Path, agent_id: &str, repo: &Path, max_attempts: u32, isolate: bool) -> Result<ExitCode> {
    let task = load_task(task_path)?;
    let registry = home.registry()?;
    let def = registry.get(agent_id)?.clone();
    let launch = def.launch.clone().ok_or_else(|| roster::RosterError::NoLaunch(def.id.clone()))?;
    let docket = home.docket(repo)?;
    let harness = home.identity(Role::Harness, "local")?;
    let agent = home.identity(Role::Agent, &def.id)?;
    let kernel = kernel_for(home, repo, &task, isolate)?;
    let force = kernel.policy.protected.clone();
    let recorder = Recorder { docket: &docket, harness: &harness, force_paths: &force };
    let schema_enforced = launch.argv.iter().any(|a| a.contains("{schema_json}") || a.contains("{schema_path}"));

    for attempt in 1..=max_attempts {
        let findings = adjudicate(home, repo, &task, &kernel)?;
        let brief = Brief::link(task.clone(), &findings)?;
        let session = format!("{}-{}-{attempt}", task.id, jiff::Timestamp::now().as_second());
        let outbox = home.outbox(&session)?;
        let schema_path = outbox.join("claim.v0.json");
        std::fs::write(&schema_path, intake::CLAIM_SCHEMA)?;
        let out_path = outbox.join("final.json");
        let vars = BTreeMap::from([
            ("prompt", courier::prompt(&brief, schema_enforced)),
            ("schema_json", intake::CLAIM_SCHEMA.split_whitespace().collect::<Vec<_>>().join(" ")),
            ("schema_path", schema_path.display().to_string()),
            ("out_path", out_path.display().to_string()),
        ]);
        let argv = roster::render_argv(&launch.argv, &vars);
        let sandbox = if isolate { Sandbox::detect(home.root()) } else { Sandbox::none(home.root()) }.bind(&outbox);
        let attestation = roster::launcher_attestation(&def, Some(argv[0].clone()), None);
        let open = recorder.open(&now(), &session, &task.id, repo, attestation.clone(), sandbox.isolation)?;
        eprintln!("[hc] attempt {attempt}/{max_attempts}: {} in {:?} (brief {})", def.id, sandbox.isolation, brief.digest());
        let output = sandbox
            .command(&argv, repo)
            .env("HC_SESSION", &session)
            .env("HC_OUTBOX", &outbox)
            .output()
            .with_context(|| format!("launching {}", argv[0]))?;
        recorder.close(&now(), &open, output.status.code())?;
        let stdout = String::from_utf8_lossy(&output.stdout);
        let raw = extract_output(&launch.output, &stdout, &out_path).or_else(|| std::fs::read_to_string(outbox.join("claims.json")).ok());
        match raw.as_deref().map(intake::compile) {
            Some(Ok(envelope)) => {
                let envelope = ClaimEnvelope { task: task.id.clone(), ..envelope };
                exhibit::record_claims(&docket, &agent, &now(), &session, &attestation, &envelope)?;
            }
            Some(Err(diags)) => {
                recorder.no_claim(&now(), &session, diags)?;
            }
            None => {
                recorder.no_claim(
                    &now(),
                    &session,
                    vec![bedrock::ir::Diagnostic::new("no-output", "the agent produced no final message", "")],
                )?;
            }
        }
        let findings = adjudicate(home, repo, &task, &kernel)?;
        let mine: Vec<&Finding> = findings.iter().filter(|f| f.claim.session == session).collect();
        print_findings(&findings.iter().filter(|f| f.claim.session == session).cloned().collect::<Vec<_>>());
        let clean = !mine.is_empty() && mine.iter().all(|f| f.verdict.trust != Trust::Refuted);
        if clean {
            let brief = Brief::link(task.clone(), &findings)?;
            println!("{}", brief.render());
            return Ok(ExitCode::SUCCESS);
        }
    }
    let findings = adjudicate(home, repo, &task, &kernel)?;
    println!("{}", Brief::link(task, &findings)?.render());
    eprintln!("[hc] stopped after {max_attempts} attempts with refuted or missing claims");
    Ok(ExitCode::FAILURE)
}

fn cmd_hook(home: &Home, dialect: &str, event: &str) -> ExitCode {
    let fail_closed = std::env::var_os("HC_FAIL_CLOSED").is_some();
    let outcome = (|| -> Result<(String, String, i32)> {
        let d = bailiff::dialect::Dialect::parse(dialect).with_context(|| format!("unknown dialect {dialect:?}"))?;
        let kind = bailiff::dialect::EventKind::parse(event).with_context(|| format!("unsupported event {event:?}"))?;
        let mut raw = String::new();
        std::io::stdin().read_to_string(&mut raw)?;
        let payload: serde_json::Value =
            serde_json::from_str(&raw).with_context(|| format!("hook payload is not valid JSON ({} bytes)", raw.len()))?;
        anyhow::ensure!(payload.is_object(), "hook payload must be a JSON object");
        let input = bailiff::dialect::read(d, kind, &payload);
        let docket = home.docket(&input.cwd)?;
        let harness = home.identity(Role::Harness, "local")?;
        let env = bailiff::Env {
            docket: &docket,
            harness: &harness,
            state_dir: home.root().join("hooks"),
            sandbox: Sandbox::detect(home.root()),
            clock: &now,
        };
        let verdict = bailiff::handle(&env, d, &input)?;
        Ok(bailiff::render(d, kind, &verdict))
    })();
    match outcome {
        Ok((stdout, stderr, code)) => {
            print!("{stdout}");
            eprint!("{stderr}");
            ExitCode::from(u8::try_from(code).unwrap_or(1))
        }
        Err(e) => {
            eprintln!("[hc] hook error: {e:#}");
            if fail_closed { ExitCode::from(2) } else { ExitCode::SUCCESS }
        }
    }
}

fn cmd_assay(seed: u64, disable: &[String], work: Option<PathBuf>, json: bool) -> Result<ExitCode> {
    let disabled = disable.iter().filter(|s| !s.is_empty()).map(|s| parse_pass(s)).collect::<Result<BTreeSet<_>>>()?;
    let (_guard, dir) = assay::workdir(work)?;
    let report = assay::evaluate(&dir, seed, &disabled)?;
    if json {
        println!("{}", serde_json::to_string_pretty(&report)?);
    } else {
        for s in &report.sessions {
            let mark = match (s.kind.is_lie(), s.caught) {
                (true, true) => "caught ",
                (true, false) => "MISSED ",
                (false, false) => "clean  ",
                (false, true) => "FALSE+ ",
            };
            println!("{mark} {:<24} {}", s.id, s.refuting_rules.join(", "));
        }
        println!(
            "\nseed {}  disabled {:?}\nlies caught {}/{}  false refutations {}/{}  deterministic {}  facts below observed {}",
            report.seed,
            report.disabled,
            report.lies_caught,
            report.lies,
            report.false_refutations,
            report.honest,
            report.deterministic,
            report.facts_below_observed
        );
        println!("{}", if report.passes() { "ACCEPT" } else { "REJECT" });
    }
    Ok(if report.passes() { ExitCode::SUCCESS } else { ExitCode::FAILURE })
}

fn main() -> Result<ExitCode> {
    let cli = Cli::parse();
    let home = Home::locate()?;
    match cli.command {
        Command::Init => {
            for (role, name) in [(Role::Harness, "local"), (Role::Kernel, "local")] {
                let id = home.identity(role, name)?;
                println!("{} -> {}", id.principal(), id.key_path.display());
            }
            println!("hc home: {}", home.root().display());
            Ok(ExitCode::SUCCESS)
        }
        Command::Schema => {
            print!("{}", intake::CLAIM_SCHEMA);
            Ok(ExitCode::SUCCESS)
        }
        Command::Agents => {
            let registry = home.registry()?;
            for def in registry.iter() {
                let launch = if def.launch.is_some() { "launchable" } else { "detect-only" };
                println!("{:<14} {:<10} {:<11} {}", def.id, def.vendor, launch, def.executables.join(","));
            }
            match registry.detect() {
                Some(a) => println!("\ndetected: {} via {:?} (pid {:?})", a.agent_id, a.level, a.pid),
                None => println!("\ndetected: none"),
            }
            Ok(ExitCode::SUCCESS)
        }
        Command::Claim { file, repo, session, declared_as } => cmd_claim(&home, file.as_deref(), &repo, session, declared_as.as_deref()),
        Command::Brief { task, repo } => {
            let task = load_task(&task)?;
            let kernel = kernel_for(&home, &repo, &task, true)?;
            let findings = adjudicate(&home, &repo, &task, &kernel)?;
            print_findings(&findings);
            print!("{}", Brief::link(task, &findings)?.render());
            Ok(ExitCode::SUCCESS)
        }
        Command::Run { task, agent, repo, max_attempts, no_isolation } => cmd_run(&home, &task, &agent, &repo, max_attempts, !no_isolation),
        Command::Hook { dialect, event } => Ok(cmd_hook(&home, &dialect, &event)),
        Command::Mcp { repo } => {
            clerk::serve_stdio(clerk::Clerk::new(home, repo.canonicalize()?))?;
            Ok(ExitCode::SUCCESS)
        }
        Command::Conform { session, repo } => {
            let report = conform::align(&home.docket(&repo)?.records()?, &session);
            println!("{}", serde_json::to_string_pretty(&report)?);
            Ok(ExitCode::SUCCESS)
        }
        Command::Onboard { test_command, repo, agents, hooks, hc_bin } => {
            for dialect in hooks.unwrap_or_default() {
                let rel = courier::hooks::path(&dialect).with_context(|| format!("unknown harness {dialect:?}"))?;
                let path = repo.join(rel);
                let existing = std::fs::read_to_string(&path).ok().and_then(|t| serde_json::from_str(&t).ok()).unwrap_or_default();
                let merged = courier::hooks::merge(&dialect, existing, &hc_bin);
                std::fs::create_dir_all(path.parent().expect("config has a parent"))?;
                std::fs::write(&path, serde_json::to_string_pretty(&merged)? + "\n")?;
                println!("hooks   {}", path.display());
            }
            let registry = home.registry()?;
            let mut files = BTreeSet::new();
            for def in registry.iter().filter(|d| agents.is_empty() || agents.contains(&d.id)) {
                files.insert(def.instructions_file.clone());
            }
            for file in files {
                let path = repo.join(&file);
                let changed = courier::write_protocol_block(&path, &test_command)?;
                println!("{} {}", if changed { "wrote  " } else { "current" }, path.display());
            }
            Ok(ExitCode::SUCCESS)
        }
        Command::Assay { seed, disable, work, json } => cmd_assay(seed, &disable, work, json),
        Command::ProbeModel { ollama, models, repeats } => {
            let results = assay::probe::run(&ollama, &models, repeats)?;
            for r in &results {
                eprintln!(
                    "{:<24} {:<12} valid {}/{}  correct {}/{}  median {} ms",
                    r.model,
                    if r.constrained { "schema" } else { "prompt-only" },
                    r.valid,
                    r.trials,
                    r.semantic,
                    r.trials,
                    r.median_ms
                );
            }
            println!("{}", serde_json::to_string_pretty(&results)?);
            Ok(ExitCode::SUCCESS)
        }
        Command::ExportNotion { task, data_source, title_property, repo } => {
            let token = std::env::var("NOTION_TOKEN").context("NOTION_TOKEN is not set")?;
            let task = load_task(&task)?;
            let kernel = kernel_for(&home, &repo, &task, true)?;
            let findings = adjudicate(&home, &repo, &task, &kernel)?;
            let brief = Brief::link(task, &findings)?;
            let page = courier::notion::create_page(&token, &courier::notion::page_body(&data_source, &title_property, &brief))?;
            println!("{}", page.get("url").and_then(|u| u.as_str()).unwrap_or("created"));
            Ok(ExitCode::SUCCESS)
        }
    }
}
