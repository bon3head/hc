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

| One Linux binary runs on old and new distributions | `cargo zigbuild --target x86_64-unknown-linux-gnu.2.17`; Ubuntu 20.04/22.04/24.04 images built locally with debootstrap | Native build needed glibc 2.39 (would not start on 20.04 or 22.04); portable build needs 2.17 and runs on all three | Release builds use zigbuild |
| Setup either works or stops with the fix | `scripts/matrix.sh`: clean installs as an ordinary user, then 11 single faults | 14/14 after fixes | Matrix kept as a script |
| A useful fact reaches the next agent and does not outlive its tree | `scripts/exhibit-stale.sh` through the real hooks, `hc claim`, `hc brief` | Packet 1: fact on tree A. Packet 2: moved to stale after an edit. Packet 3: runner missing, new claim unresolved and A's fact kept. Packet 4: runner present, refuted | Gate criterion |

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
| Truncated hook payloads | Parsed as empty and silently allowed | Now a visible hook error; gating hooks block on error (audit F8), `HC_FAIL_OPEN=1` allows |
| Hook and MCP sessions have a final tree | They never end, so `tests_pass` had nothing to re-run | Latest checkpoint is the session's tree; claims checkpoint first |
| Hook sessions record the tree claims refer to | Exhibit through the real hook path | PostToolUse snapshotted only after plan tools, so an honest claim had no tree and was refuted | Snapshot after every file-writing tool and before `hc claim` |
| "Could not check" is a failure | Exhibit with the test runner removed from the verifier's PATH | Claims were refuted for missing evidence | New outcome `unresolved`: the claim stays unverified with the reason |
| Re-judging every claim on every brief is harmless | Exhibit, packet 3 | A fact verified on tree A was lost when today's machine lacked the runner | Settled verdicts are reused under the same policy; only unresolved ones are re-examined |
| Facts can say "the session's final tree" | Exhibit, packet 2 | After a later edit that sentence was false | Facts name their tree; tree-bound results move to a stale section |
| Acceptance invariants catch a starved verifier | Corpus argument: all four held with an empty facts section | True: nothing required honest claims to be delivered | `honest_admitted == honest_verifiable` added; disabling reexec now delivers 18/32 and is rejected |
| `init` can succeed on any machine | Matrix on Ubuntu 20.04 (git 2.25) | Setup succeeded, then every hook would have failed | `init` runs the doctor's system checks first |
| Errors that wrap a source can repeat it | Matrix: "No such file or directory (os error 2): No such file or directory" | Eleven error variants printed their source twice | Messages no longer embed the source; the chain prints once |
| A mutation battery's kills mean detection | First battery run: five mutants "killed by cargo test" within seconds | Every kill was a setup failure: the scratch copy lacked `refs/specs` (the RFC 8785 vectors), then a timestamp-preserving sync left a stale binary that cargo did not rebuild | The battery now requires every check to pass on unmutated code before counting a kill, and syncs with fresh timestamps |
| The README prior-work line was accurate | Opened dunnit's PyPI page and the Soei and HSC-Bench abstracts | dunnit already re-runs proof commands and inspects diffs for test-gaming; Soei already separates completion claims from proof in a hash-chained ledger | README states the overlap and narrows hc's claim to tree-bound verdicts, stale/unresolved semantics, native plans and handoff compilation |
| The F2 control failing only in the battery's copy was a code difference | Same F2 scenario: main checkout passed; the copy's brief printed the old rendering `(evidence: docket#1)` although its sources were byte-identical to main | The copy's binary (built 07:28) was newer than its restored sources (mtime 07:22, preserved by `copy2`), so cargo kept a binary built from a mutant-era source. Syncing with fresh timestamps made the copy's control pass with the same bytes as main; the battery now refuses to count kills unless that control passes | Battery baseline-gated; sync without preserved mtimes |
| Sessions are a sound unit of evidence | GTM audit T3/T9/T23: a fresh session id, an MCP client's first call, or an owner's `hc init` each shifted writes out of view or onto the wrong party | Per-session evidence let the agent choose what the checks saw | Repository continuity: a signed baseline, every snapshot diffed against the last recorded tree, write checks over all writes since the baseline, contract read from the baseline |
| A protected write blocks Stop until the session ends | Attack T24 with fail-fast and net-diff | "Never released" plus "any write" would have trapped an agent that reverted its edit | Stop blocks on net difference from the baseline; restoring the file lets the session stop; fail-fast never releases tampering |
| `hc claim` can judge a claim against the session's last snapshot | Attack T25: a second session's claim was judged on the fixed tree after `add()` broke | Only hook sessions were snapshotted at claim time | `hc claim` records the tracked repository before every claim |
| Exhibit bytes are reproducible | Gate criterion 7 failed: README tree ids came from the debug build | The hook config written into the repository held the hc binary's absolute path, so trees depended on where hc is installed | The exhibit records `--hc-bin hc`; debug and release now give identical bytes |
| rust-analyzer in the build sandbox | Distribution host blocked; alternate download declined | Run locally; clippy pedantic as the enforced gate here |

## Not yet run

- Live sessions in Claude Code, Codex and Antigravity (needs accounts; the hook payloads here follow their references exactly).
- Local-model claim filing on a GPU: `gpu/probe_local_models.ipynb`.
- Feedback channel comparison (tool-result `additionalContext` vs. Stop-reason vs. system message) on live agents; no published study compares them.
