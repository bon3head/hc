#!/usr/bin/env bash
# Setup matrix: run the quickstart and `hc doctor` on clean Ubuntu 20.04, 22.04
# and 24.04 images as an ordinary user, then break one thing at a time on 24.04
# and require that setup either works or stops with the right fix.
#
#   scripts/matrix-images.sh          # once: build hc-matrix:{focal,jammy,noble}
#   cargo zigbuild --release -p hc --target x86_64-unknown-linux-gnu.2.17
#   scripts/matrix.sh [path/to/hc]
set -uo pipefail
ROOT=$(cd "$(dirname "$0")/.." && pwd)
HC=${1:-$ROOT/target/x86_64-unknown-linux-gnu/release/hc}
[[ -x $HC ]] || { echo "no hc binary at $HC (see header)"; exit 2; }
PASSED=0 FAILED=()

# run IMAGE SCRIPT [docker args...]: SCRIPT runs as user `dev` with hc on PATH; prints combined output.
run() {
  local image=$1 script=$2; shift 2
  docker run --rm -i "$@" -v "$HC:/opt/hc/hc:ro" -v "$ROOT/scripts:/scripts:ro" "hc-matrix:$image" bash -c '
    useradd -m dev >/dev/null 2>&1
    eval "$PRE"
    su dev -c "export PATH=/opt/hc:\$PATH; cd ~; $SCRIPT"' 2>&1
}

# expect NAME OUTPUT PATTERN...: every pattern must appear.
expect() {
  local name=$1 out=$2; shift 2
  for pat in "$@"; do
    if ! grep -qE -- "$pat" <<<"$out"; then
      FAILED+=("$name: missing /$pat/"); printf '  ✗ %s\n%s\n' "$name" "$(sed 's/^/      /' <<<"$out" | tail -15)"; return
    fi
  done
  PASSED=$((PASSED + 1)); printf '  ✓ %s\n' "$name"
}

toy='git config --global user.email d@x; git config --global user.name d; mkdir -p demo/src demo/tests && cd demo && git init -q
printf "def add(a, b):\n    return a + b\n" > src/calc.py; : > src/__init__.py; : > tests/__init__.py
printf "import unittest\nfrom src.calc import add\nclass T(unittest.TestCase):\n    def test_add(self):\n        self.assertEqual(add(2, 3), 5)\n" > tests/test_calc.py'

echo "Clean machines"
for image in noble jammy; do
  out=$(PRE=: SCRIPT="HC_BIN=hc bash /scripts/quickstart.sh /tmp/qs && hc doctor --repo /tmp/qs/demo" run "$image" x -e PRE -e SCRIPT)
  expect "$image: quickstart and doctor" "$out" "quickstart ok" "hc is ready|hc will run" "signed docket +a test entry was signed and verified"
done
out=$(PRE=: SCRIPT="$toy; hc init 'Fix add'; echo exit=\$?; ls -a" run focal x -e PRE -e SCRIPT)
expect "focal: init refuses old git with the fix" "$out" "cannot run on this machine" "2\.25 cannot sign commits with SSH keys" "upgrade git to 2\.34" "exit=1"
[[ $out == *".handoff"* ]] && FAILED+=("focal: init wrote files before refusing")

echo "Broken machines (24.04)"
doctor_in_repo="$toy; hc init 'Fix add' --for claude >/dev/null; hc doctor; echo exit=\$?"

out=$(PRE='rm -f /usr/bin/git' SCRIPT='mkdir r && cd r && hc doctor; echo exit=$?' run noble x -e PRE -e SCRIPT)
expect "git missing" "$out" "✗ git +not found on PATH" "fix: install git" "exit=1"

out=$(PRE='rm -f /usr/bin/ssh-keygen' SCRIPT="$toy; hc init x; echo exit=\$?" run noble x -e PRE -e SCRIPT)
expect "ssh-keygen missing: init refuses" "$out" "ssh-keygen: not found on PATH" "openssh-client" "exit=1"

out=$(PRE=: SCRIPT="$doctor_in_repo" run noble x -e PRE -e SCRIPT -v /usr/bin/bwrap:/usr/bin/bwrap:ro)
expect "bubblewrap blocked by seccomp: warning, still usable" "$out" "! bubblewrap +installed but cannot create a namespace" "hc will run, with 1 warning" "exit=0"

out=$(PRE=: SCRIPT="$doctor_in_repo" run noble x -e PRE -e SCRIPT)
expect "bubblewrap absent: warning, still usable" "$out" "! bubblewrap +not installed" "exit=0"

out=$(PRE=: SCRIPT="$toy; hc init x >/dev/null; chmod -R a-w ~/.hc; hc doctor; echo exit=\$?" run noble x -e PRE -e SCRIPT)
expect "hc home not writable" "$out" "✗ writable +cannot write to" "chown" "exit=1"

out=$(PRE=: SCRIPT="$toy; hc init x --for claude --hc-bin /opt/gone/hc >/dev/null; hc doctor; echo exit=\$?" run noble x -e PRE -e SCRIPT)
expect "hooks call a binary not on PATH" "$out" "✗ claude hooks +.claude/settings.json runs ./opt/gone/hc., which is not on PATH" "exit=1"

out=$(PRE=: SCRIPT="$toy; hc init x >/dev/null; echo '{bad' > .handoff/task.json; hc doctor; echo exit=\$?" run noble x -e PRE -e SCRIPT)
expect "task file is not JSON" "$out" "✗ task +.handoff/task.json:" "exit=1"

out=$(PRE=: SCRIPT="$toy; hc init x --test 'pytest -q' >/dev/null; hc doctor; echo exit=\$?" run noble x -e PRE -e SCRIPT)
expect "test runner not installed" "$out" "✗ test command +.pytest. is not on PATH" "exit=1"

tamper='h=$(ls -d ~/.hc/repos/*/docket.git); t=$(git --git-dir=$h rev-parse refs/hc/docket^{tree}); c=$(git -c user.name=x -c user.email=x@x --git-dir=$h commit-tree -m forged $t); git --git-dir=$h update-ref refs/hc/docket $c'
out=$(PRE=: SCRIPT="HC_BIN=hc bash /scripts/quickstart.sh /tmp/qs >/dev/null; export HC_HOME=/tmp/qs/hc-home; ${tamper//\~\/.hc/\$HC_HOME}; hc doctor --repo /tmp/qs/demo; echo exit=\$?" run noble x -e PRE -e SCRIPT)
expect "docket forged outside hc" "$out" "✗ docket" "altered outside hc" "exit=1"

out=$(PRE=: SCRIPT="mkdir r && cd r && hc doctor; echo exit=\$?" run noble x -e PRE -e SCRIPT)
expect "not a git repository" "$out" "✗ git work tree" "fix: git init" "exit=1"

out=$(docker run --rm --user 4242:4242 -e HOME=/tmp -v "$HC:/opt/hc/hc:ro" hc-matrix:noble bash -c 'cd /tmp && git init -q r && cd r && /opt/hc/hc doctor; echo exit=$?' 2>&1)
expect "arbitrary UID with no passwd entry (OpenShift-style)" "$out" "signed docket" "exit="

echo
if ((${#FAILED[@]} == 0)); then echo "matrix: $PASSED passed"; exit 0; fi
echo "matrix: $PASSED passed, ${#FAILED[@]} failed"; printf '  - %s\n' "${FAILED[@]}"; exit 1
