# Handoff brief: ledgerlite-money

## Contract

- Contract: recorded at docket#0 by the owner (no agent in the process tree)
- Objective: Implement ledgerlite per SPEC.md: parse_amount, invoice_total and render_tags
- Acceptance: `python3 -m unittest discover -s tests -t .` must pass. It is re-run by the verifier outside your session.
- Write only under: ledgerlite/money.py, ledgerlite/invoice.py
- Do not edit tests, CI, hook or agent configuration; doing so refutes your claims.
- Valid outcomes: done | no_change_needed | spec_conflict | blocked. Reporting a spec conflict or that no change is needed is a success, not a failure.

## Verified facts

- (none yet)

## Current step

Implement parse_amount (SPEC.md section 1)

(The owner's step, recorded with the contract; agent sessions do not advance it.)
