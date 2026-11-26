#!/usr/bin/env bash
# Release gate. Every criterion is a command with a pass condition; the
# release counts only when this script prints PASS and exits 0.
set -uo pipefail
cd "$(dirname "$0")/.."

ROOT=$(pwd)
WORK=$(mktemp -d)
trap 'rm -rf "$WORK"' EXIT
FAILED=()
pass() { printf '  \033[32mPASS\033[0m %s\n' "$1"; }
fail() { printf '  \033[31mFAIL\033[0m %s\n' "$1"; FAILED+=("$1"); }
check() { local name=$1; shift; if "$@" >"$WORK/out" 2>&1; then pass "$name"; else fail "$name"; tail -15 "$WORK/out" | sed 's/^/       /'; fi; }

echo "1. Correctness"
check "rustfmt --check" cargo fmt --all -- --check
check "clippy pedantic, warnings are errors" cargo clippy -q --workspace --all-targets -- -D warnings
check "cargo test --workspace" cargo test -q --workspace
check "release build" cargo build -q --release -p hc
HC="$ROOT/target/release/hc"

echo "2. Acceptance on three seeds"
for seed in 7 11 23; do
  report=$(HC_HOME="$WORK/home-$seed" "$HC" assay --seed "$seed" --json)
  read -r caught lies falsep det below adm ver <<<"$(python3 -c 'import json,sys; r=json.load(sys.stdin); print(r["lies_caught"], r["lies"], r["false_refutations"], r["deterministic"], r["facts_below_observed"], r["honest_admitted"], r["honest_verifiable"])' <<<"$report")"
  name="seed $seed: $caught/$lies lies caught, $falsep false refutations, $adm/$ver honest claims delivered, deterministic=$det, facts below observed=$below"
  if [[ $caught == "$lies" && $falsep == 0 && $adm == "$ver" && $ver -gt 0 && $det == True && $below == 0 ]]; then pass "$name"; else fail "$name"; fi
done

echo "3. Every pass earns its place"
full=$(HC_HOME="$WORK/abl" "$HC" assay --seed 7 --json | python3 -c 'import json,sys; print(json.load(sys.stdin)["lies_caught"])')
for p in command tamper scope signature reexec falsify; do
  got=$(HC_HOME="$WORK/abl" "$HC" assay --seed 7 --disable "$p" --json | python3 -c 'import json,sys; print(json.load(sys.stdin)["lies_caught"])')
  if (( got < full )); then pass "disabling $p lowers recall ($full -> $got)"; else fail "disabling $p does not lower recall ($full -> $got)"; fi
done

echo "4. Hook robustness (fault injection)"
for payload in '' '{"session_id":"s","cwd":"/tmp"' 'not json' '[1]' '"str"'; do
  out=$(printf '%s' "$payload" | HC_HOME="$WORK/fi" "$HC" hook claude Stop 2>&1 >/dev/null); code=$?
  printf '%s' "$payload" | HC_FAIL_CLOSED=1 HC_HOME="$WORK/fi" "$HC" hook claude Stop >/dev/null 2>&1; closed=$?
  label="payload $(printf '%q' "$payload" | cut -c1-24): open exit $code with visible error, closed exit $closed"
  if [[ $code == 0 && $out == *"hook error"* && $closed == 2 ]]; then pass "$label"; else fail "$label"; fi
done

echo "5. Hook latency on a real repository (release binary)"
repo="$WORK/repo"; mkdir -p "$repo/src" "$repo/tests" "$repo/.handoff"
git -C "$repo" init -q
printf 'def add(a, b):\n    return a + b\n' >"$repo/src/calc.py"; : >"$repo/src/__init__.py"; : >"$repo/tests/__init__.py"
printf 'import unittest\nfrom src.calc import add\n\n\nclass T(unittest.TestCase):\n    def test_add(self):\n        self.assertEqual(add(2, 3), 5)\n' >"$repo/tests/test_calc.py"
echo '{"id":"t","objective":"o","test_command":"python3 -m unittest discover -s tests -t .","next_step":"n"}' >"$repo/.handoff/task.json"
lat() {
  local event=$1 body=$2 n=30 i
  for ((i = 0; i < n; i++)); do
    local b=${body//@I@/$i}
    local s=$(date +%s%N)
    printf '{"session_id":"lat","cwd":"%s","hook_event_name":"%s",%s}' "$repo" "$event" "$b" | HC_HOME="$WORK/lat" "$HC" hook claude "$event" >/dev/null 2>&1
    echo $(( ($(date +%s%N) - s) / 1000000 ))
  done | sort -n | awk '{a[NR]=$1} END {print a[int(NR*0.95)]}'
}
pre=$(lat PreToolUse '"tool_name":"Edit","tool_input":{"file_path":"src/calc.py"}')
post=$(lat PostToolUse '"tool_name":"TodoWrite","tool_input":{"todos":[{"content":"Step @I@","status":"in_progress","activeForm":"x"}]},"tool_response":{}')
if (( pre < 50 )); then pass "PreToolUse p95 ${pre} ms < 50 ms"; else fail "PreToolUse p95 ${pre} ms >= 50 ms"; fi
if (( post < 500 )); then pass "PostToolUse plan update p95 ${post} ms < 500 ms"; else fail "PostToolUse plan update p95 ${post} ms >= 500 ms"; fi
edit=$(lat PostToolUse '"tool_name":"Edit","tool_input":{"file_path":"src/n@I@.py"},"tool_response":{}' )
if (( edit < 500 )); then pass "PostToolUse edit snapshot p95 ${edit} ms < 500 ms"; else fail "PostToolUse edit snapshot p95 ${edit} ms >= 500 ms"; fi

echo "6. Docs match code"
stale=$(grep -rnE 'crates/hc-(core|observe|kernel|link|adapters|cli)|dsse\.rs|keys\.rs|statement\.rs|hc_core::|hc onboard' README.md docs/decisions.md integration/ 2>/dev/null | grep -v 'Decisions not taken' || true)
if [[ -z $stale ]]; then pass "no references to removed designs"; else fail "stale references"; echo "$stale" | sed 's/^/       /'; fi
missing=0
for c in $(ls crates); do grep -q "\`$c\`" integration/skills/developing-handoff-compiler/references/architecture.md || { echo "       crate $c missing from architecture.md"; missing=1; }; done
if (( missing == 0 )); then pass "every crate documented in the architecture reference"; else fail "crates missing from the architecture reference"; fi

echo "7. Quickstart reproduces in a fresh hc home"
check "scripts/quickstart.sh" env HC_BIN="$HC" bash scripts/quickstart.sh "$WORK/qs"
check "scripts/exhibit-stale.sh: a fact stays scoped to its tree; missing evidence is unresolved, not refuted" env HC_BIN="$HC" bash scripts/exhibit-stale.sh "$WORK/ex"

echo "8. Independent audit"
if [[ -f docs/audit-resolution.md ]] && grep -q '^Unresolved critical or high: 0$' docs/audit-resolution.md; then pass "no unresolved critical or high findings"; else fail "audit not resolved (docs/audit-resolution.md)"; fi

echo
if (( ${#FAILED[@]} == 0 )); then echo "PASS"; exit 0; else echo "FAIL (${#FAILED[@]})"; printf '  - %s\n' "${FAILED[@]}"; exit 1; fi
