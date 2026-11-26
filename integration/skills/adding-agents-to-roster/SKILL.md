---
name: adding-agents-to-roster
description: Registers a new coding agent with hc by adding a data entry to the roster, so hc can detect it from the process tree, generate its signing identity, launch it headless and read its final structured message. Use when an agent is missing from `hc agents`, when wiring a local model or a new CLI into hc, or when overriding a built-in launch template.
license: MIT
---

# Adding agents to the roster

Agents are data, not code. Add an entry to `$HC_HOME/roster.json` (a JSON array). An entry with the same `id` as a built-in one replaces it.

```json
[{
  "id": "neon",
  "vendor": "local",
  "executables": ["neon"],
  "instructions_file": "AGENTS.md",
  "launch": {
    "argv": ["neon", "run", "--schema", "{schema_path}", "--out", "{out_path}", "{prompt}"],
    "output": {"kind": "file"}
  }
}]
```

- `id`: lowercase letters, digits, `.`, `_`, `-`. Becomes the signing principal `agent:<id>@hc`; its key is generated on first use.
- `executables`: binary names matched against the process tree for `hc claim`. Interpreter-wrapped CLIs (`node cli.js`, `python -m tool`) match on the script name.
- `instructions_file`: where `hc onboard` writes the protocol block.
- `launch`: `null` for detect-only agents. Placeholders: `{prompt}`, `{schema_path}`, `{schema_json}`, `{out_path}`.
- `output.kind`: `stdout_json_field` (with `field`), `file` (reads `{out_path}`) or `stdout` (parsed as free text).

When the CLI has a JSON Schema flag, pass `{schema_path}` or `{schema_json}` so the schema is enforced at decode time. Otherwise the prompt embeds the schema and the front end recovers the JSON.

## Check it

1. `hc agents` lists the entry as `launchable` or `detect-only`.
2. Run the CLI by hand with the rendered argv and confirm the output lands where `output` says.
3. `hc run --task <task.json> --agent <id>` and confirm the first attempt records claims rather than `no_claim`.

Built-in entries and their sources: [references/builtin-agents.md](references/builtin-agents.md).
