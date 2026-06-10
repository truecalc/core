#!/usr/bin/env python3
"""
Read cargo bench --output-format bencher lines from stdin.
Generate a JSON document: { "baseline_recorded_at": "...", "benchmarks": [...] }
Each benchmark: { "name": "...", "mean_ns": N, "stddev_ns": M, "vs_baseline_pct": P }
P = (measured/baseline - 1)*100, or null if not in baselines.
Prints JSON to stdout.
"""
import json
import re
import sys
from pathlib import Path

BASELINES_PATH = Path(__file__).parent.parent.parent / "crates" / "workbook" / "benches" / "baselines.json"
LINE_RE = re.compile(r"^test (.+?) \.\.\. bench:\s+([\d,]+) ns/iter \(\+/- ([\d,]+)\)")


def main():
    with open(BASELINES_PATH) as f:
        data = json.load(f)
    baselines = data["benchmarks"]
    recorded_at = data.get("recorded_at", "")

    benchmarks = []
    for line in sys.stdin:
        m = LINE_RE.match(line.strip())
        if not m:
            continue
        name = m.group(1).strip()
        mean_ns = int(m.group(2).replace(",", ""))
        stddev_ns = int(m.group(3).replace(",", ""))

        if name in baselines:
            baseline_ns = baselines[name]["mean_ns"]
            vs_baseline_pct = round((mean_ns / baseline_ns - 1) * 100, 2)
        else:
            vs_baseline_pct = None

        benchmarks.append({
            "name": name,
            "mean_ns": mean_ns,
            "stddev_ns": stddev_ns,
            "vs_baseline_pct": vs_baseline_pct,
        })

    summary = {
        "baseline_recorded_at": recorded_at,
        "benchmarks": benchmarks,
    }
    print(json.dumps(summary, indent=2))


if __name__ == "__main__":
    main()
