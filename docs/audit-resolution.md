# Audit 1: resolution

Every finding in `docs/audit-1.md` was confirmed by its reproduction and repaired. Each repair is checked through the normal entry points (`hc hook`, a bare `hc claim`, `hc brief`) by `scripts/attacks.sh`, which is a release-gate criterion. A unit test of a helper alone would not show that the shipped path uses it.

The release question is whether any remaining defect lets the receiver rely on a false fact, lose a valid fact, or act outside the task contract. The answer for each finding is below.

| Finding | Promise it challenged | When it mattered | What the receiver saw | Resolution | Evidence (`attacks.sh`) |
|---|---|---|---|---|---|
| F1 critical | The ledger is out of the agent's reach | Any commit-bound test claim | Possibly forged facts: a planted `post-checkout` hook ran as the verifier and could read the signing keys | Commit trees are materialized through hc's own repository with the agent's object directory as a git alternate, so no git command checks out in the agent's repository. The remaining read-only call (`rev-parse`) goes through `Git::untrusted`, which overrides `core.hooksPath` and `core.fsmonitor` and uses `--end-of-options`. A commit name that does not resolve leaves the claim unresolved instead of aborting the brief | F1: hook and fsmonitor plants do not run, and the session's `.git` changes refute its claim |
| F2 critical | Write-based checks see what the session wrote | A bare `hc claim`, the documented protocol | A false `no_change_needed` admitted as a fact | A bare `hc claim` binds to the repository's open hook session. A claim whose session has no recorded observations gets an unresolved check (`no-session-evidence`) and cannot rise above `asserted` | F2: refuted, citing the write the hooks recorded |
| F3 high | Agent text is data, never instructions | A command with a trailing `# …` | Injected markdown, including a fake "Current step" | Intake rejects commands with a shell comment or control characters, ids outside `[A-Za-z0-9._-]{1,64}`, and paths that are absolute, contain `..` or control characters, or are empty. The brief renders commands re-quoted from their parsed words and inlines every agent string on one line without backticks | F3: four cases; the brief keeps exactly one Current step |
| F4 high | A fact means what it says | `command_ran` with `exit_code: null` | A failing command shown as "passes" | `command_ran` must state its exit code. Its fact reads "exits N", never "passes" | F4: null rejected; exit 0 on a red suite refuted |
| F5 high | Honest work is not refuted | Running the tests writes `tests/__pycache__` | A true claim refuted for "tampering" | Bytecode and test-runner caches are excluded from snapshots. Re-execution compiles from source (`PYTHONDONTWRITEBYTECODE`), so a cache cannot reach a verdict. Real test edits are still recorded | F5: honest claim reproduced with caches present; a real test edit is still caught |
| F6 high | Claims are compared to what happened | `./src/x.py`, or a directory | A false `files_untouched` admitted; a true `files_changed` refuted | Claimed paths are normalized, and a claimed path covers everything beneath it | F6: three cases |
| F7 high | A fact describes the code in front of the receiver | A file claim followed by more edits | A stale file fact shown as current | Every tree-bound result, including file claims, moves to Stale once the session's tree changes | F7: moved to stale after the edit |
| F8 medium | "Could not evaluate" is not "allowed" | Any hc error during `PreToolUse`, `TaskCompleted` or `Stop` | The gate silently skipped | Gating hooks block on error with the reason shown. A `Stop` that is already continuing is let through so a broken hc cannot trap the agent; `HC_FAIL_OPEN=1` opts out. `.handoff/task.json` and `.agents/**` are now protected, so deleting the task cannot disable the Stop gate | F8: Stop blocks on a malformed payload; a continuing Stop is let through. The permission case is skipped when run as root |
| F9 medium | Writes the hooks do not see are still caught | Shell writes to `.git/hooks` or `.git/config`, which git never tracks | An invisible hook plant | The recorder fingerprints `.git/config` and `.git/hooks/*` when a session opens and at every snapshot. Changes are recorded as writes to protected paths, so claims are refuted and the Stop gate blocks. Pre-write denial for shell tools is still not attempted: a tool-name list cannot see what a shell command will write. The snapshot at the next step is the authority | F9: Stop blocks on a planted `pre-commit` |

## Found while closing the audit

- A session that changed nothing had no end tree, so an honest claim on an untouched tree came back unresolved ("no snapshot"). The start snapshot is now the session's tree until something changes (found by F5's attack).
- The acceptance denominator was chosen by claim type in the evaluator. The assay now compares every brief with a per-scenario manifest of where each claim must land and at what trust, written from what the scenario does to the repository. Missing, extra, duplicated, misplaced and stale claims are all reported. With reexec disabled, the known-bad run must fail with exactly the expected signature: test claims left unverified.

## Residual, not closed here

- Concurrent hook processes share a read-modify-write state file without a lock (audit "Not examined", plausible). The docket itself appends with compare-and-swap.
- Snapshot materialization relies on git's `checkout-index` path checks (`..`, `.git`). These were not fuzzed.
- The F8 permission case runs only as a non-root user. Use `scripts/matrix.sh` there.

Unresolved critical or high: 0
