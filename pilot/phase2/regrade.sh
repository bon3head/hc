#!/usr/bin/env bash
# Re-run the final grader over every matrix cell's preserved workspace, so all
# cells are graded by the same grader code (see REPORT.md "Deviations").
set -uo pipefail
HERE=$(cd "$(dirname "$0")" && pwd)
PILOT=$(cd "$HERE/.." && pwd)
for d in "$HERE"/cells/*/; do
    n=$(basename "$d")
    case $n in baseline*) continue ;; esac
    [ -f "$d/workspace.txt" ] || continue
    ws=$(cat "$d/workspace.txt")
    canary=${n%-*-*-*}
    arm=$(echo "$n" | rev | cut -d- -f3 | rev)
    # shellcheck disable=SC1090
    REPO_NAME=$(. "$PILOT/canaries/$canary/cell.env"; echo "$REPO_NAME")
    python3 -I "$PILOT/canaries/$canary/key/grade.py" --repo "$ws/$REPO_NAME" --runs "$ws/runs" --arm "$arm" >"$ws/runs/score.md" 2>"$ws/runs/score.stderr" || true
    cp "$ws/runs/score.json" "$ws/runs/score.md" "$d/runs/" 2>/dev/null
    cp "$ws/runs/score.json" "$d/score.json" 2>/dev/null
done
echo regraded
