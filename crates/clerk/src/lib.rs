//! clerk: takes filings from surfaces that have no hooks (desktop and web chat
//! apps, IDE agents, any MCP client). It is the attestation tier: the agent
//! calls these tools, and every answer comes from the same kernel and step
//! gates the hook tier uses, so the two tiers cannot drift apart.
//!
//! Tools: `schema`, `brief`, `plan`, `claim`, `verify`, `conform`.

use std::path::{Path, PathBuf};

use anyhow::Context as _;
use bailiff::dialect::{Dialect, EventKind, Input};
use bedrock::ir::Role;
use brief::{Brief, Task};
use cell::Sandbox;
use chambers::Home;
use crossexam::Kernel;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{CallToolResult, ContentBlock};
use rmcp::{ErrorData as McpError, ServiceExt, schemars, tool, tool_router};
use serde::Deserialize;
use serde_json::{Value, json};

fn now() -> String {
    jiff::Timestamp::now().strftime("%Y-%m-%dT%H:%M:%SZ").to_string()
}

fn text(t: impl Into<String>) -> CallToolResult {
    CallToolResult::success(vec![ContentBlock::text(t)])
}

fn fail(e: &anyhow::Error) -> McpError {
    McpError::internal_error(format!("{e:#}"), None)
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct RepoArgs {
    /// Repository root (default: the server's working directory).
    #[serde(default)]
    pub repo: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct PlanArgs {
    #[serde(default)]
    pub repo: Option<String>,
    /// A stable id for this working session.
    pub session: String,
    /// The full current plan as a Markdown checklist: `- [ ]` pending, `- [/]` in progress, `- [x]` done, `- [-]` dropped.
    pub markdown: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ClaimArgs {
    #[serde(default)]
    pub repo: Option<String>,
    pub session: String,
    /// The claim envelope as JSON text, matching the `schema` tool's output.
    pub claims: String,
    /// Your agent name, used only when the client process is not a registered agent.
    #[serde(default)]
    pub agent: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct SessionArgs {
    #[serde(default)]
    pub repo: Option<String>,
    pub session: String,
}

#[derive(Debug, Clone)]
pub struct Clerk {
    home: Home,
    default_repo: PathBuf,
}

impl Clerk {
    pub fn new(home: Home, default_repo: PathBuf) -> Self {
        Self { home, default_repo }
    }

    fn repo(&self, repo: Option<&str>) -> PathBuf {
        repo.map_or_else(|| self.default_repo.clone(), PathBuf::from)
    }

    fn task(repo: &Path) -> anyhow::Result<Task> {
        let path = repo.join(".handoff/task.json");
        let raw = std::fs::read_to_string(&path).with_context(|| format!("no task file at {}", path.display()))?;
        Ok(serde_json::from_str(&raw)?)
    }

    fn gate(&self, repo: &Path, session: &str, kind: EventKind, tool: Option<&str>, tool_input: Value) -> anyhow::Result<String> {
        self.with_gate(repo, session, kind, tool, tool_input, false)
    }

    fn with_gate(
        &self,
        repo: &Path,
        session: &str,
        kind: EventKind,
        tool: Option<&str>,
        tool_input: Value,
        checkpoint_only: bool,
    ) -> anyhow::Result<String> {
        let docket = self.home.docket(repo)?;
        let harness = self.home.identity(Role::Harness, "local")?;
        let env = bailiff::Env {
            docket: &docket,
            harness: &harness,
            state_dir: self.home.root().join("hooks"),
            sandbox: Sandbox::detect(self.home.root()),
            clock: &now,
        };
        let input = Input {
            kind,
            session: session.to_owned(),
            cwd: repo.to_path_buf(),
            tool: tool.map(str::to_owned),
            tool_input,
            tool_response: None,
            task_id: None,
            continuing: false,
            artifact_dir: None,
        };
        if checkpoint_only {
            bailiff::checkpoint_now(&env, Dialect::Mcp, &input)?;
            return Ok(String::new());
        }
        let verdict = bailiff::handle(&env, Dialect::Mcp, &input)?;
        Ok(bailiff::render(Dialect::Mcp, kind, &verdict).0)
    }

    fn brief_text(&self, repo: &Path) -> anyhow::Result<String> {
        let task = Self::task(repo)?;
        let docket = self.home.docket(repo)?;
        let kernel = Kernel::for_task(
            &task.test_command,
            task.allowed_paths.clone(),
            task.tests_writable,
            docket.git_dir(),
            repo.to_path_buf(),
            Sandbox::detect(self.home.root()),
        );
        let mut findings = kernel.adjudicate(&docket, &self.home.identity(Role::Kernel, "local")?, &now(), &task.id)?;
        kernel.observe_now(&mut findings)?;
        Ok(Brief::link(task, &findings)?.render())
    }

    fn file_claims(&self, repo: &Path, session: &str, raw: &str, agent: Option<&str>) -> anyhow::Result<String> {
        let envelope = match intake::compile(raw) {
            Ok(e) => e,
            Err(diags) => {
                let lines: Vec<String> = diags
                    .iter()
                    .map(|d| format!("{} at {}: {}", d.code, if d.path.is_empty() { "/" } else { &d.path }, d.message))
                    .collect();
                return Ok(format!("rejected; fix and resubmit:\n{}", lines.join("\n")));
            }
        };
        self.with_gate(repo, session, EventKind::PostToolUse, None, Value::Null, true)?;
        let attestation = self.home.registry()?.detect().unwrap_or_else(|| roster::declared(agent.unwrap_or("mcp-client")));
        let identity = self.home.identity(Role::Agent, &attestation.agent_id)?;
        let docket = self.home.docket(repo)?;
        exhibit::record_claims(&docket, &identity, &now(), &format!("mcp-{session}"), &attestation, &envelope)?;
        let mut out = format!("recorded {} claims from {} ({:?})\n", envelope.claims.len(), attestation.agent_id, attestation.level);
        out.push_str(&self.brief_text(repo).unwrap_or_default());
        Ok(out)
    }
}

#[tool_router(server_handler)]
impl Clerk {
    #[tool(description = "The JSON Schema that claims must match.")]
    async fn schema(&self) -> Result<CallToolResult, McpError> {
        Ok(text(intake::CLAIM_SCHEMA))
    }

    #[tool(
        description = "The current handoff brief for this repository's task: verified facts, refuted claims, unverified items and the current step. Read it before starting."
    )]
    async fn brief(&self, Parameters(args): Parameters<RepoArgs>) -> Result<CallToolResult, McpError> {
        Ok(text(self.brief_text(&self.repo(args.repo.as_deref())).map_err(|e| fail(&e))?))
    }

    #[tool(
        description = "Report your current plan whenever it changes. Steps you mark done are checked against the changes recorded while they were in progress."
    )]
    async fn plan(&self, Parameters(args): Parameters<PlanArgs>) -> Result<CallToolResult, McpError> {
        let repo = self.repo(args.repo.as_deref());
        let out = self
            .gate(&repo, &args.session, EventKind::PostToolUse, Some("hc_plan"), json!({"markdown": args.markdown}))
            .map_err(|e| fail(&e))?;
        Ok(text(out))
    }

    #[tool(
        description = "File typed claims about what you did, as JSON matching the `schema` tool. Returns the updated brief with verdicts."
    )]
    async fn claim(&self, Parameters(args): Parameters<ClaimArgs>) -> Result<CallToolResult, McpError> {
        let repo = self.repo(args.repo.as_deref());
        Ok(text(self.file_claims(&repo, &args.session, &args.claims, args.agent.as_deref()).map_err(|e| fail(&e))?))
    }

    #[tool(
        description = "Run the end-of-work checks: protected paths, cheat signatures, the task's acceptance command on a clean copy of the tree, and plan conformance. Call before saying you are done."
    )]
    async fn verify(&self, Parameters(args): Parameters<SessionArgs>) -> Result<CallToolResult, McpError> {
        let repo = self.repo(args.repo.as_deref());
        Ok(text(self.gate(&repo, &args.session, EventKind::Stop, None, Value::Null).map_err(|e| fail(&e))?))
    }

    #[tool(
        description = "Plan conformance for a session: which steps are backed by recorded changes, which are not, and work done outside any step."
    )]
    async fn conform(&self, Parameters(args): Parameters<SessionArgs>) -> Result<CallToolResult, McpError> {
        let repo = self.repo(args.repo.as_deref());
        let records = self.home.docket(&repo).and_then(|d| Ok(d.records()?)).map_err(|e| fail(&e))?;
        let report = conform::align(&records, &format!("mcp-{}", args.session));
        Ok(text(serde_json::to_string_pretty(&report).unwrap_or_default()))
    }
}

/// Serve over stdio until the client disconnects.
pub fn serve_stdio(clerk: Clerk) -> anyhow::Result<()> {
    let runtime = tokio::runtime::Builder::new_current_thread().enable_all().build()?;
    runtime.block_on(async move {
        let service = clerk.serve(rmcp::transport::stdio()).await?;
        service.waiting().await?;
        Ok(())
    })
}
