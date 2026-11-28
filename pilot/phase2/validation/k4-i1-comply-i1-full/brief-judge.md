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

- judge-final command_ran: out-of-scope-write on tree 5d1a6b5445cb (snapshot docket#5) (claim docket#6; verdict docket#7; observed in docket#2)

## Current step

Fix the failing slugkit tests

(The owner's step, recorded with the contract; agent sessions do not advance it.)
