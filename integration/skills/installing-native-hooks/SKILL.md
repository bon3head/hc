---
name: installing-native-hooks
description: Installs and checks hc's verification hooks in Claude Code, Codex and Antigravity so plan steps and stops are verified through each harness's own hook system. Use when setting up hc in a repository, when hooks do not seem to fire, or when adding a task file for hook-time checks.
license: MIT
---

# Installing native hooks

1. Add the task file the hooks read, `.handoff/task.json`:

   ```json
   {"id": "fix-parser", "objective": "...", "test_command": "cargo test", "allowed_paths": ["src/**"], "next_step": "...", "step_tests": false, "tests_writable": false}
   ```

2. Install: `hc onboard --test-command "cargo test" --hooks` (or `--hooks claude,codex`). This merges hc's entries into `.claude/settings.json`, `.codex/hooks.json` and `.agents/hooks.json`, keeping any hooks already there. Running it again changes nothing.
3. Approve where the harness asks: Codex requires reviewing new hooks in `/hooks` (trust is tied to the hook's hash).
4. Make the plan visible. On Claude Code models where the task tools are off by default, start sessions with `CLAUDE_CODE_ENABLE_TODO_TOOLS=1`; on Codex, enable `update_plan` if your configuration disabled it. Without a native plan, hooks still deny protected writes and gate Stop; only step checks are skipped.
5. Check: start a session, ask the agent to edit a test file, and confirm the write is denied with an "hc verifier" message.

`hc hook` fails open on its own errors (reported on stderr) so an hc bug never blocks work. Set `HC_FAIL_CLOSED=1` in the harness environment to block instead.

What each harness supports and why: [references/harness-matrix.md](references/harness-matrix.md).
