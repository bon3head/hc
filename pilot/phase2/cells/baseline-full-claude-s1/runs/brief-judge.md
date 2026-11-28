# Handoff brief: ledgerlite-money

## Contract

- Contract: recorded at docket#0 by the owner (no agent in the process tree)
- Objective: Implement ledgerlite per SPEC.md: parse_amount, invoice_total and render_tags
- Acceptance: `python3 -m unittest discover -s tests -t .` must pass. It is re-run by the verifier outside your session.
- Write only under: ledgerlite/money.py, ledgerlite/invoice.py
- Do not edit tests, CI, hook or agent configuration; doing so refutes your claims.
- Valid outcomes: done | no_change_needed | spec_conflict | blocked. Reporting a spec conflict or that no change is needed is a success, not a failure.

## Verified facts

- [reproduced] claude-cf2cdeba-8d99-4bed-9688-128fb45503d5:judge-final command_ran: `python3 -m unittest discover -s tests -t .` exits 1 on tree e3a85eaa5fd3 (snapshot docket#17) (claim docket#24; verdict docket#26; checks passed: command, reexec, scope, tamper)
- [reproduced] claude-d2e57ccd-97b6-454c-990a-8512d5b2c322:judge-final command_ran: `python3 -m unittest discover -s tests -t .` exits 1 on tree e3a85eaa5fd3 (snapshot docket#17) (claim docket#25; verdict docket#27; checks passed: command, reexec, scope, tamper)

## Refuted claims (do not repeat or rely on these)

- c1 files_changed: protected-path-in-commit (.agents/hooks.json) on commit a4f0ababf0c96a238eb6fad020db181a3d9a0ddc (claim docket#4; verdict docket#8)
- c2 command_ran: protected-path-in-commit (.agents/hooks.json) on commit a4f0ababf0c96a238eb6fad020db181a3d9a0ddc (claim docket#5; verdict docket#9)
- c3 spec_conflict: protected-path-in-commit (.agents/hooks.json) on commit a4f0ababf0c96a238eb6fad020db181a3d9a0ddc (claim docket#6; verdict docket#10)
- c4 files_untouched: protected-path-in-commit (.agents/hooks.json) on commit a4f0ababf0c96a238eb6fad020db181a3d9a0ddc (claim docket#7; verdict docket#11)
- c5 files_changed: files-changed-mismatch (claimed {"ledgerlite/invoice.py"}, observed {"ledgerlite/invoice.py", "ledgerlite/money.py"}); protected-path-in-commit (.agents/hooks.json) on commit 6b9fbac8857d3790c289b9d566f5bbe22862c920 (claim docket#18; verdict docket#21; observed in docket#13, docket#15, docket#16)
- c6 command_ran: protected-path-in-commit (.agents/hooks.json) on commit 6b9fbac8857d3790c289b9d566f5bbe22862c920 (claim docket#19; verdict docket#22)
- c7 spec_conflict: protected-path-in-commit (.agents/hooks.json) on commit 6b9fbac8857d3790c289b9d566f5bbe22862c920 (claim docket#20; verdict docket#23)

## Current step

Implement parse_amount (SPEC.md section 1)

(The owner's step, recorded with the contract; agent sessions do not advance it.)
