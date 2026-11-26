# Reading list

All paths are relative to the repository root. Packs are repomix XML (compressed signatures plus doc comments); graphs are graphify output (`graph.json` plus `GRAPH_REPORT.md`) for structural queries such as `graphify query "<question>" --graph refs/graphs/<name>/graphify-out/graph.json`.

## By task

| Task | Read first |
|---|---|
| Path matching, ignore rules, walking trees | `refs/packs/exemplar-ripgrep-globset-ignore.xml`, `refs/graphs/ripgrep/` |
| Tree-sitter parsing or new cheat signatures | `refs/packs/exemplar-ast-grep-core.xml`, `refs/graphs/ast-grep/`, `refs/packs/crate-tree-sitter-0.27.2.xml` |
| Attestations, statements, envelopes | `refs/specs/in-toto/statement.md`, `refs/specs/dsse/protocol.md`, `refs/packs/exemplar-in-toto-rs.xml`, `refs/graphs/in-toto-rs/` |
| Signature verification and bundles | `refs/packs/exemplar-sigstore-rs.xml`, `refs/graphs/sigstore-rs/` |
| Canonical JSON | `refs/specs/jcs/README.md`, `refs/specs/jcs/testdata/`, `refs/packs/crate-serde_jcs-0.4.0.xml` |
| Claim schema validation | `refs/packs/crate-jsonschema-0.63.1.xml` |
| CLI flags and subcommands | `refs/packs/crate-clap-4.6.10.xml` |
| HTTP and the Notion export | `refs/packs/crate-ureq-3.5.2.xml`, `refs/packs/spec-notion-sdk-api.xml` |
| Timestamps | `refs/packs/crate-jiff-1.0.3.xml` |
| Command matching | `refs/packs/crate-shlex-2.0.1.xml`, `refs/packs/crate-regex-1.13.2.xml` |

## Why these exemplars

- ripgrep's `globset` and `ignore`: small focused crates, a builder API, thorough doc tests. The model for crate boundaries here.
- ast-grep core: a cached parser per language and node-text matching over tree-sitter, the pattern `crossexam::signatures` follows.
- in-toto-rs and sigstore-rs: typed attestation models and verification code that keeps untrusted input separate from verified output.

## Code index

`refs/graphs/handoff-compiler/` is the graphify graph of this workspace (526 nodes). For a precise symbol index, generate one locally with `rust-analyzer scip .` and query it with any SCIP tool; it is not committed because it goes stale with every change.
