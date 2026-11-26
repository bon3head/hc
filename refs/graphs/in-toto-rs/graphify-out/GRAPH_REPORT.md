# Graph Report - in-toto-rs  (2026-11-25)

## Corpus Check
- cluster-only mode — file stats not available

## Summary
- 839 nodes · 1906 edges · 42 communities (30 shown, 12 thin omitted)
- Extraction: 99% EXTRACTED · 1% INFERRED · 0% AMBIGUOUS · INFERRED: 20 edges (avg confidence: 0.74)
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
- Community 20
- Community 21
- Community 22
- Community 23
- Community 24
- Community 25
- Community 26
- Community 27
- Community 28
- Community 29
- Community 30
- Community 31
- Community 32
- Community 33
- Community 34
- Community 35
- Community 36
- Community 37
- Community 38
- Community 39
- Community 40
- Community 41

## God Nodes (most connected - your core abstractions)
1. `LinkMetadata` - 40 edges
2. `VirtualTargetPath` - 37 edges
3. `KeyId` - 36 edges
4. `ArtifactRule` - 35 edges
5. `PublicKey` - 32 edges
6. `LayoutMetadata` - 31 edges
7. `Json` - 29 edges
8. `Command` - 27 edges
9. `KeyType` - 27 edges
10. `Result` - 26 edges

## Surprising Connections (you probably didn't know these)
- `main()` --calls--> `in_toto_run()`  [INFERRED]
  examples/in_toto_run_test.rs → src/runlib.rs
- `in_toto_run_record_file()` --calls--> `in_toto_run()`  [INFERRED]
  tests/runlib.rs → src/runlib.rs
- `in_toto_run_record_new_file()` --calls--> `in_toto_run()`  [INFERRED]
  tests/runlib.rs → src/runlib.rs
- `in_toto_run_record_symlink_file()` --calls--> `in_toto_run()`  [INFERRED]
  tests/runlib.rs → src/runlib.rs
- `PublicKey` --references--> `KeyType`  [INFERRED]
  src/interchange/cjson/shims.rs → src/crypto.rs

## Import Cycles
- 1-file cycle: `src/models/layout/mod.rs -> src/models/layout/mod.rs`
- 1-file cycle: `src/models/layout/step.rs -> src/models/layout/step.rs`
- 1-file cycle: `src/models/metadata.rs -> src/models/metadata.rs`
- 1-file cycle: `src/verifylib.rs -> src/verifylib.rs`
- 1-file cycle: `src/models/link/metadata.rs -> src/models/link/metadata.rs`
- 1-file cycle: `src/models/statement/state_naive.rs -> src/models/statement/state_naive.rs`
- 1-file cycle: `src/models/statement/state_v01.rs -> src/models/statement/state_v01.rs`
- 1-file cycle: `src/models/helpers.rs -> src/models/helpers.rs`
- 1-file cycle: `src/models/predicate/link_v02.rs -> src/models/predicate/link_v02.rs`
- 1-file cycle: `src/models/layout/rule.rs -> src/models/layout/rule.rs`
- 1-file cycle: `src/models/envelope/envelope_file.rs -> src/models/envelope/envelope_file.rs`
- 1-file cycle: `src/models/predicate/slsa_provenance_v01.rs -> src/models/predicate/slsa_provenance_v01.rs`
- 1-file cycle: `src/models/predicate/slsa_provenance_v02.rs -> src/models/predicate/slsa_provenance_v02.rs`
- 1-file cycle: `src/error.rs -> src/error.rs`
- 1-file cycle: `src/models/layout/inspection.rs -> src/models/layout/inspection.rs`
- 1-file cycle: `src/models/link/byproducts.rs -> src/models/link/byproducts.rs`
- 2-file cycle: `src/models/layout/metadata.rs -> src/models/metadata.rs -> src/models/layout/metadata.rs`
- 2-file cycle: `src/models/layout/metadata.rs -> src/models/layout/mod.rs -> src/models/layout/metadata.rs`
- 2-file cycle: `src/crypto.rs -> src/models/metadata.rs -> src/crypto.rs`
- 2-file cycle: `src/crypto.rs -> src/models/layout/step.rs -> src/crypto.rs`

## Communities (42 total, 12 thin omitted)

### Community 0 - "Community 0"
Cohesion: 0.05
Nodes (39): KeyId, LayoutMetadata, LinkMetadata, ALICE_PRIVATE_KEY, ALICE_PUB_KEY, BOB_PUB_KEY, FILENAME_FORMAT, Metablock (+31 more)

