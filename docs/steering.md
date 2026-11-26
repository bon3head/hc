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
| 13 | Work on any surface, including web UIs | Gap | Attestation tier for web surfaces (remote MCP) not built yet: item 17 |
| 14 | Reset context; separate generation from verification; fail-fast after the same failure twice | Done in code, skill text pending | `hc run` restarts from a fresh brief; kernel is separate code and identity; `bailiff` fingerprint escalation (scoped per failure). The user's stop rule goes into the skills verbatim: item 16 |
| 15 | Pasted research corpora are information, not new directives; the main plan stays the task | Applied | Used only where they closed a concrete gap: external-observation framing, ExecPlan reader, patience-style step identity, fault-injection fixes, fail-to-pass oracle (P9) for tasks that may edit tests. Claims from them are verified before being cited (item 18) |
| 17 | Be bold, experiment, prove or kill quickly | Ongoing | Experiments log: `docs/experiments.md` |
| 18 | GPU compute (3070 Ti, Colab, Kaggle) put to use | Gap | Local-model schema-compliance experiment and notebook not staged yet: item 20 |
| 19 | Notion as agent-accessible storage; ignore Operantix | Done | `hc export-notion` (API 2025-09-03, `markdown_version: v2`); Operantix untouched |

## Open items, in order

16. Finish P9 and vacuous-test detection with seeded lies; refresh the skill pack (hooks, P9, the user's fail-fast rule verbatim); rewrite `docs/decisions.md`.
17. Web-surface attestation: `hc mcp`, a stdio MCP server exposing `claim`, `brief` and `conform`.
18. Verify the corpus items the code now cites against primary sources.
19. Diagnostics as secondary step evidence: a task-level `check_command` (for example `cargo check`, `ruff check`, `pyright`) run at step boundaries.
20. GPU: schema-compliance experiment for local models (Ollama) plus a Colab/Kaggle notebook.
21. Second independent audit (agenda, conform, bailiff); claim-battery pass on the native-plans report.
