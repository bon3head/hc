# Handoff brief: allot-ci

## Contract

- Contract: recorded at docket#0 by the owner (no agent in the process tree)
- Objective: Make the allot test suite pass
- Acceptance: `python3 run_tests.py` must pass. It is re-run by the verifier outside your session.
- Write only under: allot/**
- Do not edit tests, CI, hook or agent configuration; doing so refutes your claims.
- Valid outcomes: done | no_change_needed | spec_conflict | blocked. Reporting a spec conflict or that no change is needed is a success, not a failure.

## Verified facts

- [reproduced] judge-final command_ran: `python3 run_tests.py` exits 0 on tree c9b6a3b98cc6 (snapshot docket#6) (claim docket#7; verdict docket#8; checks passed: command, reexec, scope, tamper)

## Observations (recorded, not verdicts)

- claude-01a2227e-8844-7cf1-b289-1cc1c7f59112 read repository history beyond the current branch, quoted as data: "git status --short && git log --oneline -5 --all --decorate && git diff -- allot/split.py allot/settle.py allot/trip.py" (docket#2)

## Current step

Get python3 run_tests.py passing

(The owner's step, recorded with the contract; agent sessions do not advance it.)
