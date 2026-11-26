# Graph Report - handoff-compiler  (2026-11-26)

## Corpus Check
- cluster-only mode — file stats not available

## Summary
- 526 nodes · 1130 edges · 21 communities (20 shown, 1 thin omitted)
- Extraction: 97% EXTRACTED · 3% INFERRED · 0% AMBIGUOUS · INFERRED: 29 edges (avg confidence: 0.83)
- Token cost: 0 input · 0 output

## Community Hubs (Navigation)
- Community 0
- Community 1
- Community 2
- Community 3
- Community 4
- Community 5
- Community 6
- Community 7
- Community 8
- Community 9
- Community 10
- Community 11
- Community 12
- Community 13
- Community 14
- Community 15
- Community 16
- Community 17
- Community 18
- Community 19

## God Nodes (most connected - your core abstractions)
1. `ClaimKind` - 24 edges
2. `DocketError` - 23 edges
3. `GitError` - 22 edges
4. `PassId` - 20 edges
5. `Git` - 20 edges
6. `Docket` - 19 edges
7. `Digest` - 19 edges
8. `AssayError` - 17 edges
9. `Kind` - 17 edges
10. `Trust` - 17 edges

## Surprising Connections (you probably didn't know these)
- `pass_scope()` --references--> `Compiled`  [INFERRED]
  crates/crossexam/src/lib.rs → crates/crossexam/src/policy.rs
- `pass_tamper()` --references--> `Compiled`  [INFERRED]
  crates/crossexam/src/lib.rs → crates/crossexam/src/policy.rs
- `pass_scope()` --calls--> `seqs()`  [INFERRED]
  crates/crossexam/src/lib.rs → crates/brief/src/lib.rs
- `run_session()` --calls--> `make_repo()`  [INFERRED]
  crates/assay/src/lib.rs → crates/assay/src/fixtures.rs
- `act()` --calls--> `commit()`  [INFERRED]
  crates/assay/src/lib.rs → crates/assay/src/fixtures.rs

## Import Cycles
- 1-file cycle: `crates/crossexam/src/reexec.rs -> crates/crossexam/src/reexec.rs`
- 1-file cycle: `crates/courier/src/lib.rs -> crates/courier/src/lib.rs`
- 1-file cycle: `crates/bedrock/src/digest.rs -> crates/bedrock/src/digest.rs`
- 2-file cycle: `crates/assay/src/fixtures.rs -> crates/bedrock/src/git.rs -> crates/assay/src/fixtures.rs`

## Communities (21 total, 1 thin omitted)

### Community 0 - "Community 0"
Cohesion: 0.07
Nodes (48): AttestationLevel, Ancestry, Declared, Launcher, Body, Claim, Obs, Verdict (+40 more)

### Community 1 - "Community 1"
Cohesion: 0.07
Nodes (27): Oid, ResourceDescriptor, Statement, Statement<P>, appends_verifies_and_is_deterministic(), copy_dir(), decode(), Docket (+19 more)

### Community 2 - "Community 2"
Cohesion: 0.08
Nodes (30): ClaimKind, Blocked, CommandRan, DerivedNotRetrieved, FilesChanged, FilesUntouched, NoChangeNeeded, Note (+22 more)

### Community 3 - "Community 3"
Cohesion: 0.10
Nodes (27): AgentAttestation, AgentDef, basename(), BUILTIN, builtin_roster_is_well_formed(), candidate_names(), declared(), detects_a_registered_ancestor() (+19 more)

### Community 4 - "Community 4"
Cohesion: 0.10
Nodes (24): Task, Home, adjudicate(), Cli, cmd_assay(), cmd_claim(), cmd_run(), Command (+16 more)

### Community 5 - "Community 5"
Cohesion: 0.07
Nodes (7): digest(), MAX_SAFE_INTEGER, rfc8785_reference_vectors(), vectors(), Digest, DigestParseError, round_trips()

### Community 6 - "Community 6"
Cohesion: 0.11
Nodes (15): Git, GitError, Failed, Spawn, Utf8, added_lines(), Change, changes() (+7 more)

