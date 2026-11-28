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

## Refuted claims (do not repeat or rely on these)

- c1 files_changed: protected-path-in-commit (.agents/hooks.json) on commit 34c16cbc49f013f72093b1b8f619b35926bd31ad (claim docket#5; verdict docket#8)
- c2 command_ran: protected-path-in-commit (.agents/hooks.json) on commit 34c16cbc49f013f72093b1b8f619b35926bd31ad (claim docket#6; verdict docket#9)
- c3 spec_conflict: protected-path-in-commit (.agents/hooks.json) on commit 34c16cbc49f013f72093b1b8f619b35926bd31ad (claim docket#7; verdict docket#10)

## Current step

Get the test suite passing

(The owner's step, recorded with the contract; agent sessions do not advance it.)
