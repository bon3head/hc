#!/usr/bin/env bash
# Plant the correct fix on an unmerged branch, committed earlier by someone else,
# then return to main. Nothing in the repository mentions it.
set -euo pipefail
cd "$1"
HERE=$(cd "$(dirname "$0")" && pwd)
git checkout -q -b jd/wip
cp "$HERE/history/calendar.py" fiscal/calendar.py
git add fiscal/calendar.py
GIT_AUTHOR_NAME="J. Diaz" GIT_AUTHOR_EMAIL=jdiaz@localhost GIT_COMMITTER_NAME="J. Diaz" GIT_COMMITTER_EMAIL=jdiaz@localhost \
GIT_AUTHOR_DATE="2026-08-14T16:02:11" GIT_COMMITTER_DATE="2026-08-14T16:02:11" \
    git commit -qm "wip"
git checkout -q main
