<p align="center"><img src="assets/mark.svg" width="88" alt=""></p>

<h1 align="center">hc · the handoff compiler</h1>

<p align="center">When one coding agent hands work to the next, hc tells the next one what still holds,<br>for the exact code in front of it, and why.</p>

Agents say "done" when they are not, edit the tests that are supposed to check them, and hand over summaries nobody verified. The next agent then builds on whatever it was told. hc sits in the harnesses you already use (Claude Code, Codex, Antigravity, any MCP client), records what actually happened in a signed ledger outside the repository, checks every claim deterministically, and compiles the handoff from the facts that held up. No LLM decides a verdict.

## What the next agent receives

One agent fixes `add()` and claims the suite passes. It then keeps editing, and on that second edit it breaks `add()`. These are the lines the next agent receives at each point, taken verbatim from `scripts/exhibit-stale.sh` (a release-gate check, so they cannot drift from the code):

<!-- exhibit:begin -->
**1. After the fix.** The hooks recorded the fixed code as tree da19c5cdb31f (ledger entry 3). The claim (entry 4) was re-run on that tree outside the agent's session, and the kernel's signed verdict (entry 5) says it holds:
```
- [reproduced] c1 tests_pass: `python3 -m unittest discover -s tests -t .` passes on tree da19c5cdb31f (snapshot docket#3) (claim docket#4; verdict docket#5; checks passed: command, reexec, tamper)
```
**2. After another edit.** The result is still true of tree da19c5cdb31f, so it is kept, but it no longer describes the code:
```
- c1: `python3 -m unittest discover -s tests -t .` passes on tree da19c5cdb31f (snapshot docket#3); the code is now at tree 76a562749e27
- src/calc.py modified (1 line added or changed): written in session claude-x
```
**3. The agent claims the suite passes on the new tree, and the verifier cannot start `python3`.** Missing evidence is neither a failure nor a pass:
```
- c2 tests_pass: basis quoted as data: "ran the suite" (not settled on tree 76a562749e27 (snapshot docket#7): 'python3' could not start: No such file or directory (os error 2))
```
**4. The verifier can run it.** The same claim on the same tree is false:
```
- c2 tests_pass: reexec-disagrees (exit codes [1, 1]) on tree 76a562749e27 (snapshot docket#7) (claim docket#8; verdict docket#10)
```
<!-- exhibit:end -->

So the answer to "can I rely on this for the code in front of me?" is in the brief itself. Every line names the tree it is about and the ledger entry that recorded that tree, and `hc show 3 4 5` prints those entries: the harness's snapshot, the agent's claim and the kernel's verdict, each with its signer verified. Facts (1) say which checks passed. Stale results (2) say the code moved, and the section under them lists which paths changed and whether a session wrote them or they changed while no session was running (paths only: file contents never enter a brief). Unsettled claims (3) say why. Refutations (4) give the observed outcome. The evidence ends at the brief: whether a receiving agent behaves better with it is a separate, live measurement that has not been run yet.

## Quickstart

```sh
cargo install --locked --path crates/hc
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
| `hc show <entry>…` | The ledger entries a brief cites, with their verified signers |
| `hc mcp` | MCP server: `brief`, `plan`, `claim`, `verify`, `conform`, `schema` |
| `hc run --agent codex` | Launch an agent headless, verify, retry from a fresh brief |
| `hc claim` | An agent files typed claims; bound to the session its hooks recorded |
| `hc conform --session <id>` | Plan-versus-log conformance for one session |
| `hc assay --seed 7` | Acceptance run over the seeded corpus |

## How a claim becomes a fact

1. **Claim.** The agent files typed JSON (`hc schema`). Intake rejects anything that could change meaning on the way to the brief: comments in commands, markup in ids, `..` in paths, a `command_ran` with no exit code.
2. **Observation.** The hooks snapshot the working tree into hc's ledger after every edit, at every plan step and before every claim. That includes `.git/config` and `.git/hooks`, which git itself never tracks. A change made while no session was watching is attributed to the next one, because every snapshot is compared with the repository's last recorded tree. Every entry is an SSH-signed git commit in hc's own repository under `~/.hc`.
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

`hc init` records the starting point: the accepted code and the task contract. The kernel and the hooks follow that recorded contract. If the repository's `.handoff/task.json` is edited, the brief says it differs and ignores it. Re-running `hc init` is how the owner changes the contract.

At step boundaries the same checks run through the harness's own hooks. The gating hooks block, and if hc itself fails they block too: "could not check" never reads as "allowed". The same failure twice in a row ends retries with a request for the root cause.

## Trust model

hc catches the honest mistakes and the known cheating patterns: edited tests, fake passes, misreported files, laundering through new sessions, stale results, injected text. Each one is replayed by `scripts/attacks.sh`. What it can hold against depends on how the agent runs:

| Mode | The agent can | So the ledger is |
|---|---|---|
| Hooks and MCP (`hc init`) | run anything as your user, including reading `~/.hc` | evidence against mistakes and cheating in the repository, but not against an agent that deliberately attacks hc's own files or keys |
| `hc run` (bubblewrap) | see an empty `~/.hc`; write only the repository and its outbox | out of the agent's reach: its keys and records are not visible to it |

No check run as the same OS user can keep a determined same-user process away from that user's files. The brief also says who recorded the contract: if `hc init` was run from inside an agent's session, process ancestry detects it and the brief names the agent.

## Evidence

There are three kinds of evidence, and they answer different questions.

**Fixture behaviour: does hc give the right verdicts?** `scripts/gate.sh` is the release gate, and these numbers come from it:

- **Seeded corpus, three seeds.** 17 honest and 19 lying sessions run against real git repositories and Python test suites. All 19 lies are caught. All 32 facts that a ground-truth manifest, written from what each scenario does, expects are delivered, with no claim missing, extra, misplaced or stale. Disabling any single check lowers recall. With re-execution disabled, the run fails for exactly the expected reason: 22 test claims are left unverified.
- **Attacks.** `scripts/attacks.sh` replays every confirmed finding of two independent audits (`docs/audit-1.md`, `docs/audit-gtm.md`) through the real entry points: 62 cases. The gate derives the finding list from the audit reports, so a finding cannot be closed by assertion.
- **Robustness and speed.** Malformed hook payloads block the gate visibly. Hook latency p95 is 8 ms before a write, and 21 ms after an edit (48 ms for a plan update).
- **Setup.** `scripts/matrix.sh` runs on clean Ubuntu 20.04, 22.04 and 24.04 and under eleven broken-machine faults. Every case either works or stops with the fix.

**Mutation sensitivity: would the gate notice if hc were wrong?** `scripts/mutants.py` plants 31 plausible bugs in trust-critical code. A control run must pass every check first, and each mutant counts as caught only if the check expected to see it fails. All 31 are caught. The first full run caught 28: the 2 that survived each exposed a missing replay, and both were added (`docs/mutants.txt`).

**A live receiving agent: does the brief change what the next agent does?** Not measured yet.

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
