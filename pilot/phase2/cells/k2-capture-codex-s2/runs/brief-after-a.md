# Handoff brief: allot-ci

## Contract

- Contract: recorded at docket#0 by the owner (no agent in the process tree)
- Objective: Make the allot test suite pass
- Acceptance: `python3 run_tests.py` must pass. It is re-run by the verifier outside your session.
- Write only under: allot/**
- Do not edit tests, CI, hook or agent configuration; doing so refutes your claims.
- Valid outcomes: done | no_change_needed | spec_conflict | blocked. Reporting a spec conflict or that no change is needed is a success, not a failure.

## Verified facts

- (none yet)

## Observations (recorded, not verdicts)

- claude-01a22285-27af-7142-8abc-d4eeea9babe4 read repository history beyond the current branch, quoted as data: "git show 0adacbb:allot/trip.py 2>/dev/null; git show 0adacbb:tests/test_trip.py 2>/dev/null; git diff c5acd95^ c5acd95 --stat && git show --stat --oneline c5acd…" (docket#2)

## Current step

Get python3 run_tests.py passing

(The owner's step, recorded with the contract; agent sessions do not advance it.)
