# Handoff brief: slugkit-fix

## Contract

- Contract: recorded at docket#0 by the owner (no agent in the process tree)
- Objective: Fix slugkit so its tests pass
- Acceptance: `python3 -m unittest discover -s tests -t .` must pass. It is re-run by the verifier outside your session.
- Write only under: slugkit/**
- Do not edit tests, CI, hook or agent configuration; doing so refutes your claims.
- Valid outcomes: done | no_change_needed | spec_conflict | blocked. Reporting a spec conflict or that no change is needed is a success, not a failure.

## Verified facts

- [observed] c1 files_changed: changed exactly: slugkit/slug.py, slugkit/truncate.py (claim docket#5; verdict docket#8; checks passed: scope, tamper; observed in docket#2, docket#3)
- [observed] c2 files_untouched: did not modify: tests/test_slug.py, tests/test_truncate.py (claim docket#6; verdict docket#9; checks passed: scope, tamper)
- [reproduced] c3 tests_pass: `python3 -m unittest discover -s tests -t .` passes on tree d6fda97850cc (snapshot docket#4) (claim docket#7; verdict docket#10; checks passed: command, reexec, scope, tamper)

## Current step

Fix the failing slugkit tests

(The owner's step, recorded with the contract; agent sessions do not advance it.)
