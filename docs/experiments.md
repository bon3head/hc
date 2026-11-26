# Experiments log

Each entry: the hypothesis, how it was tested, the result, and what changed. Kills are recorded as carefully as wins.

## Proven

| Hypothesis | Test | Result | Consequence |
|---|---|---|---|
| Deterministic checks catch the documented cheat classes | Seeded corpus, 17 lies + 17 honest sessions, real git repos and test runs, seeds 7, 11, 23 | 17/17 caught, 0/17 false refutations on every seed | Acceptance gate |
| Every check earns its place | Disable one pass at a time | Recall drops for each: reexec 12, tamper 13, scope 14, signature 15, command 16, falsify 16 (of 17) | No pass removed |
| Runs are reproducible | Two independent generations with shared keys | Identical docket heads and brief digests | Determinism criterion in the gate |
| Native plans can be checked step by step without a new methodology | Ten hook sessions with payloads shaped as Claude Code, Codex and Antigravity document them | Honest plan fully conformant (1000‰); false completion observed; shell-made test edit blocked at the step; `TaskCompleted` veto works | `agenda`, `conform`, `bailiff` |
| Hooks are fast enough to stay on | Release binary, real repo, 30 calls each | `PreToolUse` p95 8 ms; plan-update `PostToolUse` p95 54 ms | Hooks on by default |
| Surfaces without hooks can use the same gates | `hc mcp` driven over stdio JSON-RPC (protocol 2025-11-25) | Handshake, 6 tools; verify blocks then clears; claims reproduced and observed; malformed claims rejected | `clerk` |
| Agent identity can come from the process tree | `hc claim` inside this build environment | Detected `claude-code` by ancestry | `roster` |
| Fail-to-pass separates real tests from rigged ones | Writable-tests tasks: honest regression test vs. rewritten expectations | Honest passes; rigged and vacuous suites refuted (`tests-do-not-discriminate`) | P9 |

## Killed or corrected

| Idea or assumption | What happened | Decision |
|---|---|---|
| Periodic "heartbeat" reminders | Research: gains saturate after one restatement; labs already inject reminders | Replaced by failure-triggered observations through native hooks |
| API proxy for context injection | Cannot reach subscription CLIs or hosted agents; wire formats are vendor-owned | Dropped for native hooks |
| PATH shims to observe agent test runs | The agent's own runs are not evidence | Kernel re-executes instead |
| Per-entry DSSE signatures | No second-machine verifier yet; git SSH signing covers authorship | Dropped (YAGNI) |
| Custom canonical JSON | `serde_jcs` passes all RFC 8785 vectors | Dependency instead |
| "2^53 is outside the safe range" (my test) | Property test showed RFC 8785 accepts any integer with an exact double | Test corrected; front end stays stricter (I-JSON) |
| Git verification "just works" | A sandbox-injected global signing program broke signing | Every git call made hermetic |
| Identical step texts | Two "Run tests" steps got the same key | Keys now carry an occurrence index |
| Plan-file writes as evidence | Writing `task.md` made an empty step look done | Plan-file writes excluded from attribution |
| Escalation as a permanent pass | After one escalation, a new failure was let through | Escalation scoped to one failure fingerprint |
| Truncated hook payloads | Parsed as empty and silently allowed | Now a visible hook error; `HC_FAIL_CLOSED=1` blocks |
| Hook and MCP sessions have a final tree | They never end, so `tests_pass` had nothing to re-run | Latest checkpoint is the session's tree; claims checkpoint first |
| rust-analyzer in the build sandbox | Distribution host blocked; alternate download declined | Run locally; clippy pedantic as the enforced gate here |

## Not yet run

- Live sessions in Claude Code, Codex and Antigravity (needs accounts; the hook payloads here follow their references exactly).
- Local-model claim filing on a GPU: `gpu/probe_local_models.ipynb`.
- Feedback channel comparison (tool-result `additionalContext` vs. Stop-reason vs. system message) on live agents; no published study compares them.