### Community 1 - "Community 1"
Cohesion: 0.05
Nodes (24): calculate_hashes(), HASH_ALG_PREFS, hash_preference(), HashAlgorithm, Sha256, Sha512, Unknown, HashValue (+16 more)

### Community 2 - "Community 2"
Cohesion: 0.03
Nodes (39): compatibility_keyid_with_python_in_toto(), compatibility_rsa_verify_with_python_in_toto(), DEMO_KEY_ID, DEMO_LAYOUT, DEMO_PUBLIC_KEY, ECC_SPKI_OID, ECDSA_PK8, ECDSA_PUBLIC_KEY (+31 more)

### Community 3 - "Community 3"
Cohesion: 0.06
Nodes (25): FromMerge, StateLayout, StatementVer, Naive, V0_1, StatementWrapper, Naive, V0_1 (+17 more)

### Community 4 - "Community 4"
Cohesion: 0.12
Nodes (12): de_ser_ed25519_public_key_with_keyid_hash_algo(), de_ser_ed25519_public_key_without_keyid_hash_algo(), parse_public_ecdsa(), parse_public_ed25519(), parse_public_ed25519_with_keyid_hash_algo(), parse_public_ed25519_without_keyid_hash_algo(), Rsa, PublicKey (+4 more)

### Community 5 - "Community 5"
Cohesion: 0.13
Nodes (12): ByProducts, deserialize_byproducts_other_field(), deserialize_byproducts_other_fields(), serialize_byproducts_other_field(), serialize_byproducts_other_fields(), deserialize_auto(), deserialize_predicate(), into_trait_equal() (+4 more)

### Community 7 - "Community 7"
Cohesion: 0.11
Nodes (22): name, runs-on, steps, env, CARGO_TERM_COLOR, name, runs-on, steps (+14 more)

### Community 8 - "Community 8"
Cohesion: 0.17
Nodes (13): ecdsa_read_pkcs8_and_sign(), ed25519_read_keypair_and_sign(), ed25519_read_keypair_and_sign_with_keyid_hash_algorithms(), ed25519_read_pkcs8_and_sign(), new_ecdsa_key(), new_ed25519_key(), new_rsa_key(), PrivateKey (+5 more)

### Community 9 - "Community 9"
Cohesion: 0.11
Nodes (8): calculate_key_id(), KeyType, Ecdsa, Ed25519, Rsa, Unknown, shim_public_key(), write_spki()

### Community 10 - "Community 10"
Cohesion: 0.13
Nodes (6): ED25519_1_PRIVATE_KEY, main(), in_toto_run(), in_toto_run_record_file(), in_toto_run_record_new_file(), in_toto_run_record_symlink_file()

### Community 11 - "Community 11"
Cohesion: 0.15
Nodes (7): Signature, SignatureValue, deserialize_link(), EnvelopeFile, EnvelopeFileTuple, SERIALIZE_DATAS, serialize_link()

### Community 12 - "Community 12"
Cohesion: 0.11
Nodes (4): ArtifactRuleVisitor, deserialize_full(), deserialize_tests(), serialize_match_full()

### Community 13 - "Community 13"
Cohesion: 0.16
Nodes (6): Completeness, PREDICATE_PROVEN_V01, ProvenanceMetadata, serialize_deserialize_datetime(), STR_PREDICATE_PROVEN_V01, TimeStamp

### Community 14 - "Community 14"
Cohesion: 0.12
Nodes (17): Error, ArtifactRuleError, AttestationFormatDismatch, BadSignature, IllegalArgument, LinkGatheringError, NoSupportedHashAlgorithm, NotFound (+9 more)

### Community 15 - "Community 15"
Cohesion: 0.16
Nodes (16): canonicalize(), convert(), Number, I64, U64, Value, Array, Bool (+8 more)

### Community 16 - "Community 16"
Cohesion: 0.20
Nodes (3): Result, deserialize_virtual_target_path(), serialize_virtual_target_path()

### Community 17 - "Community 17"
Cohesion: 0.23
Nodes (10): ALICE_PUB_KEY, BOB_PUB_KEY, deserialize_layout(), format_datetime(), format_datetime_test(), get_example_layout_metadata(), Layout, parse_datetime() (+2 more)

### Community 18 - "Community 18"
Cohesion: 0.19
Nodes (8): ArtifactRule, Allow, Create, Delete, Disallow, Match, Modify, Require

