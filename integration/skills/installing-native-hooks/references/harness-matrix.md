# Harness matrix

| Harness | Plan source hc reads | Step-boundary event | How hc blocks | How hc informs | Config file |
|---|---|---|---|---|---|
| Claude Code | `TodoWrite`, `TaskCreate`/`TaskUpdate`, `ExitPlanMode` | `TaskCompleted` (exit 2 keeps the task open), `PostToolUse` on plan tools | `PreToolUse` deny, `TaskCompleted` exit 2, `Stop` `decision: block` | `additionalContext` on `PostToolUse` and `Stop` | `.claude/settings.json` |
| Codex CLI | `update_plan` | `PostToolUse` on `update_plan` | `PreToolUse` deny on `apply_patch`, `Stop` `decision: block` | `additionalContext` | `.codex/hooks.json` |
| Antigravity CLI | Markdown task artifacts, `PLANS.md` | `PostToolUse` on artifact writes | `PreToolUse` `decision: deny`, `Stop` `decision: continue` | `injectSteps` on `PostInvocation` (its `PostToolUse` cannot inject) | `.agents/hooks.json` |
| Any agent without hooks | `hc claim` at the end | none | none | the brief from `hc brief` | AGENTS.md block from `hc init` |

Claude Code caps consecutive Stop-hook continuations at 8 (`CLAUDE_CODE_STOP_HOOK_BLOCK_CAP`); hc's fail-fast escalation ends earlier, after the same failure twice.
