#!/usr/bin/env bash
# The README quickstart as a script: a fresh hc home, a toy repository, one
# `hc init`, a clean `hc doctor`, a protected write denied, a failing Stop
# blocked, a fixed Stop allowed, a conformant plan. Exits non-zero on any deviation.
set -euo pipefail
DIR=${1:-$(mktemp -d)}
HC=${HC_BIN:-hc}
export HC_HOME="$DIR/hc-home"
repo="$DIR/demo"
rm -rf "$HC_HOME" "$repo"
mkdir -p "$repo/src" "$repo/tests"
cd "$repo"
git init -q .
printf 'def add(a, b):\n    return a - b\n' >src/calc.py
: >src/__init__.py
: >tests/__init__.py
printf 'import unittest\nfrom src.calc import add\n\n\nclass T(unittest.TestCase):\n    def test_add(self):\n        self.assertEqual(add(2, 3), 5)\n' >tests/test_calc.py

"$HC" init "Make add correct." --for claude >/dev/null
grep -q '"test_command": "python3 -m unittest discover -s tests -t ."' .handoff/task.json
grep -q 'hc hook claude TaskCompleted' .claude/settings.json
"$HC" doctor >/dev/null

hook() { printf '{"session_id":"demo","cwd":"%s","hook_event_name":"%s",%s}' "$repo" "$1" "$2" | "$HC" hook claude "$1"; }

hook PreToolUse '"tool_name":"Edit","tool_input":{"file_path":"tests/test_calc.py"}' | grep -q '"permissionDecision":"deny"'
hook PostToolUse '"tool_name":"TodoWrite","tool_input":{"todos":[{"content":"Fix add","status":"in_progress","activeForm":"Fixing add"}]},"tool_response":{}' >/dev/null
hook Stop '"stop_hook_active":false' | grep -q '"decision":"block"'
printf 'def add(a, b):\n    return a + b\n' >src/calc.py
hook PostToolUse '"tool_name":"TodoWrite","tool_input":{"todos":[{"content":"Fix add","status":"completed","activeForm":"Fixing add"}]},"tool_response":{}' >/dev/null
[[ -z $(hook Stop '"stop_hook_active":false') ]]
"$HC" conform --session claude-demo --repo "$repo" | python3 -c 'import json,sys; r=json.load(sys.stdin); assert r["fitness_permille"] == 1000, r; assert r["steps"][0]["label"] == "matched", r'
echo "quickstart ok"
