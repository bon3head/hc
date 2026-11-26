#!/usr/bin/env python3
"""Mutation battery for the trust-critical code.

Each mutant is a plausible bug: one exact text replacement in the source, with
the behavior it must change and the gate check meant to see that change. The
gate's own evidence runs on a scratch copy: the workspace tests,
scripts/attacks.sh, scripts/exhibit-stale.sh and the seed-7 assay.

The unmutated control runs first through the same copy, build and checks. If
it fails, nothing is scored: a "kill" would only be a setup failure. Then, for
each mutant, the intended check runs first:

    killed            the intended check saw the change
    killed-by-other   another gate check saw it (the intended one is blind)
    survived          no gate check can tell this bug from correct code
    stillborn/invalid the mutant itself is broken (does not compile / not found)

    python3 scripts/mutants.py [--only NAME] [--work DIR]
"""
import argparse, os, pathlib, shutil, subprocess, sys, time

ROOT = pathlib.Path(__file__).resolve().parent.parent

# (name, file, original, mutated, intended check, behavior that must change)
MUTANTS = [
    ("unresolved-promotes", "crates/crossexam/src/lib.rs",
     "} else if checks.iter().any(|c| c.result == Outcome::Unresolved) {", "} else if false {",
     "attacks.sh", "a claim with an unresolved check is promoted (G1)"),
    ("no-session-passes-vacuously", "crates/crossexam/src/lib.rs",
     "if ev.start_tree.is_none() {", "if false {",
     "attacks.sh", "write checks pass on an empty write set (G1)"),
    ("settled-ignores-policy", "crates/crossexam/src/lib.rs",
     ".filter(|v| v.policy == digest && v.checks", ".filter(|v| v.checks",
     "attacks.sh", "a verdict settled under an old policy survives a policy change (G4)"),
    ("paths-not-normalized", "crates/crossexam/src/lib.rs",
     "path.split('/').filter(|part| !part.is_empty() && *part != \".\").collect::<Vec<_>>().join(\"/\")", "path.to_owned()",
     "attacks.sh", "./src/calc.py is compared as a different file (F6)"),
    ("paths-exact-only", "crates/crossexam/src/lib.rs",
     "path == c || path.strip_prefix(c.as_str()).is_some_and(|rest| rest.starts_with('/'))", "path == c",
     "attacks.sh", "a claimed directory does not cover its files (F6)"),
    ("unavailable-rerun-passes", "crates/crossexam/src/lib.rs",
     "return Ok(vec![unresolved(PassId::Reexec, \"reexec-unavailable\", why)]);",
     "return Ok(vec![check(PassId::Reexec, true, \"reexec-unavailable\", vec![], Some(why))]);",
     "exhibit-stale.sh", "a rerun that never started counts as reproduced (packet 3)"),
    ("tests-pass-any-exit", "crates/crossexam/src/lib.rs",
     "if claim.kind == ClaimKind::TestsPass { Some(0) } else { claim.exit_code }", "claim.exit_code",
     "attacks.sh", "tests_pass with a null exit code admits a red suite (G2)"),
    ("reexec-any-run-agrees", "crates/crossexam/src/lib.rs",
     "check(PassId::Reexec, all && counts_ok,", "check(PassId::Reexec, any && counts_ok,",
     "attacks.sh", "a flaky suite is reproduced (G3)"),
    ("tamper-blind", "crates/crossexam/src/lib.rs",
     "checks.extend(pass_tamper(claim, &ev, &compiled));", "let _ = pass_tamper(claim, &ev, &compiled);",
     "assay seed 7", "edited tests and hook configs go unnoticed (TestsEdited, HookConfig)"),
    ("stale-never", "crates/brief/src/lib.rs",
     "(None, Some(then), Some(now)) if then != now => Some(now.to_owned()),", "(None, Some(_), Some(_)) if false => None,",
     "exhibit-stale.sh", "a fact for tree A is shown as current on tree B (packet 2)"),
    ("inline-raw", "crates/brief/src/lib.rs",
     "            if c.is_control() {\n                ' '", "            if false {\n                ' '",
     "cargo test", "control characters in agent text reach the brief"),
    ("command-rendered-raw", "crates/brief/src/lib.rs",
     "inline(&shlex::try_join(words.iter().map(String::as_str)).unwrap_or_default())", "command.unwrap_or_default().to_owned()",
     "cargo test", "the raw command string is rendered instead of its parsed words"),
    ("intake-allows-comments", "crates/intake/src/lib.rs",
     "!cmd.chars().any(char::is_control)\n        && shlex::split", "true || shlex::split",
     "attacks.sh", "a command with a shell comment is accepted (F3)"),
    ("intake-allows-any-path", "crates/intake/src/lib.rs",
     "let trimmed = path.trim_start_matches(\"./\");\n    !trimmed.is_empty()",
     "let trimmed = path.trim_start_matches(\"./\");\n    true || !trimmed.is_empty()",
     "attacks.sh", "a path carrying a newline is accepted (F3)"),
    ("intake-null-exit", "crates/intake/src/lib.rs",
     "claim.kind == ClaimKind::CommandRan && claim.exit_code.is_none()", "false",
     "attacks.sh", "command_ran with no exit code is accepted (F4)"),
    ("snapshot-keeps-bytecode", "crates/exhibit/src/snapshot.rs",
     "        git.run(&rm)?;\n", "        let _ = &rm;\n",
     "attacks.sh", "running the suite looks like editing tests/ (F5)"),
    ("git-meta-blind", "crates/exhibit/src/lib.rs",
     "    let dot = worktree.join(\".git\");\n",
     "    return BTreeMap::new();\n    #[allow(unreachable_code)]\n    let dot = worktree.join(\".git\");\n",
     "attacks.sh", "a planted .git hook is not recorded (F1, F9)"),
    ("edits-not-snapshotted", "crates/bailiff/src/lib.rs",
     "if WRITE_TOOLS.contains(&input.tool.as_deref().unwrap_or_default()) {", "if false {",
     "attacks.sh", "a later edit does not move the session's tree (F7)"),
    ("claim-unbound", "crates/hc/src/main.rs",
     "        .or_else(|| bailiff::open_session_for(&env, repo))\n", "",
     "attacks.sh", "a bare hc claim loses its session's writes (F2)"),
    ("hooks-fail-open", "crates/hc/src/main.rs",
     "if block { ExitCode::from(2) } else { ExitCode::SUCCESS }", "if false { ExitCode::from(2) } else { ExitCode::SUCCESS }",
     "attacks.sh", "Stop allows when hc cannot evaluate (F8)"),
    ("task-file-unprotected", "crates/crossexam/src/policy.rs",
     "    \".handoff/**\",\n", "    \"no-such-path/**\",\n",
     "attacks.sh", "the agent can rewrite its own task contract unnoticed (G5)"),
    ("agent-git-checkout", "crates/crossexam/src/reexec.rs",
     "exhibit::snapshot::materialize_from(ledger, &tree, &dir, Some(Path::new(&objects)))?;",
     "let _ = (ledger, &objects); Git::at(repo).run(&[\"worktree\", \"add\", \"--detach\", \"--quiet\", dir.to_str().expect(\"utf-8\"), commit])?;",
     "attacks.sh", "the agent's post-checkout hook runs inside the verifier (F1)"),
]

