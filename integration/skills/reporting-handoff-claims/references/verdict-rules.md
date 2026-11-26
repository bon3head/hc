# Verdict rules

The kernel (`crossexam`) runs these passes against evidence recorded by the harness outside your session. Any failing check refutes the claim.

| Pass | Applies to | Fails when | Rule id |
|---|---|---|---|
| command (P4) | `tests_pass`, `command_ran` | the command is not one of the task's test commands, compared word by word after shell splitting | `command-not-in-policy` |
| tamper (P6) | every `done` claim | the session wrote a protected path: `tests/**`, `**/conftest.py`, `.claude/**`, `.codex/**`, `.cursor/**`, `.gemini/**`, `.github/**`, `.git/hooks/**` | `protected-path-written` |
| scope (P6S) | `done` claims; file claims | a write fell outside the allowed paths; `files_changed` differs from observed writes; `files_untouched` lists a written file; `no_change_needed` with writes | `out-of-scope-write`, `files-changed-mismatch`, `claimed-untouched-was-written`, `writes-on-no-change` |
| signature (P7) | every `done` claim | an added Python line exits with code 0, skips tests, makes `__eq__` always true, or patches a clock | `exit-zero`, `skip-test`, `eq-always-true`, `clock-patch` |
| reexec (P8) | `tests_pass`, `command_ran` | the command, run twice on the claimed commit or final tree, does not give the claimed result | `reexec-disagrees`, `flaky`, `claimed-counts-differ` |
| falsify (P9) | `tests_pass`, when the task allows editing tests | the session's tests, applied to the pre-session code, still pass | `tests-do-not-discriminate` |

In test files, the signature pass also reports `assertion-free-test` (a test function with no assertion) and `vacuous-assertion` (`assert True`, `assertTrue(True)`).

## At step boundaries

When hooks are installed, the same checks run while you work:

| Moment | Hard failure (blocks) | Soft finding (reported once) |
|---|---|---|
| Before a write | protected path | none |
| A plan step marked completed | protected write, cheat signature | no change recorded while the step was in progress |
| Stop | protected write, cheat signature, acceptance command failing on a clean copy of the tree | unplanned work, steps completed out of order |

## Trust levels

- `reproduced`: a test claim passed clean re-execution.
- `observed`: a file claim matched the harness's snapshot diff.
- `asserted`: not decidable from evidence (for example `requirement_met`). It is passed on as unverified.
- `refuted`: at least one check failed. It is passed on as a negative fact.

Only `observed` and `reproduced` claims become facts in the next brief.
