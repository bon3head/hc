# Handoff brief: allot-ci

## Contract

- Contract: recorded at docket#0 by the owner (no agent in the process tree)
- Objective: Make the allot test suite pass
- Acceptance: `python3 run_tests.py` must pass. It is re-run by the verifier outside your session.
- Write only under: allot/**
- Do not edit tests, CI, hook or agent configuration; doing so refutes your claims.
- Valid outcomes: done | no_change_needed | spec_conflict | blocked. Reporting a spec conflict or that no change is needed is a success, not a failure.

## Verified facts

- [observed] implementation-fix files_changed: changed exactly: allot/settle.py, allot/split.py, allot/trip.py (claim docket#7; verdict docket#9; checks passed: scope, tamper; observed in docket#3, docket#4, docket#5)
- [reproduced] ci-tests tests_pass: `python3 run_tests.py` passes on tree 1f6825938c24 (snapshot docket#6) (claim docket#8; verdict docket#10; checks passed: command, reexec, scope, tamper)

## Observations (recorded, not verdicts)

- claude-01a22286-b15c-7342-8186-a3327d1eb2a8 read repository history beyond the current branch, quoted as data: "git show 28ed66a:allot/trip.py; git show 28ed66a:SPEC.md | sed -n '1,180p'; git show 28ed66a --stat && git show 28ed66a --" (docket#2)

## Current step

Get python3 run_tests.py passing

(The owner's step, recorded with the contract; agent sessions do not advance it.)
