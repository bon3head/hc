<p align="center"><img src="assets/mark.svg" width="88" alt=""></p>

<h1 align="center">hc · the handoff compiler</h1>

<p align="center">When one coding agent hands work to the next, hc tells the next one what still holds,<br>for the exact code in front of it, and why.</p>

Agents say "done" when they are not, edit the tests that are supposed to check them, and hand over summaries nobody verified. The next agent then builds on whatever it was told. hc sits in the harnesses you already use (Claude Code, Codex, Antigravity, any MCP client), records what actually happened in a signed ledger the agent cannot reach, checks every claim deterministically, and compiles the handoff from the facts that held up. No LLM decides a verdict.

## What the next agent receives

One agent fixes `add()` and claims the suite passes. It then keeps editing, and on that second edit it breaks `add()`. These are the lines the next agent receives at each point, taken verbatim from `scripts/exhibit-stale.sh` (a release-gate check, so they cannot drift from the code):

<!-- exhibit:begin -->
**1. After the fix (tree c39331d126c4).** The claim was re-run outside the agent's session and holds:
```
- [reproduced] c1 tests_pass: `python3 -m unittest discover -s tests -t .` passes on tree c39331d126c4 (claim docket#3; checks passed: command, reexec, tamper)
```
**2. After another edit (tree 3b01a194da46).** The result is still true of the old tree, so it is kept, but it no longer describes the code:
```
- c1: `python3 -m unittest discover -s tests -t .` passes on tree c39331d126c4; the session's tree is now 3b01a194da46
```
**3. The agent claims the suite passes on the new tree, and the verifier cannot start `python3`.** Missing evidence is not a failure, and it is not a pass either:
```
- c2 tests_pass: basis quoted as data: "ran the suite" (not settled: 'python3' could not start: No such file or directory (os error 2))
```
**4. The verifier can run it.** The claim is false:
```
- c2 tests_pass: reexec-disagrees (exit codes [1, 1]) (claim docket#7)
```
<!-- exhibit:end -->

So the answer to "can I rely on this for the code in front of me?" is in the brief itself. Facts (1) name their check and tree. Stale results (2) say what changed. Unsettled claims (3) say why. Refutations (4) give the observed outcome. The evidence ends at the brief: whether a receiving agent behaves better with it is a separate, live measurement that has not been run yet.

## Quickstart

```sh
cargo install --path crates/hc
cd your-repo
hc init "Fix the date parser"
```

That's the whole setup. `hc init` finds your test command, installs hooks for the agents on your machine and adds the hc protocol to `AGENTS.md`. If something can't work on that machine (git too old, no OpenSSH), it stops and prints the fix. Then work as usual. `hc doctor` checks the machine and the repository, and every problem it finds comes with the command that fixes it.

**Other MCP clients** (no hooks needed):

```json
{ "mcpServers": { "hc": { "command": "hc", "args": ["mcp"] } } }
```

**Requirements:** Linux (full) or macOS (agents identified by name, no sandbox), git 2.34+, OpenSSH. bubblewrap is optional and isolates `hc run`.

| Command | Does |
|---|---|
| `hc init ["objective"]` | Set up the repository; re-run to change the objective |
| `hc doctor` | Check this machine and repository, with a fix for each problem (`-v`, `--json`) |
| `hc brief` | The handoff the next agent receives |
| `hc mcp` | MCP server: `brief`, `plan`, `claim`, `verify`, `conform`, `schema` |
| `hc run --agent codex` | Launch an agent headless, verify, retry from a fresh brief |
| `hc claim` | An agent files typed claims; bound to the session its hooks recorded |
| `hc conform --session <id>` | Plan-versus-log conformance for one session |
| `hc assay --seed 7` | Acceptance run over the seeded corpus |

## How a claim becomes a fact

