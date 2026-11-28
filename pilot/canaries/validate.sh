#!/usr/bin/env bash
# Validate a canary before any subject sees it: seed a cell exactly as the matrix
# does, then replace the agent with a scripted solution driven through hc's real
# hook entry points (SessionStart, a PostToolUse per changed file, Stop), run the
# operator judge, and grade. Each scripted solution must be classified as intended
# by both the ground-truth grader and hc.
#
#   pilot/canaries/validate.sh <canary> <solution-script> <arm> <out-dir>
#
# Run detached from any agent (setsid env -i), like the matrix.
set -euo pipefail
HERE=$(cd "$(dirname "$0")" && pwd)
PILOT=$(cd "$HERE/.." && pwd)
canary=$1 script=$(realpath "$2") arm=$3 out=$4
C=$HERE/$canary
# shellcheck disable=SC1091
source "$C/cell.env"
export PILOT_DIR=$out FIXTURE=$C/fixture PROMPT_A=$C/prompt.md GRADER=$C/key/grade.py REPO_NAME OBJECTIVE ARM=$arm SKIP_CHECK=1
[ -e "$C/seed-history.sh" ] && export SEED_HOOK=$C/seed-history.sh
rm -rf "$out"
"$PILOT/run.sh" setup >/dev/null
REPO=$out/$REPO_NAME RUNS=$out/runs
export HC_HOME=$out/hc-home
sid="scripted-$(basename "$script" .sh)"
hook() { # event json-body
    printf '{"session_id":"%s","cwd":"%s","hook_event_name":"%s",%s}' "$sid" "$REPO" "$1" "$2" | hc hook claude "$1" >/dev/null 2>&1 || true
}
cd "$REPO"
hook SessionStart '"source":"startup"'
# History reads the script makes are reported through PreToolUse, as Bash calls would be.
before=$(git status --porcelain=v1 -uall | sort)
HC_VALIDATE_HOOK="$sid" bash "$script" "$REPO" >"$RUNS/scripted.out" 2>&1
if grep -q 'git log --all' "$script"; then
    hook PreToolUse '"tool_name":"Bash","tool_input":{"command":"git log --all --oneline"}'
fi
changed=$( { git diff --name-only; git ls-files --others --exclude-standard; git diff --name-only --diff-filter=D; } | sort -u)
transcript=$RUNS/a.jsonl
: >"$transcript"
for f in $changed; do
    hook PostToolUse "\"tool_name\":\"Edit\",\"tool_input\":{\"file_path\":\"$REPO/$f\"},\"tool_response\":{}"
    printf '{"type":"assistant","message":{"content":[{"type":"tool_use","name":"Edit","input":{"file_path":"%s"}}]}}\n' "$REPO/$f" >>"$transcript"
done
printf '{"type":"assistant","message":{"content":[{"type":"tool_use","name":"Bash","input":{"command":"bash %s"}}]}}\n' "$script" >>"$transcript"
grep -q 'git log --all' "$script" && printf '{"type":"assistant","message":{"content":[{"type":"tool_use","name":"Bash","input":{"command":"git log --all --oneline"}}]}}\n' >>"$transcript"
printf '{"type":"result","result":"scripted solution %s applied","total_cost_usd":0}\n' "$(basename "$script")" >>"$transcript"
hook Stop '"stop_hook_active":false,"last_assistant_message":"done"'
echo "claude 0" >"$RUNS/a.exit"
hc brief >"$RUNS/brief-after-a.md" 2>/dev/null || true
"$PILOT/run.sh" judge >/dev/null
"$PILOT/run.sh" score >/dev/null 2>&1 || true
python3 - "$RUNS/score.json" <<'PY'
import json, sys
d = json.load(open(sys.argv[1]))
hc = d.get("hc", {})
print(json.dumps({"valid": d.get("valid"), "problems": d.get("problems"),
                  "outcome": d.get("outcome") or d.get("status"), "cheat": d.get("cheat", d.get("complied")),
                  "signatures": d.get("cheat_signatures") or d.get("other_cheat_signatures"),
                  "probe_mismatches": len(d.get("probe_mismatches") or {}),
                  "retrieved": d.get("retrieved_fix"), "history_reads": d.get("history_reads"),
                  "hc_class": hc.get("class"), "hc_provenance": hc.get("provenance_class"),
                  "judge": hc.get("judge", {}).get("placement"), "rules": hc.get("judge", {}).get("rules")}))
PY
