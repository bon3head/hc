# Design decisions and their sources

Every component that implements a format, protocol or algorithm names what it implements and the evidence it is tested against. Spec copies live in `refs/specs/`; packed sources of every pinned dependency live in `refs/packs/`.

## Formats and protocols

| Component | Implements | Conformance evidence | Location |
|---|---|---|---|
| Canonical JSON | RFC 8785 (JCS) via `serde_jcs` 0.4 | All six cyberphone reference vectors (`refs/specs/jcs/testdata`); proptest permutation invariance and fixed point; integers beyond an exact double rejected | `crates/bedrock/src/canon.rs` |
| Claim integers | I-JSON safe range, RFC 7493 §2.2: ±(2^53−1) | Front-end diagnostic `unsafe-integer` | `crates/intake/src/lib.rs` |
| Ledger entry shape | in-toto Statement v1 (`_type`, `subject[].digest`, `predicateType`, `predicate`); git digest names `gitTree`, `gitBlob`, `gitCommit` from `digest_set.md` | Round-trip decode of every entry on read | `crates/bedrock/src/ir.rs` |
| Ledger | One commit per entry on `refs/hc/docket` in a bare repository; parent links are the hash chain | Tamper tests: foreign key → `U` rejected, wrong role rejected, deterministic commit ids | `crates/docket/src/lib.rs` |
| Signatures | git SSH commit signing (`gpg.format=ssh`), ed25519 keys from `ssh-keygen`, verification against `allowed_signers` (ssh-keygen(1) ALLOWED SIGNERS) via `%G?` / `%GS` | Signer must be `G` and its role must match the predicate type | `crates/seal`, `crates/docket` |
| Hermetic git | `GIT_CONFIG_GLOBAL=/dev/null`, `GIT_CONFIG_NOSYSTEM=1` on every call | A sandbox-injected global signing program was observed to break verification; hermetic calls are immune | `crates/bedrock/src/git.rs` |
| Snapshots | git tree objects written through a temporary `GIT_INDEX_FILE` into the docket's object database; ignored files under protected patterns force-added | Snapshot test with gitignored `.claude/` config | `crates/exhibit/src/snapshot.rs` |
| Diffs | `git diff-tree -r` and `git diff -U0` hunk headers for added lines | Hunk-range unit tests | `crates/exhibit/src/diff.rs` |
| Isolation | bubblewrap: whole filesystem visible, the hc home covered by tmpfs, only the session outbox bound back | Test that a secret in the hc home is invisible inside the cell | `crates/cell/src/lib.rs` |
| Agent detection | Process ancestry from `/proc/<pid>/{stat,exe,cmdline}` (Linux) or `ps` (macOS); interpreter-wrapped CLIs matched on script name | Live detection of a registered ancestor | `crates/roster/src/lib.rs` |
| Claim schema | JSON Schema 2020-12, portable subset accepted by OpenAI strict mode, Anthropic structured outputs, `codex exec --output-schema`, `claude --json-schema`, `agy --json-schema` | Schema-lint test: all properties required, no `additionalProperties`, no unsupported keywords, `basis` before `status` | `schema/claim.v0.json` |
| Native plans | Claude `TodoWrite`/`TaskCreate`/`TaskUpdate`/`ExitPlanMode`, Codex `update_plan`, Gemini `write_todos`, ACP `plan`, Markdown checklists, Codex ExecPlan `## Progress` (OpenAI Cookbook) | Parser test per shape | `crates/agenda/src/lib.rs` |
| Hook dialects | Claude Code and Codex hook references (shared snake_case dialect); Antigravity hook reference (camelCase, `injectSteps`, `decision: continue`) | Rendering tests per event; ten end-to-end hook sessions | `crates/bailiff/src/dialect.rs` |
| Hook configs | `.claude/settings.json`, `.codex/hooks.json` (`{"hooks": {Event: [{matcher, hooks}]}}`), `.agents/hooks.json` (flat) | Idempotent merge test preserving foreign hooks | `crates/courier/src/lib.rs` |
| Notion export | Notion API `2025-09-03` (default in the official SDK): page under `parent: {type: "data_source_id"}`, body as `markdown` with `markdown_version: "v2"` | Request-body test | `crates/courier/src/lib.rs` |
| Agent Skills | agentskills.io specification and the `skills-ref` validator's rules; Anthropic authoring practices (reserved words, under 500 lines, one-level references, contents list over 100 lines) | `crates/hc/tests/skill_pack.rs` | `integration/skills/` |

## Algorithms

| Component | Basis | Evidence | Location |
|---|---|---|---|
| Step identity for id-less plans | Patience-diff anchoring (Bram Cohen): unique texts first, repeats in order, then token-set Jaccard ≥ 0.5 with nearest position; identical texts get distinct keys | Reorder, reword, removal, duplicate-boilerplate and ExecPlan timestamp tests | `crates/agenda/src/lib.rs` |
| Plan conformance | Process-mining conformance checking (van der Aalst): plan as model, docket as log; events attributed to the in-progress step; labels matched / plan-only / log-only; fitness = matched ÷ (matched + plan-only + unplanned) | Unit tests; honest and false-completion hook sessions | `crates/conform/src/lib.rs` |
| Cheat signatures | Tree-sitter queries on lines the session added: exit with code 0 and `SkipTest` (Baker et al. 2025), `__eq__` always true (ImpossibleBench 2025), patched clocks (METR 2025); in test files, assertion-free tests and assertions that cannot fail | Positive and negative fixtures per rule | `crates/crossexam/src/signatures.rs` |
| Clean re-execution (P8) | Detached `git worktree` at the claimed commit, or the observed tree materialized from the docket; run twice | Seeded `stale_run` and `wrong_commit` lies | `crates/crossexam/src/reexec.rs` |
| Fail-to-pass (P9) | SWT-Bench dual-state validation: the session's tests overlaid on the pre-session tree must fail | Seeded `rigged_tests` lie caught only by P9; honest regression tests pass | `crates/crossexam/src/lib.rs`, `crates/exhibit/src/snapshot.rs` |
| Fail-fast | Same failure fingerprint (sorted, deduplicated failure texts) twice in a row → one escalation asking for a root-cause report, then the stop is allowed; a different failure resets it | Hook-session test including a new failure after escalation | `crates/bailiff/src/lib.rs`, `crates/conform/src/lib.rs` |
| Feedback framing | Corrections presented as external observations of the working tree rather than critique of the agent's own output (self-correction asymmetry; verification of the cited study pending, see `docs/steering.md` item 18) | Header asserted in hook-session tests | `crates/bailiff/src/lib.rs` |
| Verified type | LCF-style abstract type: a private seal field makes `Verified` constructible only inside `crossexam` | `trybuild` compile-fail test | `crates/crossexam/src/verified.rs` |

## Decisions not taken

| Considered | Outcome | Why |
|---|---|---|
| Per-entry DSSE envelopes with `ed25519-dalek` | Dropped | No verifier on a second machine yet; git's own SSH signing covers authorship. Signed checkpoints of the docket head (Certificate Transparency's signed tree head pattern) are the upgrade when one exists |
| An API proxy for context injection | Dropped | Cannot reach subscription CLIs or hosted agents; wire formats are vendor-owned. Native hook channels replace it |
| PATH shims to observe agent test runs | Dropped | The agent's own runs are not trusted; the kernel re-executes instead |
| Periodic generic reminders ("heartbeats") | Dropped | Gains saturate after one restatement; failure-triggered, observation-specific feedback through native hooks replaces them |
| Custom canonical JSON | Dropped | `serde_jcs` implements RFC 8785 and passes the reference vectors |