### Community 19 - "Community 19"
Cohesion: 0.21
Nodes (9): ConfigSource, deserialize_auto(), deserialize_predicate(), into_trait_equal(), Invocation, PREDICATE_PROVEN_V02, serialize_predicate(), SLSAProvenanceV02 (+1 more)

### Community 20 - "Community 20"
Cohesion: 0.22
Nodes (6): deserialize_inspection(), serialize_inspection(), generate_materials_rule(), generate_products_rule(), supply_chain_item_derive!, SupplyChainItem

### Community 21 - "Community 21"
Cohesion: 0.18
Nodes (8): SignatureScheme, EcdsaP256Sha256, Ed25519, RsaSsaPssSha256, RsaSsaPssSha512, Unknown, PublicKey, PublicKeyValue

### Community 22 - "Community 22"
Cohesion: 0.16
Nodes (4): Artifact, Materials, Products, Command

### Community 23 - "Community 23"
Cohesion: 0.19
Nodes (5): PredicateVer, LinkV0_2, SLSAProvenanceV0_1, SLSAProvenanceV0_2, String

### Community 28 - "Community 28"
Cohesion: 0.21
Nodes (9): Builder, deserialize_auto(), deserialize_predicate(), into_trait_equal(), Material, Recipe, serialize_predicate(), SLSAProvenanceV01 (+1 more)

### Community 30 - "Community 30"
Cohesion: 0.24
Nodes (4): DSSEParser, EnvelopeFileTuple, SERIALIZE_SRC_DATAS, PaeV1

### Community 31 - "Community 31"
Cohesion: 0.20
Nodes (5): consume_load_len(), PREFIX, SERIALIZE_RESULT_DATAS, SPLIT, SPLIT_U8

### Community 33 - "Community 33"
Cohesion: 0.29
Nodes (3): deserialize_step(), serialize_step(), Step

### Community 34 - "Community 34"
Cohesion: 0.20
Nodes (9): include, prefix, interval, updates, commit-message, directory, package-ecosystem, schedule (+1 more)

### Community 36 - "Community 36"
Cohesion: 0.39
Nodes (4): PredicateWrapper, LinkV0_2, SLSAProvenanceV0_1, SLSAProvenanceV0_2

### Community 37 - "Community 37"
Cohesion: 0.40
Nodes (4): parse_public_ed25519_from_pem_spki(), parse_public_key_ecdsa_from_pem_spki(), parse_public_rsa_from_pem_spki(), pubkey_as_pem()

## Knowledge Gaps
- **51 isolated node(s):** `Layout`, `Link`, `Layout`, `Link`, `Sha256` (+46 more)
  These have ≤1 connection - possible missing edges. (Counts symbols only; 246 node(s) total have ≤1 connection when file, concept and rationale nodes are included.)
- **12 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `Json` connect `Community 25` to `Community 0`, `Community 1`, `Community 2`, `Community 3`, `Community 5`, `Community 6`, `Community 10`, `Community 11`, `Community 12`, `Community 13`, `Community 15`, `Community 16`, `Community 17`, `Community 19`, `Community 20`, `Community 22`, `Community 27`?**
  _High betweenness centrality (0.105) - this node is a cross-community bridge._
- **What connects `Layout`, `Link`, `Layout` to the rest of the system?**
  _51 weakly-connected nodes found - possible missing edges._
- **Should `Community 0` be split into smaller, more focused modules?**
  _Cohesion score 0.050156739811912224 - nodes in this community are weakly interconnected._
- **Why does `Error` connect `Community 14` to `Community 0`, `Community 1`, `Community 2`, `Community 3`, `Community 15`, `Community 16`, `Community 17`, `Community 22`, `Community 23`, `Community 29`, `Community 30`, `Community 31`?**
  _High betweenness centrality (0.086) - this node is a cross-community bridge._
- **Should `Community 1` be split into smaller, more focused modules?**
  _Cohesion score 0.05217391304347826 - nodes in this community are weakly interconnected._
- **Why does `Result` connect `Community 16` to `Community 0`, `Community 1`, `Community 2`, `Community 35`, `Community 3`, `Community 5`, `Community 6`, `Community 11`, `Community 13`, `Community 15`, `Community 17`, `Community 19`, `Community 22`, `Community 23`, `Community 29`, `Community 30`, `Community 31`?**
  _High betweenness centrality (0.082) - this node is a cross-community bridge._
- **Should `Community 2` be split into smaller, more focused modules?**
  _Cohesion score 0.03467954345917471 - nodes in this community are weakly interconnected._