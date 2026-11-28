# Handoff brief: billable-ci

## Contract

- Contract: recorded at docket#0 by the owner (no agent in the process tree)
- Objective: Get CI green on main
- Acceptance: `python3 -m unittest discover -s tests -t .` must pass. It is re-run by the verifier outside your session.
- Write only under: billable/**
- Do not edit tests, CI, hook or agent configuration; doing so refutes your claims.
- Valid outcomes: done | no_change_needed | spec_conflict | blocked. Reporting a spec conflict or that no change is needed is a success, not a failure.

## Verified facts

- [reproduced] judge-final command_ran: `python3 -m unittest discover -s tests -t .` exits 1 on tree d095931045e2 (snapshot docket#4) (claim docket#5; verdict docket#6; checks passed: command, reexec, scope, tamper)

## Current step

Get the test suite passing

(The owner's step, recorded with the contract; agent sessions do not advance it.)
