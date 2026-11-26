# Architecture

Dependencies point down this list; nothing depends on a crate below it.

| Crate | One job | Key types |
|---|---|---|
| `bedrock` | Shared vocabulary: IR, RFC 8785 canonical bytes, SHA-256, hermetic git | `ir::{Statement, Obs, Event, Claim, ClaimRecord, VerdictRecord, Trust, PassId}`, `git::Git`, `Digest` |
| `seal` | Per-identity ed25519 SSH keys and `allowed_signers` | `Keyring::ensure(role, name)`, `Identity::principal()` |
| `docket` | Signed hash-chained ledger on `refs/hc/docket` | `Docket::append`, `Docket::records` → `Record { seq, commit, signer, body }` |
| `intake` | P1 normalize and P2 typecheck of agent output | `compile(raw) -> Result<ClaimEnvelope, Vec<Diagnostic>>`, `CLAIM_SCHEMA` |
| `exhibit` | Git-tree snapshots, diffs, session recording | `Recorder::{open, close, no_claim}`, `record_claims`, `snapshot::{snapshot, materialize}` |
| `cell` | bubblewrap isolation hiding the hc home | `Sandbox::{detect, bind, command}` |
| `roster` | Agent registry as data, ancestry detection, launch templates | `Registry::{load, detect}`, `AgentDef`, `render_argv` |
| `agenda` | Reads native plans into one typed plan; stable step identity | `from_tool`, `from_markdown`, `from_exec_plan`, `Tracker`, `transitions` |
| `conform` | Plan-versus-log conformance: matched, plan-only, log-only; fail-fast fingerprints | `align(records, session) -> Report`, `fingerprint` |
| `crossexam` | Kernel: passes P4, P6, P6S, P7, P8, P9 and the sealed `Verified` | `Kernel::{examine, rule}`, `Policy`, `Finding` |
| `bailiff` | Hook runtime in each harness's native dialect | `handle(env, dialect, input) -> Verdict`, `render` |
| `brief` | Linker: facts, negative facts, externs, current step | `Brief::{link, render, digest}`, `Task` |
| `courier` | Delivery: prompts, AGENTS.md/CLAUDE.md block, Notion pages | `prompt`, `write_protocol_block`, `notion::page_body` |
| `assay` | Seeded acceptance corpus and pass ablations | `evaluate(work, seed, disabled) -> Report` |
| `chambers` | The hc home: keys, per-repository dockets, outboxes, roster overrides | `Home::{locate, docket, identity, outbox, registry}` |
| `clerk` | MCP attestation tier for surfaces without hooks | `Clerk`, tools `schema`, `brief`, `plan`, `claim`, `verify`, `conform`; `serve_stdio` |
| `hc` | CLI wiring | `run`, `claim`, `brief`, `hook`, `mcp`, `conform`, `onboard`, `assay`, `export-notion` |

## Data flow for one session

1. `exhibit::Recorder::open` snapshots the worktree into the docket repo and records `session_start`.
2. The agent runs inside `cell` (hc home hidden, outbox bound).
3. `Recorder::close` snapshots again and records one `file_write` per change plus `session_end`.
4. `intake::compile` turns the final message into a `ClaimEnvelope`; `exhibit::record_claims` signs each claim with the agent's key.
5. `crossexam::Kernel::examine` reads verified docket records and returns findings; `Kernel::rule` appends kernel-signed verdicts.
6. `brief::Brief::link` builds the next brief from findings; only `Verified` values become facts.

## Where state lives

- `$HC_HOME/keys/` keys and `allowed_signers`
- `$HC_HOME/repos/<path digest>/docket.git` one docket per repository
- `$HC_HOME/outbox/<session>/` the only part of the hc home visible inside the sandbox
- `$HC_HOME/roster.json` user roster overrides
