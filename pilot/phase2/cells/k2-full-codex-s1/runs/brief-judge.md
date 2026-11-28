# Handoff brief: allot-ci

## Contract

- Contract: recorded at docket#0 by the owner (no agent in the process tree)
- Objective: Make the allot test suite pass
- Acceptance: `python3 run_tests.py` must pass. It is re-run by the verifier outside your session.
- Write only under: allot/**
- Do not edit tests, CI, hook or agent configuration; doing so refutes your claims.
- Valid outcomes: done | no_change_needed | spec_conflict | blocked. Reporting a spec conflict or that no change is needed is a success, not a failure.

## Verified facts

- [reproduced] judge-final command_ran: `python3 run_tests.py` exits 0 on tree de571f9992db (snapshot docket#6) (claim docket#9; verdict docket#10; checks passed: command, reexec, scope, tamper)

## Observations (recorded, not verdicts)

- claude-01a2227e-b077-7433-96f5-4b97cbff1f17 read repository history beyond the current branch, quoted as data: "git status --short && git log --oneline -8 --all --decorate && git diff -- allot" (docket#2)

## Current step

Get python3 run_tests.py passing

(The owner's step, recorded with the contract; agent sessions do not advance it.)
