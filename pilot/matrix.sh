#!/usr/bin/env bash
# Run one named cell of the phase-2 matrix by composing pilot/run.sh.
#
#   pilot/matrix.sh <canary> <arm> <agent> <seed>      e.g. pilot/matrix.sh k1 full claude 1
#   pilot/matrix.sh baseline full claude 1             the two-session ledgerlite pilot
#
# The cell detaches itself first (setsid, scrubbed environment), so neither the
# caller nor any agent that launched it is in the subjects' process tree. The
# caller waits for $OUT/<cell>/done. Idempotent: a cell whose score.json is
# valid is skipped. A session that is invalid (non-zero exit or no tool calls)
# is retried once from a fresh workspace; a second failure stays invalid.
#
# Workspaces live under ~/hcw/<opaque id>/ so that no path a subject sees names
# the canary, arm or experiment. Results are copied to pilot/phase2/cells/<cell>/.
set -euo pipefail

HERE=$(cd "$(dirname "$0")" && pwd)
OUT=${MATRIX_OUT:-$HERE/phase2/cells}
[ $# -eq 4 ] || { sed -n '2,15p' "$0"; exit 2; }
canary=$1 arm=$2 agent=$3 seed=$4
cell="$canary-$arm-$agent-s$seed"
dest=$OUT/$cell

if [ -z "${HC_MATRIX_DETACHED:-}" ]; then
    mkdir -p "$dest"
    rm -f "$dest/done"
    setsid -f env -i HOME="$HOME" USER="$USER" PATH="$PATH" LANG=C.UTF-8 HC_MATRIX_DETACHED=1 \
        ${MATRIX_OUT:+MATRIX_OUT="$MATRIX_OUT"} bash "$0" "$@" >"$dest/matrix.log" 2>&1 </dev/null
    echo "matrix: $cell launched detached; wait for $dest/done"
    exit 0
fi

finish() { echo "$1" >"$dest/done"; }
trap 'finish "error $?"' ERR

if python3 -c 'import json,sys; d=json.load(open(sys.argv[1])); sys.exit(0 if d.get("valid") else 1)' "$dest/score.json" 2>/dev/null; then
    echo "matrix: $cell already scored; skipping"
    finish skipped
    exit 0
fi

case $arm in full|capture) ;; *) echo "arm must be full or capture" >&2; finish "error arm"; exit 2 ;; esac
case $agent in claude|codex) ;; *) echo "agent must be claude or codex" >&2; finish "error agent"; exit 2 ;; esac

if [ "$canary" = baseline ]; then
    export FIXTURE=$HERE/fixture REPO_NAME=ledgerlite GRADER=$HERE/score.py
    unset PROMPT_A PROMPT_B SEED_HOOK OBJECTIVE || true
    phases=(setup a oob b judge)
    export AGENT_A=$agent AGENT_B=$agent
else
    C=$HERE/canaries/$canary
    [ -d "$C/fixture" ] || { echo "no canary $canary" >&2; finish "error canary"; exit 2; }
    # shellcheck disable=SC1091
    source "$C/cell.env"   # REPO_NAME, OBJECTIVE
    export FIXTURE=$C/fixture PROMPT_A=$C/prompt.md GRADER=$C/key/grade.py REPO_NAME OBJECTIVE
    [ -e "$C/seed-history.sh" ] && export SEED_HOOK=$C/seed-history.sh
    phases=(setup a judge)
    export AGENT_A=$agent
fi
export ARM=$arm SKIP_CHECK=1

id=$(printf '%s' "$cell" | sha1sum | cut -c1-10)
attempt_run() { # workspace
    export PILOT_DIR=$1
    rm -rf "$PILOT_DIR"
    for p in "${phases[@]}"; do
        "$HERE/run.sh" "$p" || return 1
    done
}

ws=$HOME/hcw/$id
rc=0
attempt_run "$ws" || rc=$?
retried=no
if [ $rc -ne 0 ] && ls "$ws"/runs/*.invalid >/dev/null 2>&1 && [ ! -e "$ws/runs/setup.invalid" ]; then
    echo "matrix: $cell session invalid; retrying once"
    rm -rf "$ws.attempt1"; mv "$ws" "$ws.attempt1"
    retried=yes
    rc=0
    attempt_run "$ws" || rc=$?
fi

PILOT_DIR=$ws "$HERE/run.sh" score >/dev/null 2>"$ws/runs/score.stderr" || true

# Cost of every Claude session in this cell (Codex reports tokens only).
python3 - "$ws/runs" "$agent" "$retried" "$ws.attempt1/runs" >"$ws/runs/cost.json" <<'PY'
import json, sys, pathlib
runs, agent, retried, first = pathlib.Path(sys.argv[1]), sys.argv[2], sys.argv[3], pathlib.Path(sys.argv[4])
def cost(d):
    usd, tokens = 0.0, {}
    for t in sorted(d.glob("*.jsonl")) if d.exists() else []:
        for line in t.read_text(errors="replace").splitlines():
            try: e = json.loads(line)
            except json.JSONDecodeError: continue
            if e.get("type") == "result" and isinstance(e.get("total_cost_usd"), (int, float)):
                usd += e["total_cost_usd"]
            if e.get("type") == "turn.completed":
                for k, v in (e.get("usage") or {}).items():
                    tokens[k] = tokens.get(k, 0) + v
    return usd, tokens
u, t = cost(runs)
u1, t1 = cost(first) if retried == "yes" else (0.0, {})
print(json.dumps({"agent": agent, "usd": round(u + u1, 4) if agent == "claude" else None,
                  "codex_tokens": t if agent == "codex" else None, "retried": retried,
                  "first_attempt_usd": round(u1, 4) if retried == "yes" else None}))
PY

mkdir -p "$dest"
rm -rf "$dest/runs" "$dest/runs.attempt1"
cp -r "$ws/runs" "$dest/runs"
[ -d "$ws.attempt1/runs" ] && cp -r "$ws.attempt1/runs" "$dest/runs.attempt1"
echo "$ws" >"$dest/workspace.txt"
[ -e "$ws/runs/score.json" ] && cp "$ws/runs/score.json" "$dest/score.json"
[ -e "$ws/runs/env.txt" ] && cp "$ws/runs/env.txt" "$dest/env.txt"
finish "ok rc=$rc retried=$retried"
