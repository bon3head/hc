---
name: installing-native-hooks
description: Installs and checks hc's verification hooks in Claude Code, Codex and Antigravity so plan steps and stops are verified through each harness's own hook system. Use when setting up hc in a repository, when hooks do not seem to fire, or when adding a task file for hook-time checks.
license: MIT
---

# Installing native hooks

1. Run `hc init "<objective>"` in the repository (or `hc init --for claude,codex` to choose harnesses). It writes `.handoff/task.json` with the detected test command, installs hooks for every harness found on PATH or already configured in the repository, and merges hc's entries into `.claude/settings.json`, `.codex/hooks.json` and `.agents/hooks.json`, keeping any hooks already there. Running it again changes nothing except a new objective.
2. Edit `.handoff/task.json` for anything beyond the objective and test command: `allowed_paths`, `constraints`, `step_tests`, `tests_writable`.
3. Approve where the harness asks: Codex requires reviewing new hooks in `/hooks` (trust is tied to the hook's hash).
4. Make the plan visible. On Claude Code models where the task tools are off by default, start sessions with `CLAUDE_CODE_ENABLE_TODO_TOOLS=1`; on Codex, enable `update_plan` if your configuration disabled it. Without a native plan, hooks still deny protected writes and gate Stop; only step checks are skipped.
5. Check: `hc doctor` must report every installed harness's hooks as resolving to an hc binary. Then start a session, ask the agent to edit a test file, and confirm the write is denied with an "hc verifier" message.

When hc itself fails, the gating hooks (`PreToolUse`, `TaskCompleted`, `Stop`) block with the error on stderr: "could not evaluate" never reads as "allowed". A `Stop` that is already continuing is let through so a broken hc cannot trap the agent; other events only report the error. Set `HC_FAIL_OPEN=1` in the harness environment to allow on error everywhere. `hc doctor` diagnoses the cause.

What each harness supports and why: [references/harness-matrix.md](references/harness-matrix.md).
