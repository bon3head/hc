#!/usr/bin/env bash
# Live pilot: two agent sessions on a seeded task, an owner edit between them,
# and a score from evidence hc does not produce. See pilot/OPERATOR.md.
#
#   pilot/run.sh check|setup|a|oob|b|score|all
#
# PILOT_DIR  where the run lives (default ~/hc-pilot; must not be under /mnt/)
# AGENT_A    claude | codex (default claude)
# AGENT_B    claude | codex (default claude)
set -euo pipefail

HERE=$(cd "$(dirname "$0")" && pwd)
PILOT_DIR=${PILOT_DIR:-$HOME/hc-pilot}
REPO=$PILOT_DIR/ledgerlite
RUNS=$PILOT_DIR/runs
export HC_HOME=$PILOT_DIR/hc-home
AGENT_A=${AGENT_A:-claude}
AGENT_B=${AGENT_B:-claude}

die() { echo "pilot: $*" >&2; exit 1; }

session() { # name agent prompt-file
    local name=$1 agent=$2 prompt
    prompt=$(cat "$3")
    [ -d "$REPO" ] || die "run setup first"
    [ -e "$RUNS/$name.jsonl" ] && die "session $name already ran; start over with: rm -rf $PILOT_DIR && $0 setup"
    command -v "$agent" >/dev/null || die "$agent is not on PATH"
    git -C "$REPO" rev-parse HEAD >"$RUNS/$name.head-before"
    echo "pilot: session $name ($agent) started $(date -u +%FT%TZ); transcript $RUNS/$name.jsonl"
    local rc=0
    cd "$REPO"
    case $agent in
        claude) claude -p "$prompt" --permission-mode bypassPermissions --output-format stream-json --verbose \
                    >"$RUNS/$name.jsonl" 2>"$RUNS/$name.stderr" || rc=$? ;;
        codex)  codex exec --dangerously-bypass-approvals-and-sandbox --json "$prompt" \
                    >"$RUNS/$name.jsonl" 2>"$RUNS/$name.stderr" || rc=$? ;;
        *) die "unknown agent $agent" ;;
    esac
    echo "$agent $rc" >"$RUNS/$name.exit"
    if [ "$rc" -ne 0 ] || ! grep -q '"tool_use"\|"command_execution"\|"exec_command' "$RUNS/$name.jsonl"; then
        echo "invalid: $agent exited $rc with no tool calls" >"$RUNS/$name.invalid"
        echo "pilot: session $name is INVALID ($agent exited $rc, no tool calls recorded). Its output:" >&2
        tail -n 15 "$RUNS/$name.stderr" "$RUNS/$name.jsonl" >&2
        echo "pilot: fix the cause, then start over: rm -rf $PILOT_DIR && $0 setup" >&2
        exit 1
    fi
    hc brief >"$RUNS/brief-after-$name.md" 2>"$RUNS/brief-after-$name.stderr" || echo "pilot: hc brief exited $?" | tee -a "$RUNS/$name.notes"
    echo "pilot: session $name exited $rc; brief saved to $RUNS/brief-after-$name.md"
}

check() { # agent: prove a headless session can run a shell command before seeding anything
    local agent=$1 dir out
    command -v "$agent" >/dev/null || die "$agent is not on PATH"
    case $(command -v "$agent") in /mnt/*) die "$agent resolves to a Windows binary ($(command -v "$agent")); install it inside WSL" ;; esac
    dir=$(mktemp -d)
    out=$dir/check.jsonl
    local ask="Run this shell command and nothing else: echo hc-pilot-$$"
    case $agent in
        claude) (cd "$dir" && claude -p "$ask" --permission-mode bypassPermissions --output-format stream-json --verbose) >"$out" 2>"$dir/err" || true ;;
        codex)  (cd "$dir" && codex exec --skip-git-repo-check --dangerously-bypass-approvals-and-sandbox --json "$ask") >"$out" 2>"$dir/err" || true ;;
    esac
    if grep -q "hc-pilot-$$" "$out" && grep -q '"tool_use"\|"command_execution"\|"exec_command' "$out"; then
        echo "pilot: $agent runs headless and executes commands"
        rm -rf "$dir"
    else
        echo "pilot: $agent could not run a command headless. Its output ($dir):" >&2
        tail -n 15 "$dir/err" "$out" >&2
        exit 1
    fi
}

case ${1:-} in
check) check "$AGENT_A"; [ "$AGENT_B" = "$AGENT_A" ] || check "$AGENT_B" ;;
setup)
    case $PILOT_DIR in /mnt/*) die "PILOT_DIR is on the Windows drive; use a WSL path such as ~/hc-pilot" ;; esac
    [ -e "$PILOT_DIR" ] && die "$PILOT_DIR exists; remove it to start a fresh run"
    command -v hc >/dev/null || die "hc is not on PATH (cargo install --locked --path crates/hc)"
    "$0" check
    mkdir -p "$RUNS"
    cp -r "$HERE/fixture" "$REPO"
    cd "$REPO"
    git init -q -b main
    git add -A
    git -c commit.gpgsign=false -c user.name=pilot -c user.email=pilot@localhost commit -qm "ledgerlite: seeded task"
    hc init "Implement ledgerlite per SPEC.md: parse_amount, invoice_total and render_tags" | tee "$RUNS/init.txt"
    git add -A
    git -c commit.gpgsign=false -c user.name=pilot -c user.email=pilot@localhost commit -qm "hc init" || true
    git tag canary-base
    git rev-parse HEAD >"$RUNS/base.sha"
    hc doctor >"$RUNS/doctor.txt" 2>&1 || true
    hc brief >"$RUNS/brief-initial.md"
    { hc --version 2>/dev/null || true; git --version; uname -a; } >"$RUNS/env.txt"
    for a in claude codex; do command -v $a >/dev/null && echo "$a: $($a --version 2>/dev/null | head -1)" >>"$RUNS/env.txt"; done
    echo "pilot: ready in $REPO; next: $0 a"
    ;;
a) session a "$AGENT_A" "$HERE/prompts/session-a.md" ;;
oob)
    # The owner edits money.py by hand while no session is running. This is a
    # real regression (refunds lose their sign); session B is never told about it.
    [ -e "$RUNS/a.jsonl" ] || die "run session a first"
    [ -e "$RUNS/oob.done" ] && die "owner edit already applied"
    cat >>"$REPO/ledgerlite/money.py" <<'EOF'


_parse_amount_strict = parse_amount


def parse_amount(text: str) -> int:
    # hotfix: refunds are tracked in a separate ledger now
    return abs(_parse_amount_strict(text))
EOF
    git -C "$REPO" diff --stat >"$RUNS/oob.done"
    (cd "$REPO" && hc brief >"$RUNS/brief-after-oob.md" 2>&1) || true
    echo "pilot: owner edit applied to ledgerlite/money.py; next: $0 b"
    ;;
b)
    [ -e "$RUNS/oob.done" ] || die "apply the owner edit first: $0 oob"
    session b "$AGENT_B" "$HERE/prompts/session-b.md"
    ;;
score) python3 -I "$HERE/score.py" --repo "$REPO" --runs "$RUNS" | tee "$RUNS/score.md" ;;
all) "$0" setup && "$0" a && "$0" oob && "$0" b && "$0" score ;;
*) sed -n '2,10p' "$0"; exit 2 ;;
esac
