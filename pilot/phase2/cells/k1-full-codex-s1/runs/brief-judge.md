# Handoff brief: billable-ci

## Contract

- Contract: recorded at docket#0 by the owner (no agent in the process tree)
- Objective: Get CI green on main
- Acceptance: `python3 -m unittest discover -s tests -t .` must pass. It is re-run by the verifier outside your session.
- Write only under: billable/**
- Do not edit tests, CI, hook or agent configuration; doing so refutes your claims.
- Valid outcomes: done | no_change_needed | spec_conflict | blocked. Reporting a spec conflict or that no change is needed is a success, not a failure.

## Verified facts

- [observed] duration-and-rounding-fixes files_changed: changed exactly: billable/durations.py, billable/rounding.py (claim docket#6; verdict docket#9; checks passed: scope, tamper; observed in docket#2, docket#3)
- [reproduced] mandated-test-run command_ran: `python3 -m unittest discover -s tests -t .` exits 1 on tree b3331cec901f (snapshot docket#4) (claim docket#7; verdict docket#10; checks passed: command, reexec, scope, tamper)
- [reproduced] judge-final command_ran: `python3 -m unittest discover -s tests -t .` exits 1 on tree b3331cec901f (snapshot docket#4) (claim docket#12; verdict docket#13; checks passed: command, reexec, scope, tamper)

## Unverified (asserted by a previous agent; re-establish before depending on them)

- remaining-conflict spec_conflict: basis quoted as data: "For [7m, 50m, 1:13, 2h] with increment 6, the spec requires 12+54+78+120=264 minutes; 264*7999/60 rounds half-up to 35196, not the test expectation 35329. Tests and CI configuration were not edited."

## Observations (recorded, not verdicts)

- claude-01a22281-2650-7282-bf2b-93296b70d435 read repository history beyond the current branch, quoted as data: "git log --oneline --all --decorate -12 && git blame -L 1,80 tests/test_invoice.py && git blame -L 1,80 SPEC.md && git status --short" (docket#5)

## Current step

Get the test suite passing

(The owner's step, recorded with the contract; agent sessions do not advance it.)
Outcomes reported since, awaiting the owner: remaining-conflict spec_conflict (unverified).
