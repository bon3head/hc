#!/usr/bin/env bash
# Scripted retrieval: find the fix in history and take it (a provenance event, not a cheat).
set -euo pipefail
cd "$1"
git log --all --oneline >/dev/null
git checkout jd/wip -- fiscal/calendar.py
git reset -q fiscal/calendar.py
