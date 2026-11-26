#!/usr/bin/env bash
# Exhibit: what the next agent receives before and after one edit.
#
#   tree A  agent fixes add(); claims the suite passes        -> packet 1
#   tree B  agent edits again (breaks add); files no claim    -> packet 2
#   tree B  agent claims the suite passes; verifier has no python3 -> packet 3
#   tree B  same claim, verifier has python3                  -> packet 4
#
# Everything goes through the normal entry points: `hc hook claude ...`,
# `hc claim`, `hc brief`. Packets are written to $OUT/packet-{1..4}.md.
set -euo pipefail
HC=${HC_BIN:-hc}
OUT=${1:-$(mktemp -d)}
export HC_HOME="$OUT/hc-home"
repo="$OUT/demo"
rm -rf "$HC_HOME" "$repo"; mkdir -p "$repo/src" "$repo/tests"
cd "$repo"; git init -q .
printf 'def add(a, b):\n    return a - b\n' >src/calc.py
: >src/__init__.py; : >tests/__init__.py
printf 'import unittest\nfrom src.calc import add\n\n\nclass T(unittest.TestCase):\n    def test_add(self):\n        self.assertEqual(add(2, 3), 5)\n' >tests/test_calc.py
"$HC" init "Make add correct." --for claude >/dev/null
test_cmd='python3 -m unittest discover -s tests -t .'

hook() { printf '{"session_id":"x","cwd":"%s","hook_event_name":"%s",%s}' "$repo" "$1" "$2" | "$HC" hook claude "$1" >/dev/null; }
edit() { printf "def add(a, b):\n    return %s\n" "$1" >src/calc.py
         hook PostToolUse "\"tool_name\":\"Edit\",\"tool_input\":{\"file_path\":\"$repo/src/calc.py\"},\"tool_response\":{}"; }
claim() { printf '{"task":"demo","claims":[{"id":"%s","type":"tests_pass","basis":"ran the suite","command":"%s","exit_code":0,"passed":1,"failed":0,"paths":[],"commit":null,"depends_on":[],"status":"done"}]}' \
            "$1" "$test_cmd" | "$HC" claim --session claude-x --repo "$repo" 2>/dev/null; }
packet() { "$HC" brief --task .handoff/task.json --repo "$repo" >"$OUT/packet-$1.md"; }

hook SessionStart '"source":"startup"'
edit 'a + b'; claim c1; packet 1                 # tree A
edit 'a * b'; packet 2                           # tree B, no new claim
claim c2
mkdir -p "$OUT/nopython"; for t in git ssh-keygen; do ln -sf "$(command -v $t)" "$OUT/nopython/$t"; done
PATH="$OUT/nopython" packet 3                    # runner missing on the verifier
packet 4                                         # runner present

for n in 1 2 3 4; do echo "=== packet $n"; sed -n '/^## Verified facts/,/^## Current step/p' "$OUT/packet-$n.md" | sed '$d'; done

# The exhibit is also a regression check.
has() { grep -qF -- "$2" "$OUT/packet-$1.md" || { echo "packet $1 is missing: $2"; exit 1; }; }
section() { sed -n "/^## $2/,/^## /p" "$OUT/packet-$1.md"; }
has 1 '[reproduced] c1 tests_pass'
section 2 'Stale results' | grep -q '^- c1:' || { echo "packet 2: c1 is not stale"; exit 1; }
section 2 'Verified facts' | grep -q 'c1' && { echo "packet 2: stale c1 still listed as a fact"; exit 1; }
section 3 'Stale results' | grep -q '^- c1:' || { echo "packet 3: c1 lost its history"; exit 1; }
section 3 'Unverified' | grep -q "c2 tests_pass.*not settled on tree [0-9a-f]* (snapshot docket#[0-9]*): .python3. could not start" || { echo "packet 3: c2 is not unresolved"; exit 1; }
grep -q '^## Refuted' "$OUT/packet-3.md" && { echo "packet 3: missing evidence became a refutation"; exit 1; }
section 4 'Refuted claims' | grep -q '^- c2 tests_pass: reexec-disagrees (exit codes \[1, 1\]) on tree [0-9a-f]* (snapshot docket#[0-9]*)' || { echo "packet 4: c2 not refuted"; exit 1; }
echo "exhibit ok"
