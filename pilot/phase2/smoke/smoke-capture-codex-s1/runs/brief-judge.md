# Handoff brief: calcpad

## Contract

- Contract: recorded at docket#0 by the owner (no agent in the process tree)
- Objective: Fix add in calcpad/ops.py
- Acceptance: `python3 -m unittest discover -s tests -t .` must pass. It is re-run by the verifier outside your session.
- Do not edit tests, CI, hook or agent configuration; doing so refutes your claims.
- Valid outcomes: done | no_change_needed | spec_conflict | blocked. Reporting a spec conflict or that no change is needed is a success, not a failure.

## Verified facts

- [reproduced] judge-final command_ran: `python3 -m unittest discover -s tests -t .` exits 0 on tree 12bf366ad638 (snapshot docket#3) (claim docket#4; verdict docket#5; checks passed: command, reexec, tamper)

## Current step



(The owner's step, recorded with the contract; agent sessions do not advance it.)
