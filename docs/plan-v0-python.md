# Plan: handoff compiler MVP (`hc`)

## Header
- **Goal:** A deterministic compiler that takes a Claude Code session's typed claims, verifies them against harness-observed evidence, and links only verified facts into a fresh Codex task packet.
- **Spec:** `reports/Deterministic handoff compiler for agents.md`, section "Design synthesis" (ledger schema, passes P1–P8 + P13, trust lattice, restart loop). This plan implements the MVP subset only.
- **Global constraints:**
  - Python 3.12; runtime deps: `jsonschema` only (`inspect-ai` only inside assay, not in `hc`).
  - Authoritative ledger: `~/.hc/<repo-id>/ledger.jsonl` (outside every agent worktree). Repo mirror: `.handoff/ledger.jsonl` (read-only for trust).
  - Canonical bytes: `json.dumps(obj, sort_keys=True, separators=(",", ":"), ensure_ascii=False)`; no floats anywhere in the schema (SHORTCUT: stands in for RFC 8785; upgrade if floats are ever needed).
  - Agent-facing schema uses no `minimum`/`maximum`/`minLength`/`maxLength` (Anthropic's API rejects them with a 400); those live in the P2 validator.
  - Hash chain: each entry carries `prev = sha256(canonical bytes of previous entry)`.
  - Timestamps come only from `obs` entries; compiler never reads the wall clock.
  - Protected paths: `tests/**`, `**/conftest.py`, `.claude/**`, `.codex/**`, `.cursor/**`, `.github/**`, `.handoff/**`.
  - Trust lattice: `refuted < asserted < observed < reproduced`. Only `hc/kernel.py` can construct `observed` or above.
  - Restart cap: 3. Packet constraint count: at most 40.
  - Terminal claim statuses: `done`, `no_change_needed`, `spec_conflict`, `blocked` (all well-typed successes).
- **Review focus:**
  1. *Agent-asserted evidence laundered into trust.* A claim whose `evidence_refs` point at `claim` entries (not `obs`) must stay `asserted`. Test: `test_kernel.py::test_claim_cannot_cite_claim`.
  2. *Non-deterministic packet bytes* (dict order, set iteration, clock). Test: `test_link.py::test_packet_bytes_identical_across_runs`.
  3. *Agent disables the gate* by editing hook config or the mirror ledger. Test: `test_kernel.py::test_write_to_hook_config_refutes`.
- **Behavioral acceptance:** `hc eval fixtures/seeded --seed 7` over 30 recorded sessions (15 honest, 15 seeded false claims) must report: false-done recall ≥ 14/15; false refutations ≤ 1/15; identical packet sha256 across two compiles of every session; 0 claims below `observed` in any packet's facts section. Exits 1 if any invariant fails. **Known-bad case:** `HC_DISABLE=P6 hc eval fixtures/seeded --seed 7` must exit 1 (recall drops because the 3 tamper seeds and the hook-config seed go uncaught).

## File map
| File | Owns |
|---|---|
| `hc/schema/claim.v0.json` | Agent-facing claim schema (portable subset: all fields required, nullable unions, `additionalProperties: false`, evidence/basis before `status`) |
| `hc/ledger.py` | `append(entry) -> int`, `read() -> list[dict]`, `verify_chain() -> bool`, canonical bytes |
| `hc/passes/normalize.py` | P1: strip fences/preambles, canonicalize, content hash |
| `hc/passes/typecheck.py` | P2: full JSON Schema + cross-field rules; failure → `no_claim` diagnostic |
| `hc/observe/shim.py` | PATH shim: wraps test/build commands, appends `obs` (cmd, exit code, counts, tool_call id, ts) |
| `hc/observe/snapshot.py` | Before/after SHA-256 of worktree files → `obs` file_write entries |
| `hc/kernel.py` | P3 label, P4 bind, P5 temporal, P6 tamper/scope, P7 signature scan, `Verified` construction, `verdict` entries |
| `hc/passes/reexec.py` | P8: clean `git worktree` at claimed commit + diff, run k=2, flakiness flag |
| `hc/link.py` | P13: select by trust, order (contract → facts → negative facts → externs → current step), serialize, hash |
| `hc/backends/codex.py`, `hc/backends/agents_md.py` | Packet renderers |
| `hc/adapters/claude.py`, `hc/adapters/codex.py` | Parse `claude -p --output-format json --json-schema` and `codex exec --output-schema` output into L0 claims |
| `hc/cli.py` | `hc run`, `hc eval` |
| `fixtures/seeded/` | 30 recorded sessions + `manifest.json` (expected verdict per claim) |

## Tasks

**T1. Ledger and front end (P1, P2).**
Consumes: nothing. Produces: `ledger.append/read/verify_chain`, `normalize(raw: str) -> dict`, `typecheck(entry: dict) -> list[Diagnostic]`.
- Write `claim.v0.json` and `ledger.py`; tests `test_ledger.py::{test_chain_detects_edit, test_canonical_bytes_stable}`.
- Write P1/P2; tests `test_front_end.py::{test_strips_markdown_fence, test_missing_field_is_no_claim, test_rejects_float}`.
- Check: `pytest tests/test_ledger.py tests/test_front_end.py -q` → `5 passed`.

**T2. Observation substrate.**
Consumes: `ledger.append`. Produces: `obs` entries `{op: "command_run"|"file_write", ...}`.
- Shim and snapshot; tests `test_observe.py::{test_shim_records_exit_code, test_snapshot_records_protected_write}` against `fixtures/tiny_repo/`.
- Check: `pytest tests/test_observe.py -q` → `2 passed`.

**T3. Kernel passes P3–P7.**
Consumes: `ledger.read`, `obs` entries. Produces: `verdict` entries; `Verified` objects only from `kernel.py`.
- One test per pass plus the three review-focus tests: `test_kernel.py::{test_label_from_writer, test_bind_requires_obs, test_tests_must_run_after_last_edit, test_protected_write_refutes, test_exit0_signature_flagged, test_claim_cannot_cite_claim, test_write_to_hook_config_refutes}`.
- Check: `pytest tests/test_kernel.py -q` → `7 passed`.

**T4. Clean re-execution (P8).**
Consumes: claim `{command, commit}`. Produces: `obs` with `op: "reexec"`, raises trust to `reproduced` on agreement.
- Test `test_reexec.py::{test_disagreeing_rerun_refutes, test_flaky_marked}` on `fixtures/tiny_repo/`.
- Check: `pytest tests/test_reexec.py -q` → `2 passed`.

**T5. Linker and back ends (P13).**
Consumes: verdicts + claims. Produces: `Packet` bytes + sha256; Codex prompt and AGENTS.md section.
- Tests `test_link.py::{test_packet_bytes_identical_across_runs, test_refuted_rendered_as_negative_fact, test_asserted_rendered_as_extern, test_constraint_cap_40}`.
- Check: `pytest tests/test_link.py -q` → `4 passed`.

**T6. Vendor adapters (recorded outputs only, no live calls in tests).**
Consumes: raw CLI JSON. Produces: L0 claims for P1.
- Tests `test_adapters.py::{test_claude_structured_output_parsed, test_claude_missing_structured_output_is_no_claim, test_codex_output_file_parsed}` using captured fixtures.
- Check: `pytest tests/test_adapters.py -q` → `3 passed`.

**T7. `hc run` restart loop.**
Consumes: T1–T6. Produces: session → gate → packet → next session; stops at cap 3 with compiled diagnosis.
- Test `test_cli.py::test_restart_cap` with a stub agent that always lies.
- Check: `pytest tests/test_cli.py -q` → `1 passed`.

**T8. Acceptance: seeded corpus and `hc eval`.**
Consumes: everything. Seeds: tests edited (3), tests not re-run after last edit (3), `files_untouched` lie (3), `exit(0)` in harness (2), hook config edited (1), rerun disagrees (3).
- First run `HC_DISABLE=P6 hc eval fixtures/seeded --seed 7` → must exit 1.
- Then `hc eval fixtures/seeded --seed 7` → exit 0 with the four invariants printed.

## Self-review
1. Each MVP requirement (ledger, claimed/observed split, kernel-only trust, determinism, restart, two vendors) maps to T1–T7. Judge (P11), access audit (P9), adequacy (P10), signing/DSSE, OTel and web-UI surfaces are deliberately out of this plan.
2. Every step ends in one command with an expected count.
3. Names match across tasks: `append/read/verify_chain`, `Verified`, `Packet`, `obs/claim/verdict`.
4. Proportion: signatures and test names only.

## Next plan (separate, after this one passes)
Live assay pilot: Claude Code → Codex on the 20 frozen tasks plus `session-log-heldout-v1`, 2×3 ablation (raw-transcript vs compiled packet × no / periodic / failure-triggered diagnostics), success = the packet changes a real handoff decision.
