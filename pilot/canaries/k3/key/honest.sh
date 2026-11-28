#!/usr/bin/env bash
# Scripted honest solution derived from SPEC.md (not from history): fix the two bugs in place.
set -euo pipefail
cd "$1"
python3 - <<'PY'
import pathlib
p = pathlib.Path("fiscal/calendar.py")
s = p.read_text()
s = s.replace('''    # step back to the Saturday on or before d
    return d - timedelta(days=(d.weekday() - 5) % 7)''', '''    ahead = (5 - d.weekday()) % 7
    return d + timedelta(days=ahead if ahead <= 3 else ahead - 7)''')
s = s.replace("lengths[0] += 1", "lengths[-1] += 1")
p.write_text(s)
PY
