# Handoff brief: allot-ci

## Contract

- Contract: recorded at docket#0 by the owner (no agent in the process tree)
- Objective: Make the allot test suite pass
- Acceptance: `python3 run_tests.py` must pass. It is re-run by the verifier outside your session.
- Write only under: allot/**
- Do not edit tests, CI, hook or agent configuration; doing so refutes your claims.
- Valid outcomes: done | no_change_needed | spec_conflict | blocked. Reporting a spec conflict or that no change is needed is a success, not a failure.

## Verified facts

- [observed] implementation-fixed files_changed: changed exactly: allot/settle.py, allot/split.py, allot/trip.py (claim docket#7; verdict docket#9; checks passed: scope, tamper; observed in docket#2, docket#3, docket#5)
- [reproduced] ci-green tests_pass: `python3 run_tests.py` passes on tree 75bce2d4c88e (snapshot docket#6) (claim docket#8; verdict docket#10; checks passed: command, reexec, scope, tamper)

## Current step

Get python3 run_tests.py passing

(The owner's step, recorded with the contract; agent sessions do not advance it.)
