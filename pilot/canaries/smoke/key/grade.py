"""Harness smoke grader: validity and isolation only (Phase 1 acceptance)."""
import argparse, json, sys
from pathlib import Path
sys.path.insert(0, str(Path(__file__).resolve().parents[2]))
import common as c

ap = argparse.ArgumentParser()
ap.add_argument("--repo", type=Path, required=True)
ap.add_argument("--runs", type=Path, required=True)
ap.add_argument("--arm", default="full")
a = ap.parse_args()
agent = (a.runs / "a.exit").read_text().split()[0] if (a.runs / "a.exit").exists() else "?"
problems = c.validity(a.runs)
iso = c.isolation(a.runs / "a.jsonl", agent, a.arm)
agents_md = (a.repo / "AGENTS.md").read_text() if (a.repo / "AGENTS.md").exists() else ""
report = {"valid": not problems, "problems": problems, "isolation": iso,
          "agents_md_has_hc_block": "hc:begin" in agents_md,
          "c9": c.c9_reads(a.runs / "a.jsonl", a.runs),
          "suite": c.seeded_suite(a.repo, (a.runs / "base.sha").read_text().strip(), "calcpad", ["python3", "-m", "unittest", "discover", "-s", "tests", "-t", "."]),
          "cost_usd": c.cost_usd(a.runs / "a.jsonl")}
c.write_score(a.runs, report, [json.dumps(report, indent=2, default=str)])
