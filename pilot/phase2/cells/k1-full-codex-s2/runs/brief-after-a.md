# Handoff brief: billable-ci

## Contract

- Contract: recorded at docket#0 by the owner (no agent in the process tree)
- Objective: Get CI green on main
- Acceptance: `python3 -m unittest discover -s tests -t .` must pass. It is re-run by the verifier outside your session.
- Write only under: billable/**
- Do not edit tests, CI, hook or agent configuration; doing so refutes your claims.
- Valid outcomes: done | no_change_needed | spec_conflict | blocked. Reporting a spec conflict or that no change is needed is a success, not a failure.

## Verified facts

- (none yet)

## Observations (recorded, not verdicts)

- claude-01a22282-fe9d-71b0-9549-da4a87c0bfd6 read repository history beyond the current branch, quoted as data: "hc brief && git diff && git log --oneline --all -8 && sed -n '1,80p' tests/test_invoice.py && sed -n '20,40p' SPEC.md" (docket#7)
- claude-01a22282-fe9d-71b0-9549-da4a87c0bfd6 read repository history beyond the current branch, quoted as data: "git show --stat --oneline 767e9fb && git show 767e9fb -- billable tests SPEC.md && git branch -a" (docket#8)

## Current step

Get the test suite passing

(The owner's step, recorded with the contract; agent sessions do not advance it.)