# Mutant builds go to their own target directory, without incremental caches,
# so the battery's disk use stays bounded and the main build is never touched.
BUILD_ENV = dict(os.environ, CARGO_INCREMENTAL="0")


def run(cmd, cwd, env=None, timeout=900):
    env = env or BUILD_ENV
    start = time.time()
    try:
        p = subprocess.run(cmd, cwd=cwd, env=env, capture_output=True, text=True, timeout=timeout)
        return p.returncode, (p.stdout + p.stderr)[-1500:], time.time() - start
    except subprocess.TimeoutExpired:
        return 124, "timeout", time.time() - start


def checks(repo, work):
    hc = str(repo / "target/debug/hc")
    env = dict(BUILD_ENV, HC_BIN=hc)
    return [
        ("cargo test", ["cargo", "test", "-q", "--workspace"], env),
        ("attacks.sh", ["bash", "scripts/attacks.sh", str(work / "atk")], env),
        ("exhibit-stale.sh", ["bash", "scripts/exhibit-stale.sh", str(work / "ex")], env),
        ("assay seed 7", [hc, "assay", "--seed", "7"], dict(env, HC_HOME=str(work / "assay-home"))),
    ]


def sync(repo):
    if not repo.exists():
        shutil.copytree(ROOT, repo, ignore=shutil.ignore_patterns("refs", ".git", "target"))
        (repo / "refs").mkdir(exist_ok=True)
    for rel in ("crates", "scripts", "schema", "integration", "refs/specs", "Cargo.toml", "Cargo.lock"):
        src, dst = ROOT / rel, repo / rel
        # Fresh timestamps: preserving the originals could make a restored file look
        # older than a binary built from a mutant, and cargo would not rebuild it.
        if src.is_dir():
            shutil.rmtree(dst, ignore_errors=True)
            shutil.copytree(src, dst, copy_function=shutil.copy)
        else:
            shutil.copy(src, dst)


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--only")
    ap.add_argument("--work", default="/tmp/hc-mutants")
    args = ap.parse_args()
    work = pathlib.Path(args.work)
    repo = work / "repo"
    sync(repo)

    code, out, _ = run(["cargo", "build", "-q", "-p", "hc"], repo)
    if code != 0:
        sys.exit(f"CONTROL does not build; no mutant can be scored:\n{out}")
    for label, cmd, env in checks(repo, work):
        code, out, secs = run(cmd, repo, env)
        if code != 0:
            sys.exit(f"CONTROL failed {label}; no mutant can be scored:\n{out}")
        print(f"  control {label}: pass ({secs:.0f}s)", flush=True)

    results = []
    for name, rel, original, mutated, intended, behavior in MUTANTS:
        if args.only and name != args.only:
            continue
        path = repo / rel
        source = path.read_text()
        if source.count(original) != 1:
            print(f"  ? {name}: invalid, original text occurs {source.count(original)} times in {rel}", flush=True)
            results.append((name, "invalid", intended, behavior, ""))
            continue
        path.write_text(source.replace(original, mutated))
        try:
            code, out, _ = run(["cargo", "build", "-q", "-p", "hc"], repo)
            if code != 0:
                print(f"  ? {name}: stillborn, does not compile", flush=True)
                results.append((name, "stillborn", intended, behavior, ""))
                continue
            ordered = sorted(checks(repo, work), key=lambda c: c[0] != intended)
            outcome, by = "survived", ""
            for label, cmd, env in ordered:
                code, out, secs = run(cmd, repo, env)
                if code == 124:
                    outcome, by = "timeout", label
                    break
                if code != 0:
                    outcome, by = ("killed" if label == intended else "killed-by-other"), label
                    break
            results.append((name, outcome, intended, behavior, by))
            mark = {"killed": "✓", "killed-by-other": "~", "survived": "✗", "timeout": "?"}[outcome]
            print(f"  {mark} {name}: {outcome}{' by ' + by if by else ''} (expects {intended} to see: {behavior})", flush=True)
        finally:
            path.write_text(source)

    (work / "mutants.tsv").write_text("".join("\t".join(r) + "\n" for r in results))
    kinds = ("killed", "killed-by-other", "survived", "stillborn", "invalid", "timeout")
    counts = {k: sum(1 for r in results if r[1] == k) for k in kinds}
    print("\nmutants: " + ", ".join(f"{v} {k}" for k, v in counts.items() if v) + f" (of {len(results)})")
    sys.exit(0 if results and counts["killed"] + counts["killed-by-other"] == len(results) else 1)


if __name__ == "__main__":
    main()