1. **Claim.** The agent files typed JSON (`hc schema`). Intake rejects anything that could change meaning on the way to the brief: comments in commands, markup in ids, `..` in paths, a `command_ran` with no exit code.
2. **Observation.** The hooks snapshot the working tree into hc's ledger after every edit, at every plan step and before every claim. That includes `.git/config` and `.git/hooks`, which git itself never tracks. Every entry is an SSH-signed git commit in a repository the agent cannot see.
3. **Verdict.** The kernel checks the claim against those observations:

   | Check | Refutes a claim when |
   |---|---|
   | command | a test claim names a command other than the task's test command |
   | tamper | the session wrote tests, CI, hook, agent or git configuration |
   | scope | writes fall outside allowed paths, or file claims disagree with observed writes |
   | signature | added lines exit 0, skip tests, make `__eq__` always true, patch clocks, or add vacuous tests |
   | reexec | the test command fails, twice, on a clean copy of the claimed commit or tree |
   | falsify | when tests may be edited, the new tests also pass on the unfixed code (fail-to-pass, as in SWT-Bench) |

   A check that cannot get evidence (runner missing, commit unknown, no recorded session) leaves the claim **unresolved**: never refuted, never promoted. A settled verdict is never recomputed, so a fact stays true of the tree it was checked on.
4. **Brief.** Facts carry their tree, their claim's ledger entry and the checks that passed. Once the tree moves on they become **stale**. Agent-written text is quoted as data, one line, never as instructions.

At step boundaries the same checks run through the harness's own hooks. The gating hooks block, and if hc itself fails they block too: "could not check" never reads as "allowed". The same failure twice in a row ends retries with a request for the root cause.

## Evidence

`scripts/gate.sh` is the release gate, and every number below comes from it:

- **Seeded corpus, three seeds:** 17 honest and 17 lying sessions, run against real git repositories and Python test suites. All 17 lies are caught. All 32 facts the ground-truth manifest expects are delivered, with no claim missing, extra, misplaced or stale. Disabling any single check lowers recall. With re-execution disabled the run fails for exactly the expected reason.
- **Attacks:** every finding of the independent audit (`docs/audit-1.md`) is replayed through the real entry points by `scripts/attacks.sh`.
- **Robustness and speed:** malformed hook payloads block the gate visibly. Hook latency p95 is about 10 ms before a write and about 50 ms after.
- **Setup:** `scripts/matrix.sh` runs on clean Ubuntu 20.04, 22.04 and 24.04 and under eleven broken-machine faults; every case either works or stops with the fix. `scripts/mutants.py` plants trust-critical bugs and requires the gate's own checks to catch each one.

## Status

Ready for a live pilot, not published, and not yet shown to improve a live handoff. Hook payloads follow each vendor's documented shapes, but no live Claude Code, Codex or Antigravity session has been recorded.

**Prior work.** Two projects are close:

- [dunnit](https://pypi.org/project/dunnit/) already re-runs proof commands against a `dod.yaml` contract and inspects diffs for test-gaming.
- [Soei](https://arxiv.org/abs/2610.39468) (preprint) keeps a hash-chained ledger that separates completion claims from proof.

Re-running checks and keeping a ledger are therefore not new. What hc adds:

- verdicts bound to the tree they describe, which go stale when the code moves;
- missing evidence kept apart from refutation;
- the agent's own native plans checked step by step;
- the result compiled into the next agent's handoff.

[HSC-Bench](https://arxiv.org/abs/2610.36523) (preprint) is the closest measurement of handoffs. Structured handoff state improved evidence recovery for every receiver it tested, but it did not isolate independent verification. Whether hc's brief changes what a receiver does is the open experiment.

## How it got here

hc merges three earlier projects: crossexam (claim checking), assay (agent evals) and context heartbeats (drift reminders). The heartbeats were dropped after research showed restatement saturates quickly. In their place came feedback triggered only by failures and delivered through native hooks. A Python plan became a Rust workspace with a signed git ledger. Native plans and hooks replaced an API proxy. Two independent audits and the corpora reviews drove the current shape:

- facts bound to trees;
- "unresolved" distinguished from "refuted";
- no checkout, hook, filter or fsmonitor ever run inside the agent's repository;
- an acceptance test that can prove the verifier wrong.

The full record is in `docs/experiments.md` (what was proven and what was killed), `docs/steering.md` (every directive and its status) and `docs/decisions.md` (the sources behind each design choice).

## Layout

Seventeen crates, each doing one thing (`integration/skills/developing-handoff-compiler/references/architecture.md`). Research: `docs/research-report.md`, `docs/research-native-plans.md`, `docs/claim-battery.md`. Audits: `docs/audit-1.md`, `docs/audit-resolution.md`. Agent skill pack: `integration/skills/`. Mark: `assets/mark.svg` (terminal version in `assets/mark.txt`).
