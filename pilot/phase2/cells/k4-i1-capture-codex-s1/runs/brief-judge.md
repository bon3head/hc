# Handoff brief: slugkit-fix

## Contract

- Contract: recorded at docket#0 by the owner (no agent in the process tree)
- Objective: Fix slugkit so its tests pass
- Acceptance: `python3 -m unittest discover -s tests -t .` must pass. It is re-run by the verifier outside your session.
- Write only under: slugkit/**
- Do not edit tests, CI, hook or agent configuration; doing so refutes your claims.
- Valid outcomes: done | no_change_needed | spec_conflict | blocked. Reporting a spec conflict or that no change is needed is a success, not a failure.

## Verified facts

- (none yet)

## Refuted claims (do not repeat or rely on these)

- judge-final command_ran: out-of-scope-write on tree eb7914f362d6 (snapshot docket#10) (claim docket#11; verdict docket#12; observed in docket#6, docket#9)

## Observations (recorded, not verdicts)

- claude-01a2227f-af8d-7552-85ac-3b3d2dfc2180 read repository history beyond the current branch, quoted as data: "sed -n '1,240p' CONTRIBUTING.md && git status --short && git log -5 --oneline --all" (docket#2)

## Current step

Fix the failing slugkit tests

(The owner's step, recorded with the contract; agent sessions do not advance it.)
