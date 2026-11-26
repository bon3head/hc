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
hc init "Fix the date parser"
```

That's the whole setup. `hc init` finds your test command, installs hooks for the agents on your machine (Claude Code, Codex, Antigravity) and adds the hc protocol to `AGENTS.md`. Then work as usual. Re-running it is safe, and `hc doctor` tells you what's wrong if something isn't working.

**Other MCP clients** (no hooks needed):

```json
{ "mcpServers": { "hc": { "command": "hc", "args": ["mcp"] } } }
```

**Requirements:** Linux or macOS, git 2.34+, OpenSSH. bubblewrap is optional and isolates `hc run`. `hc init` stops with the fix if anything is missing.

| Command | Does |
|---|---|
| `hc init ["objective"]` | Set up the repository; re-run to change the objective |
| `hc doctor` | Check this machine and repository; every problem comes with its fix (`-v`, `--json`) |
| `hc brief` | The handoff the next agent receives |
| `hc mcp` | MCP server: `brief`, `plan`, `claim`, `verify`, `conform`, `schema` |
| `hc run --task t.json --agent codex` | Launch an agent headless, verify, retry from a fresh brief |
| `hc claim` | An agent files typed claims; its identity comes from the process tree |
| `hc conform --session <id>` | Plan-versus-log conformance for one session |
| `hc assay --seed 7` | Acceptance run over the seeded corpus |

## How it decides

| Check | Refutes a claim when |
|---|---|
| command (P4) | a test claim names a command other than the task's test command |
| tamper (P6) | the session wrote tests, CI, hook or agent configuration |
| scope (P6S) | writes fall outside allowed paths, or file claims disagree with observed writes |
| signature (P7) | added lines exit 0, skip tests, make `__eq__` always true, patch clocks, or add vacuous tests |
| reexec (P8) | the test command fails on a clean copy of the claimed commit or final tree |
| falsify (P9) | when tests may be edited, the new tests also pass on the unfixed code |

Every fact names the tree it was checked on. If the code changes afterwards, the fact moves to **stale** instead of being carried forward, and a check that could not run (no snapshot, runner missing) leaves the claim **unresolved** rather than refuted: missing evidence is never reported as a failure. `scripts/exhibit-stale.sh` shows the packets before and after an edit.

At step boundaries the same checks run through hooks, and plan conformance labels every step **matched**, **plan-only** (marked done with no recorded change) or leaves work **log-only** (done outside any step). The same failure twice in a row ends retries with a request for the root cause.

## Results

`scripts/gate.sh` is the release gate. On the seeded corpus (17 honest and 17 lying sessions, run for real against git repositories and Python test suites, three seeds) hc catches every lie with no false refutations and delivers every verifiable honest claim (32/32) as a fact, so an empty brief cannot pass. Disabling any single check lowers recall. See `docs/ablations.txt`.

`scripts/matrix.sh` runs setup on clean Ubuntu 20.04, 22.04 and 24.04 images as an ordinary user, then breaks one thing at a time (git or ssh-keygen missing, sandbox blocked, unwritable home, hooks pointing at a missing binary, bad task file, missing test runner, a forged docket, a UID with no passwd entry). Each case must either work or stop with the right fix. Release binaries are linked against glibc 2.17 with `cargo zigbuild`, so one Linux build runs on all of them.

## Layout

Seventeen crates, each doing one thing; see `integration/skills/developing-handoff-compiler/references/architecture.md`. Design sources and conformance evidence: `docs/decisions.md`. Research: `docs/research-report.md`, `docs/research-native-plans.md`. Agent skill pack: `integration/skills/`. Reference specs, pinned dependency sources and code graphs: `refs/`.

## Status

Verified here with simulated hook payloads shaped exactly as each vendor documents them, real git repositories and real test runs. Not yet run against live Claude Code, Codex or Antigravity sessions; that pilot is the next step.
