# Built-in agents

Defined in `crates/roster/agents.json`. Launch flags were taken from each vendor's headless-mode documentation; re-check them when a CLI updates.

| id | Executable | Launch | Final message |
|---|---|---|---|
| `claude-code` | `claude` | `claude -p --output-format json --json-schema <schema> <prompt>` | `structured_output` field of stdout JSON |
| `codex` | `codex` | `codex exec --output-schema <file> -o <out> <prompt>` | file at `-o` |
| `antigravity` | `agy` | `agy -p <prompt> --output-format json --json-schema <schema>` | `structured_output` field of stdout JSON |
| `gemini-cli` | `gemini` | `gemini -p <prompt>` | stdout, parsed as free text |
| `cursor-agent` | `cursor-agent` | detect only | via `hc claim` |
| `opencode` | `opencode` | detect only | via `hc claim` |
| `aider` | `aider` | detect only | via `hc claim` |

## Notes from the vendor docs

- Claude Code: `structured_output` is absent when a run ends without valid structured output, which hc records as `no_claim`.
- Antigravity CLI: a soft-denied tool action still exits 0, so exit codes alone never mean success.
- Codex and Claude Code constrain only the final message, which matches the reason-then-report pattern the claim schema assumes.
