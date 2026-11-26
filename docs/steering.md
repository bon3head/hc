# Steering ledger

Every directive given during the build, its status, and the evidence. Each one is treated as standing; a later directive refines an earlier one, it does not replace it.

| # | Directive | Status | Evidence or gap |
|---|---|---|---|
| 1 | Not a Python slop stack; a real implementation | Done | Rust workspace, 15 single-purpose crates, clippy pedantic `-D warnings` |
| 2 | Re-research with authoritative docs; pack exemplar Rust with repomix and graphify | Done | `refs/specs/`, `refs/packs/` (pinned to `Cargo.lock`), `refs/graphs/` (5 graphs incl. this workspace) |
| 3 | Use an LSP | Partial | rust-analyzer cannot be installed in the build sandbox (distribution host blocked; alternate download declined). Gap: run `rust-analyzer diagnostics .` locally; diagnostics as step evidence is item 19 |
| 4 | Custom implementations need supporting resources; no comments defending poor architecture | Done, re-checked | `docs/decisions.md` (rewritten for the git-backed design); no `SHORTCUT`/`TODO`/`HACK` comments in `crates/` |
| 5 | Prevent signature forgery without reinventing the wheel; YAGNI | Done | Isolation (bubblewrap hides the hc home) plus git's native SSH commit signing; per-entry DSSE killed as unneeded |
| 6 | Extensible agent access; keys generated per detected agent | Done | `roster` (agents as data, process-ancestry detection), `seal` (ed25519 key per principal on first use) |
| 7 | Use git and similar existing tools | Done | Snapshots as tree objects, ledger as signed commit chain, worktrees for re-execution, hooks/notes documented |
| 8 | Refs, LSP, graphs actually used | Done except LSP | Graph and pack queries drove the tree-sitter and globset code; see item 3 |
| 9 | Weight attention on critical code; use auditors; Anthropic patterns | In progress | Independent auditor on trust-critical crates running; a second audit of `agenda`/`conform`/`bailiff` follows |
| 10 | Memorable names; Unix "do one thing"; separation of concerns | Done | Crate table in `integration/skills/developing-handoff-compiler/references/architecture.md` |
| 11 | Repo skill pack, progressive disclosure, Agent Skills spec + Anthropic practices | Done, needs refresh | Conformance test `crates/hc/tests/skill_pack.rs`; refresh for hooks, fail-fast text and P9 is item 16 |
| 12 | Integrate with native plans/todos and hooks; do not impose a methodology; research every harness | Done (CLI tier) | `agenda` reads 7 native plan shapes; `bailiff` speaks Claude/Codex and Antigravity hook dialects; research report on native plan surfaces |
| 13 | Work on any surface, including web UIs | Done (stdio MCP) | `hc mcp` (`clerk`): the same gates over MCP for surfaces without hooks; remote transport not built |
| 14 | Reset context; separate generation from verification; fail-fast after the same failure twice | Done in code, skill text pending | `hc run` restarts from a fresh brief; kernel is separate code and identity; `bailiff` fingerprint escalation (scoped per failure). The user's stop rule goes into the skills verbatim: item 16 |
| 15 | Pasted research corpora are information, not new directives; the main plan stays the task | Applied | Used only where they closed a concrete gap: external-observation framing, ExecPlan reader, patience-style step identity, fault-injection fixes, fail-to-pass oracle (P9) for tasks that may edit tests. Claims from them are verified before being cited (item 18) |
| 17 | Be bold, experiment, prove or kill quickly | Ongoing | Experiments log: `docs/experiments.md` |
| 18 | GPU compute (3070 Ti, Colab, Kaggle) put to use | Staged, not run | `hc probe-model` and `gpu/probe_local_models.ipynb`: schema-constrained vs prompt-only claim filing on local models |
| 19 | Notion as agent-accessible storage; ignore Operantix | Done | `hc export-notion` (API 2025-09-03, `markdown_version: v2`); Operantix untouched |
| 20 | Quickstart must be low-friction and uncluttered, like well-integrated cross-provider tools | Done | One command, `hc init "objective"`: detects the test command and installed harnesses, writes the task file, touches only those harnesses' instruction files; `hc brief` defaults to the task file |
| 21 | A doctor command built from what the best doctor commands share; test setup on simulated machines | Done (Linux), CI for macOS/Windows | `hc doctor` (grouped ✓/!/✗, a fix per problem, `-v`, `--json`, exit 1 only on errors, runs before the home exists, real sign/verify and sandbox probes); `hc init` refuses early on the same errors. `scripts/matrix.sh`: 14/14 on locally built Ubuntu 20.04/22.04/24.04 images and fault cases. `.github/workflows/ci.yml` covers macOS and Windows (not run here) |
| 22 | Astra evidence corpus (information): does a useful fact reach the next agent, and what happens after an edit? | Applied | Found and fixed: hook sessions never snapshotted after edits (honest claims were refuted), missing evidence reported as refutation, facts not bound to a tree, verdicts recomputed on every brief. `scripts/exhibit-stale.sh` is now a gate criterion; assay requires every verifiable honest claim delivered (32/32) |

| 23 | Presentation: a scalable mark that cannot be mistaken for anything unfortunate, a clean README that traces foundation to current edge, context kept connected | Done | `assets/mark.svg` (checked at 16 to 512 px, light and dark) and `assets/mark.txt`. The README leads with the exhibit and keeps claim, observation, verdict and receiver consequence together; the gate checks that its packet lines are byte-identical to the release candidate's output |
| 24 | Fear that the tests are tautological; recheck foundational and current literature | Done | Ground-truth manifest for the assay; known-bad run must fail with the expected signature; `scripts/mutants.py` (22 trust-critical mutants, baseline-gated); claims cited in code rechecked against primary sources (`docs/claim-battery.md`, recheck section) |
| 25 | A final go-to-market audit: adversarial, expert, assumes the implementation misses its plan; engineering only, no rewrite | Done | `docs/audit-gtm.md` (13 release-blocking on `56d6818`); every confirmed blocking finding replayed or documented in `docs/audit-gtm-resolution.md`, enforced by gate criterion 8. T12 remains a documented boundary of hook mode |
| 26 | Systems-engineer steering: distinct states, the release question, a falsifiable acceptance result, one exhibit a developer understands in a minute | Applied | Every audit finding replayed through the real entry points (`scripts/attacks.sh`); manifest acceptance; facts name their claim's ledger entry, checks and tree |

| 27 | Systems-engineer steering 2: establish what actually ran in the F2 discrepancy, make the battery earn its result, trace recent changes through the shipped path, follow a fact to its observation | Applied | F2: stale binary from timestamp-preserving sync, shown by identical sources with differing rendering; battery baseline-gated with per-mutant expected detector; F9 verified on the headless `hc run` path (replay T11); briefs name the snapshot entry, claim entry and checks for every line |

## Open items, in order

18. Verify the corpus items the code or docs cite against primary sources (claim battery). The Astra corpus's preprints (HSC-Bench, Soei, Ledger) are recorded as unverified until opened.
23. Live pilot in Claude Code, Codex and Antigravity: the hook payloads follow vendor references, but no live session has been recorded.
24. Receiver-side measurement: the first consequential action of a fresh agent given the brief versus a raw transcript, paired per task (the corpus's measurement notes 1 to 4).
