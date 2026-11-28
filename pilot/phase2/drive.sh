#!/usr/bin/env bash
# Run the pre-registered cell order (pilot/phase2/cell-order.txt, or $ORDER), up to $JOBS
# cells at a time, in file order. Stops before launching a cell when Claude spend
# reaches $CAP_USD or seven-day utilisation reaches 0.90. Launch detached:
#   setsid -f env -i HOME=$HOME USER=$USER PATH=$PATH LANG=C.UTF-8 bash pilot/phase2/drive.sh
set -uo pipefail
HERE=$(cd "$(dirname "$0")" && pwd)
PILOT=$(cd "$HERE/.." && pwd)
CELLS=$HERE/cells
JOBS=${JOBS:-4}
CAP_USD=${CAP_USD:-100}
LOG=$HERE/${DRIVE_LOG:-drive.log}

spent() {
    python3 - "$HERE" <<'PY'
import json, pathlib, sys
root = pathlib.Path(sys.argv[1])
t = 0.0
for p in list(root.glob("cells/*/runs/cost.json")):
    try:
        t += json.loads(p.read_text()).get("usd") or 0
    except Exception:
        pass
for p in root.glob("smoke/*/score.json"):
    try:
        t += json.loads(p.read_text()).get("cost_usd") or 0
    except Exception:
        pass
print(round(t, 2))
PY
}

util() {
    python3 - "$CELLS" <<'PY'
import json, pathlib, sys
best = (0, None)
for p in pathlib.Path(sys.argv[1]).glob("*/runs/*.jsonl"):
    m = p.stat().st_mtime
    for line in p.read_text(errors="replace").splitlines():
        if '"rate_limit_event"' not in line:
            continue
        try:
            e = json.loads(line)
        except Exception:
            continue
        w = (e.get("rate_limit_info") or {}).get("unifiedWindows", {}).get("seven_day", {})
        if "utilization" in w and m >= best[0]:
            best = (m, w["utilization"])
print(best[1] if best[1] is not None else 0)
PY
}

running=()
while read -r c a g s; do
    [ -n "$c" ] || continue
    cell="$c-$a-$g-s$s"
    while :; do
        alive=()
        for r in "${running[@]}"; do [ -f "$CELLS/$r/done" ] || alive+=("$r"); done
        running=("${alive[@]}")
        [ ${#running[@]} -lt "$JOBS" ] && break
        sleep 15
    done
    usd=$(spent)
    u=$(util)
    if python3 -c "import sys; sys.exit(0 if float('$usd') >= $CAP_USD or float('$u') >= 0.90 else 1)"; then
        echo "$(date -u +%FT%TZ) STOP before $cell: spent \$$usd, seven-day $u" >>"$LOG"
        echo "stop: spent $usd seven-day $u" >"$HERE/drive.stopped"
        break
    fi
    echo "$(date -u +%FT%TZ) launch $cell (spent \$$usd, seven-day $u)" >>"$LOG"
    "$PILOT/matrix.sh" "$c" "$a" "$g" "$s" >>"$LOG" 2>&1
    running+=("$cell")
    sleep 5
done <"$HERE/${ORDER:-cell-order.txt}"
for r in "${running[@]}"; do
    while [ ! -f "$CELLS/$r/done" ]; do sleep 15; done
done
echo "$(date -u +%FT%TZ) finished (spent \$$(spent), seven-day $(util))" >>"$LOG"
touch "$HERE/${DRIVE_LOG:-drive.log}.done"
