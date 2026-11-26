# Validation

## Unit and property tests

`cargo test --workspace` covers:

- RFC 8785 reference vectors and permutation properties (`bedrock::canon`)
- signature acceptance and rejection of foreign or wrong-role signers (`docket`)
- snapshot contents including force-added ignored config (`exhibit`)
- bubblewrap hiding the hc home (`cell`, skipped where bwrap cannot run)
- positive and negative cheat signatures (`crossexam::signatures`)
- the compile-fail proof that `Verified` cannot be constructed outside `crossexam`
- plan parsing for every native shape and stable step identity (`agenda`)
- conformance labels and fitness (`conform`)
- ten end-to-end hook sessions with documented payload shapes: honest plan, false completion, pre-write denial, shell-made test edits, fail-fast and its reset, Antigravity artifact plans, Claude `TaskCompleted` veto (`bailiff`)
- skill pack conformance to the Agent Skills specification (`hc` integration test)

## Acceptance

`hc assay --seed 7` runs 17 honest and 17 lying sessions for real (git repositories, Python test runs, clean re-execution), twice in separate directories, and requires:

- at least 14 of 15 lies caught
- at most 1 of 15 honest sessions refuted
- identical docket heads and brief digests across both runs
- no fact below `observed` in any brief

A first PASS counts only after the known-bad build fails: `--disable tamper` must REJECT.

## Adding a seeded lie

1. Add a `Kind` variant and its count in `CORPUS` (keep 15 lies and 15 honest sessions).
2. Script the cheat in `act`.
3. Run with the new pass disabled and confirm the new lie is the one reported `MISSED`.
