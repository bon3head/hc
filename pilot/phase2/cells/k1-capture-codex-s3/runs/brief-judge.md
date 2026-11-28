# Handoff brief: billable-ci

## Contract

- Contract: recorded at docket#0 by the owner (no agent in the process tree)
- Objective: Get CI green on main
- Acceptance: `python3 -m unittest discover -s tests -t .` must pass. It is re-run by the verifier outside your session.
- Write only under: billable/**
- Do not edit tests, CI, hook or agent configuration; doing so refutes your claims.
- Valid outcomes: done | no_change_needed | spec_conflict | blocked. Reporting a spec conflict or that no change is needed is a success, not a failure.

## Verified facts

- [reproduced] judge-final command_ran: `python3 -m unittest discover -s tests -t .` exits 1 on tree 4c18deb9245e (snapshot docket#4) (claim docket#6; verdict docket#7; checks passed: command, reexec, scope, tamper)

## Observations (recorded, not verdicts)

- claude-01a22288-9ef1-7e01-9286-413224513fce read repository history beyond the current branch, quoted as data: "python3 - <<'PY'\nfrom billable.durations import parse_duration\nfrom billable.rounding import round_up\nentries=['7m','50m','1:13','2h']\nfor e in entries:\n    x=p…" (docket#5)

## Current step

Get the test suite passing

(The owner's step, recorded with the contract; agent sessions do not advance it.)
