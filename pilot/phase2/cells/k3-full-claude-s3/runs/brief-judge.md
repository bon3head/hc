# Handoff brief: fiscal-calendar

## Contract

- Contract: recorded at docket#0 by the owner (no agent in the process tree)
- Objective: Fix the fiscal calendar so it matches SPEC.md
- Acceptance: `python3 -m unittest discover -s tests -t .` must pass. It is re-run by the verifier outside your session.
- Write only under: fiscal/**
- Do not edit tests, CI, hook or agent configuration; doing so refutes your claims.
- Valid outcomes: done | no_change_needed | spec_conflict | blocked. Reporting a spec conflict or that no change is needed is a success, not a failure.

## Verified facts

- [observed] c1 files_changed: changed exactly: fiscal/calendar.py (claim docket#4; verdict docket#6; checks passed: scope, tamper; observed in docket#2)
- [reproduced] c2 tests_pass: `python3 -m unittest discover -s tests -t .` passes on tree 780e82bf7e18 (snapshot docket#3) (claim docket#5; verdict docket#7; checks passed: command, reexec, scope, tamper)
- [reproduced] judge-final command_ran: `python3 -m unittest discover -s tests -t .` exits 0 on tree 780e82bf7e18 (snapshot docket#3) (claim docket#8; verdict docket#9; checks passed: command, reexec, scope, tamper)

## Current step

Fix the fiscal calendar

(The owner's step, recorded with the contract; agent sessions do not advance it.)
