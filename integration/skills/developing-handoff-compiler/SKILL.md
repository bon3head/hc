---
name: developing-handoff-compiler
description: Guides changes to the hc handoff-compiler Rust workspace (bedrock, seal, docket, intake, exhibit, cell, roster, agenda, conform, crossexam, bailiff, brief, courier, chambers, clerk, assay, hc). Covers which crate owns a concern, the invariants that must not break, which packed reference code and spec to read first, and the validation loop. Use when editing, reviewing or extending this repository.
license: MIT
---

# Developing the handoff compiler

## Orient first

Find the crate that owns the concern in [references/architecture.md](references/architecture.md). Each crate does one thing; a change that spans many crates usually belongs in one.

Before writing code against a dependency or a format, open the matching reference listed in [references/reading-list.md](references/reading-list.md). Packs are version-pinned to `Cargo.lock`, so they show the API that actually compiles.

## Invariants

- Only `crossexam` mints `Verified`; `crates/crossexam/tests/ui/forge_verified.rs` must keep failing to compile.
- Every docket entry is an SSH-signed commit whose signer role matches its predicate type. Never add an unsigned write path.
- All git calls go through `bedrock::git::Git`, which disables global and system config.
- Canonical bytes come from `bedrock::canon` (RFC 8785). Claim integers stay within ±(2^53−1).
- Agent text never chooses which checks run and never enters the facts section of a brief.
- Any component that implements a spec cites it in `docs/decisions.md` with its conformance tests.

## Validation loop

Run this before calling a change done, and repeat until every step is clean:

```sh
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
rust-analyzer diagnostics .
HC_DISABLE=tamper cargo run -q -p hc -- assay --seed 7   # must print REJECT
cargo run -q -p hc -- assay --seed 7                       # must print ACCEPT
scripts/gate.sh                                            # the release gate: must print PASS
```

If a change adds a detection pass or cheat signature, add a seeded lie to `crates/assay/src/lib.rs` that only the new code catches, and confirm that disabling the new code turns ACCEPT into REJECT. Details: [references/validation.md](references/validation.md).
