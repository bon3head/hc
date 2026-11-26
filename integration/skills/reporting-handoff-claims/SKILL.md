---
name: reporting-handoff-claims
description: Reads the hc handoff brief at the start of a task and reports typed claims at the end so the hc verifier can check them. Use when working in a repository that has an hc protocol block in AGENTS.md or CLAUDE.md, when a prompt contains a "Handoff brief", or when asked to run `hc claim`, `hc brief` or `hc schema`.
license: MIT
---

# Reporting handoff claims

Work in an hc repository is judged by what the verifier can confirm, not by what the final message says.

## Start

1. Read the brief: `hc brief --task <task.json>`, or the "Handoff brief" already in the prompt.
2. Treat **Verified facts** as true, **Refuted claims** as false, and **Unverified** items as unknown until re-checked.

## While working

- Write only where the contract allows. Never edit tests, CI, hook or agent configuration: protected writes refute every `done` claim in the session.
- Do not special-case tests, exit early, skip tests or patch clocks. These shapes are detected on the lines you add.
- If the task cannot be done as specified, stop and report `spec_conflict`. If nothing needs changing, report `no_change_needed`. Both count as success.
- If a test fails twice with the same error, stop immediately and report the root cause instead of attempting a third fix. The verifier enforces this too: the same failure twice ends the retries and asks for the root cause.
- Keep your plan current in your harness's own plan tool (todo list, plan, `PLANS.md`). The verifier reads it: a step marked completed with no recorded change is reported, and so is work done while no step is in progress.
- Messages headed "hc verifier" are an independent check of the working tree. Treat them as evidence about the code, not as criticism of your reasoning.

## Finish

Make the final message one JSON object matching the claim schema (`hc schema`). For agents launched by `hc run` the CLI enforces the schema. Otherwise pipe the JSON to `hc claim`, then fix any diagnostics it prints and run it again until it exits 0.

Rules that decide verdicts:

- `tests_pass` must name the task's exact test command.
- When the task allows editing tests, new or changed tests must fail on the code as it was before your session. Tests that pass either way are refuted. The verifier re-runs it on the named `commit`, or on the final tree when `commit` is null.
- `files_changed` must list exactly the paths written. `files_untouched` must be true.
- Write `basis` before `status`.

Claim shapes and worked examples: [references/claims.md](references/claims.md). What each verdict rule checks: [references/verdict-rules.md](references/verdict-rules.md).