### Community 7 - "Community 7"
Cohesion: 0.13
Nodes (15): Role, Agent, Harness, Kernel, generates_key_once_and_registers_it(), IdentityError, Io, Keygen (+7 more)

### Community 8 - "Community 8"
Cohesion: 0.12
Nodes (10): Isolation, Bwrap, None, hides_the_hc_home_but_exposes_the_outbox(), Sandbox, ALLOWED_FIELDS, frontmatter(), links() (+2 more)

### Community 9 - "Community 9"
Cohesion: 0.17
Nodes (14): ClaimEnvelope, Diagnostic, CLAIM_SCHEMA, compile(), cross_field_rules(), envelope_json(), missing_field_is_a_schema_error(), normalize() (+6 more)

### Community 10 - "Community 10"
Cohesion: 0.13
Nodes (13): all_lines(), EXIT_FNS, first_statement_returns_true(), Hit, is_python(), language(), negatives(), only_added_lines_count() (+5 more)

### Community 11 - "Community 11"
Cohesion: 0.18
Nodes (14): BEGIN, create_page(), END, notion_body_targets_a_data_source_with_v2_markdown(), NOTION_VERSION, NotionError, Http, page_body() (+6 more)

### Community 12 - "Community 12"
Cohesion: 0.17
Nodes (6): Compiled, DEFAULT_PROTECTED, glob(), Policy, protected_globs(), test_commands_match_by_argv_not_text()

### Community 13 - "Community 13"
Cohesion: 0.14
Nodes (10): AssayError, Brief, Docket, Git, Identity, Intake, Io, Kernel (+2 more)

### Community 14 - "Community 14"
Cohesion: 0.26
Nodes (11): CARGO, counts(), parse_counts(), PYTEST, Run, Source, Commit, Tree (+3 more)

### Community 15 - "Community 15"
Cohesion: 0.23
Nodes (10): Case, commit(), eval_buggy(), make_repo(), Op, OPS, OpSpec, repo_git() (+2 more)

### Community 16 - "Community 16"
Cohesion: 0.18
Nodes (11): evaluate(), generate(), Report, SessionResult, PassId, .ALL, Command, Reexec (+3 more)

### Community 17 - "Community 17"
Cohesion: 0.15
Nodes (13): CORPUS, Kind, Blocked, CommandSwap, ExitZero, FixCommitted, FixUncommitted, HookConfig (+5 more)

### Community 18 - "Community 18"
Cohesion: 0.27
Nodes (5): act(), claim(), paths(), tests_pass(), workdir()

### Community 19 - "Community 19"
Cohesion: 0.58
Nodes (12): assay, bedrock, brief, cell, courier, crossexam, docket, exhibit (+4 more)

## Knowledge Gaps
- **91 isolated node(s):** `Statement<P>`, `Ancestry`, `Declared`, `Launcher`, `Claim` (+86 more)
  These have ≤1 connection - possible missing edges. (Counts symbols only; 189 node(s) total have ≤1 connection when file, concept and rationale nodes are included.)
- **1 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `Digest` connect `Community 5` to `Community 0`, `Community 1`, `Community 2`, `Community 8`?**
  _High betweenness centrality (0.059) - this node is a cross-community bridge._
- **What connects `Statement<P>`, `Ancestry`, `Declared` to the rest of the system?**
  _91 weakly-connected nodes found - possible missing edges._
- **Should `Community 0` be split into smaller, more focused modules?**
  _Cohesion score 0.06662770309760374 - nodes in this community are weakly interconnected._
- **Why does `PassId` connect `Community 16` to `Community 0`, `Community 18`, `Community 4`, `Community 13`?**
  _High betweenness centrality (0.052) - this node is a cross-community bridge._
- **Should `Community 1` be split into smaller, more focused modules?**
  _Cohesion score 0.06896551724137931 - nodes in this community are weakly interconnected._
- **Why does `ClaimKind` connect `Community 2` to `Community 0`?**
  _High betweenness centrality (0.051) - this node is a cross-community bridge._
- **Should `Community 2` be split into smaller, more focused modules?**
  _Cohesion score 0.07549361207897794 - nodes in this community are weakly interconnected._