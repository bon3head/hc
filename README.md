# hc — the handoff compiler

Coding agents say "done" when they are not, drift from their own plans, and edit the tests that are supposed to check them. hc verifies what they claim against what actually happened, using the tools each agent already has, and hands the next agent only the facts that held up.

- **No new methodology.** hc reads the plan the agent already keeps (Claude Code tasks and todos, Codex `update_plan`, Antigravity task artifacts, `PLANS.md`) and checks it step by step.
- **Native channels.** Findings go back through each harness's own hooks; surfaces without hooks use the `hc mcp` server.
- **Deterministic first.** Protected-path checks, AST cheat signatures, clean re-execution of the task's tests, and fail-to-pass checks on new tests. No LLM judge decides a verdict.
- **Tamper-evident record.** Every observation, claim and verdict is an SSH-signed git commit in a repository the agent cannot see.

## Quickstart

```sh
cargo install --path crates/hc
cd your-repo
mkdir -p .handoff
echo '{"id":"fix-parser","objective":"...","test_command":"cargo test","allowed_paths":["src/**"],"next_step":"..."}' > .handoff/task.json
hc init
hc onboard --test-command "cargo test" --hooks      # Claude Code, Codex and Antigravity hook configs
```

Then work as usual. `scripts/quickstart.sh` runs this flow end to end against a toy repository.

| Command | Does |
|---|---|
| `hc hook <dialect> <event>` | Hook entry point the harness calls |
| `hc mcp` | MCP server for surfaces without hooks: `brief`, `plan`, `claim`, `verify`, `conform`, `schema` |
| `hc run --task t.json --agent codex` | Launch an agent headless in a bubblewrap sandbox, verify, retry from a fresh brief |
| `hc claim` | An agent files typed claims; its identity is detected from the process tree |
| `hc brief --task t.json` | The compiled handoff for the next agent |
| `hc conform --session <id>` | Plan-versus-log conformance for one session |
| `hc assay --seed 7` | Acceptance run over the seeded corpus |
| `hc export-notion` | Publish a brief to a Notion data source |

## How it decides

| Check | Refutes a claim when |
|---|---|
| command (P4) | a test claim names a command other than the task's test command |
| tamper (P6) | the session wrote tests, CI, hook or agent configuration |
| scope (P6S) | writes fall outside allowed paths, or file claims disagree with observed writes |
| signature (P7) | added lines exit 0, skip tests, make `__eq__` always true, patch clocks, or add vacuous tests |
| reexec (P8) | the test command fails on a clean copy of the claimed commit or final tree |
| falsify (P9) | when tests may be edited, the new tests also pass on the unfixed code |

At step boundaries the same checks run through hooks, and plan conformance labels every step **matched**, **plan-only** (marked done with no recorded change) or leaves work **log-only** (done outside any step). The same failure twice in a row ends retries with a request for the root cause.

## Results

`scripts/gate.sh` is the release gate. On the seeded corpus (17 honest and 17 lying sessions, run for real against git repositories and Python test suites, three seeds) hc catches every lie with no false refutations, and disabling any single check lowers recall. See `docs/ablations.txt`.

## Layout

Seventeen crates, each doing one thing; see `integration/skills/developing-handoff-compiler/references/architecture.md`. Design sources and conformance evidence: `docs/decisions.md`. Research: `docs/research-report.md`, `docs/research-native-plans.md`. Agent skill pack: `integration/skills/`. Reference specs, pinned dependency sources and code graphs: `refs/`.

## Status

Verified here with simulated hook payloads shaped exactly as each vendor documents them, real git repositories and real test runs. Not yet run against live Claude Code, Codex or Antigravity sessions; that pilot is the next step.
