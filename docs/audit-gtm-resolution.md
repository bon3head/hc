# Go-to-market audit: resolution

`docs/audit-gtm.md` reviewed commit `56d6818` and found it not releasable: 13 release-blocking, 10 major and 5 minor findings. Its central diagnosis was that the evidence model was per-session while the agent chooses the session.

The fix replaces that model with **repository continuity**:

- `hc init` records a signed baseline holding the accepted code and the task contract.
- Every snapshot is diffed against the repository's last recorded tree, so a change made while no session watched is attributed to the next one.
- Tamper, signature and "untouched" or "no change" checks use every write since the baseline, whatever session a claim names.
- The contract that governs the kernel and the hooks is the recorded one. The repository's task file is only a copy.

Each finding is closed by a replay in `scripts/attacks.sh` (gate criterion 7) through the shipped entry points, or is listed below with its reason. Gate criterion 8 checks this mechanically: a confirmed blocking finding with neither a replay nor an entry below fails the release.

| id | Closed by | What the receiver gets now |
|---|---|---|
| T1 | replay T1 | A commit built with plumbing is judged by its content (P6/P6S over the commit's diff from the baseline) |
| T2 | replay T2 | Tamper and scope checks run whatever status a claim reports |
| T3 | replays T3, G1 | A fresh session starts where the ledger last saw the repository; claims under any session id are refuted for recorded writes |
| T4 | replay T4 | The brief and the kernel use the recorded contract and say when the task file differs; an edited test command is refuted |
| T5 | replay T5 | Commits must be object ids; a moved ref cannot change what was judged |
| T6 | replay T6 | Staleness compares with the repository's latest snapshot, and `hc brief` snapshots the working tree |
| T7 | replay T7 | `tests_pass` needs a test report, not only exit 0 |
| T8 | replay T8 | SessionStart injects the compiled brief, never a file in the repository |
| T9 | replay T9 | MCP work done before the first call is recorded; session ids are exact |
| T10 | replay T10 | An unreadable claim is unresolved; the rest of the brief stands |
| T11 | replay T11 | `hc run` makes the repository path absolute before the sandbox sees it; a headless `.git/hooks` plant is recorded |
| T13 | replay T13, `scripts/mutants.py` | The gate's checks catch the eight defects the auditor planted (see the mutation battery for the full list) |
| T14 | replay T14 | Re-execution ignores Python's user site and startup variables; a symlink out of the tree leaves the claim unresolved |
| T15 | replay T15 | P9 runs only the new tests against the unfixed code |
| T16–T20 | replays T16–T20 | Hooks honour `tests_writable`, quote commands like the kernel, normalise paths, follow linked worktrees' common dir, and protect agent notes |
| T23 | replay T23 | Re-running `hc init` records a new baseline; the owner's change is not attributed to the agent |
| T24 | replay T24 | Tampering blocks every Stop and is never released by fail-fast; restoring the file is enough |
| T25 | replay T25 | An id used in two sessions is shown with its session |

## Closed without a replay

| id | Status | Why there is no replay |
|---|---|---|
| T12 | **Boundary, not fixed** | In hook mode the agent's shell runs as the same OS user as hc, so it can read `$HC_HOME`, including the signing keys. A same-user process cannot be kept out by any check hc runs. The README's trust model now says this plainly. `hc run` hides `$HC_HOME` behind an empty tmpfs inside bubblewrap, and that is the mode to use against an agent that attacks hc itself. Hook mode catches mistakes and the cheating patterns in the replays; it does not stop a deliberate attack on hc's own files |
| T21 | Fixed by gate criterion 8 | Criterion 8 used to grep a line written by the implementer. It now derives the finding list from the audit reports and requires a replay or an entry in this table for each one |
| T22 | Fixed in the assay | `crates/assay` builds its kernel with `Kernel::for_task` and judges with `adjudicate`, the same path as `hc brief`, and records a baseline as `hc init` does. Every gate seed exercises it |

## Residual

- **T26 (comments and docs stating guarantees the code lacks).** The README's claims were rewritten against the code: the trust model, "no checkout, hook, filter or fsmonitor runs in the agent's repository", and the mode-dependent ledger isolation. Any comment found later that promises more than the code does is a defect.
- **T27 (hooks fired inside `hc run`).** Plausible, not reproduced with a live harness. Inside the cell, `$HC_HOME` is an empty tmpfs, so hooks the launched agent fires write to a throwaway ledger. Hook-tier evidence is lost under `hc run`, and the cell's own recorder still observes the session.
- **T28 (doctor ran `git rev-parse` outside the wrapper).** Fixed: `doctor.rs` uses `Git::untrusted`.
- The minors the replays do not cover are listed in `docs/audit-gtm.md` and were reviewed. None of them changes what a receiver can rely on.
