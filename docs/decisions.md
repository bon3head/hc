# Design decisions and their sources

Every component that implements a format or protocol names the document it implements and the conformance material it is tested against. Copies of the specs live in `refs/specs/`; packed sources of every pinned dependency live in `refs/packs/`.

| Component | Implements | Conformance evidence | Location |
|---|---|---|---|
| Canonical JSON | RFC 8785 (JCS): UTF-16 code-unit key order, ECMAScript number serialization, lowercase `\u00xx` control escapes, no whitespace, UTF-8 output | All six cyberphone reference vectors (`refs/specs/jcs/testdata/{input,output}`); proptest for permutation invariance and idempotence | `crates/hc-core/src/jcs.rs` |
| Number formatting | ECMA-262 §7.1.12.1 Number::toString, as required by RFC 8785 §3.2.2.3 | `ryu-js` crate (Ryū with ECMAScript output rules); covered by `values.json` vector | dependency |
| Envelopes and signatures | DSSE v1 protocol: `PAE(type, body) = "DSSEv1" SP LEN(type) SP type SP LEN(body) SP body`, JSON envelope | PAE vector from `refs/specs/dsse/protocol.md` (`DSSEv1 29 http://example.com/HelloWorld 11 hello world`); round-trip and tamper tests | `crates/hc-core/src/dsse.rs` |
| Signature algorithm | Ed25519 (RFC 8032) through `ed25519-dalek` 3.0, verified with `verify_strict` | Upstream test suite; local sign/verify/tamper tests | `crates/hc-core/src/keys.rs` |
| Ledger entry shape | in-toto Attestation Statement v1 (`_type`, `subject[]` with `digest`, `predicateType`, `predicate`), carried in a DSSE envelope with `payloadType: application/vnd.in-toto+json` | `refs/specs/in-toto/statement.md`, `envelope.md`; schema tests on every emitted statement | `crates/hc-core/src/statement.rs` |
| Hash chain | Each ledger line carries `prev = sha256(JCS(previous line))`, the append-only construction of Certificate Transparency logs without the Merkle tree | Tamper tests: edit, reorder, truncate, re-sign with a foreign key | `crates/hc-core/src/ledger.rs` |
| File locking | `std::fs::File::lock` (stable since Rust 1.89) | std | `crates/hc-core/src/ledger.rs` |
| Content store | Content-addressed blobs keyed by SHA-256, the object model used by git and Nix | Round-trip tests | `crates/hc-observe/src/blobs.rs` |
| Line diffs | Myers diff via `similar` 3.2 | Upstream; local added-line tests | `crates/hc-observe/src/diff.rs` |
| Shell words | POSIX shell word splitting via `shlex` | Upstream | command matching in `hc-kernel` |
| Path policy | Glob semantics of ripgrep's `globset` (`**` crosses directories) | Upstream; policy tests | `crates/hc-kernel/src/policy.rs` |
| Cheat signatures | Tree-sitter S-expression queries over the Python grammar, matched only on lines the session added; query style follows ast-grep's core matcher (`refs/packs/exemplar-ast-grep-core.xml`) | One positive and one negative fixture per signature | `crates/hc-kernel/src/signatures.rs` |
| Clean re-execution | `git worktree add --detach <commit>` through the git CLI, which defines worktree semantics | Integration tests on generated repos | `crates/hc-kernel/src/reexec.rs` |
| Verified type | LCF-style abstract type: a private seal field makes `Verified` constructible only inside `hc-kernel` | `trybuild` compile-fail test | `crates/hc-kernel/src/verified.rs` |
| Claim schema | JSON Schema 2020-12, portable subset accepted by OpenAI strict mode, Anthropic structured outputs, `codex exec --output-schema`, `claude --json-schema` and `agy --json-schema` | `jsonschema` 0.63 validation; schema lint test enforcing the subset | `schema/claim.v0.json` |
| Notion export | Notion API `2025-09-03` (default version in the official SDK, `refs/packs/spec-notion-sdk-api.xml`): pages created under `parent: {type: "data_source_id"}` | Request-body golden tests | `crates/hc-adapters/src/notion.rs` |

## Research basis

The architecture is specified in `../reports/Deterministic handoff compiler for agents.md` (copied into `docs/research-report.md`), with the claim-battery fact check in `docs/claim-battery.md`.
