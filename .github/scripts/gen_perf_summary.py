#!/usr/bin/env python3
"""
Read cargo bench --output-format bencher lines from stdin.
Generate a JSON document: { "baseline_recorded_at": "...", "benchmarks": [...] }
Each benchmark: { "name", "mean_ns", "stddev_ns", "ref_units", "vs_baseline_pct" }

`ref_units` is the benchmark's time divided by the reference benchmark's time in
the same run, and `vs_baseline_pct` compares that ratio against the recorded
baseline ratio — the same normalised comparison the regression gate makes, so
the published summary and the gate never disagree. Both are null for a
benchmark with no baseline entry.
"""
import json
import re
import sys
from pathlib import Path

BASELINES_PATH = (
    Path(__file__).parent.parent.parent / "crates" / "workbook" / "benches" / "baselines.json"
)
LINE_RE = re.compile(r"^test (.+?) \.\.\. bench:\s+([\d,]+) ns/iter \(\+/- ([\d,]+)\)")


def main():
    with open(BASELINES_PATH) as f:
        data = json.load(f)
    baselines = data["benchmarks"]
    reference = data["reference"]
    recorded_at = data.get("recorded_at", "")

    measured = []
    for line in sys.stdin:
        m = LINE_RE.match(line.strip())
        if not m:
            continue
        measured.append(
            {
                "name": m.group(1).strip(),
                "mean_ns": int(m.group(2).replace(",", "")),
                "stddev_ns": int(m.group(3).replace(",", "")),
            }
        )

    ref_ns = next((b["mean_ns"] for b in measured if b["name"] == reference), None)

    benchmarks = []
    for b in measured:
        ref_units = None
        vs_baseline_pct = None
        if ref_ns:
            ref_units = round(b["mean_ns"] / ref_ns, 4)
            if b["name"] in baselines:
                baseline_units = baselines[b["name"]]["ref_units"]
                vs_baseline_pct = round((ref_units / baseline_units - 1) * 100, 2)
        benchmarks.append({**b, "ref_units": ref_units, "vs_baseline_pct": vs_baseline_pct})

    summary = {
        "baseline_recorded_at": recorded_at,
        "reference": reference,
        "reference_mean_ns": ref_ns,
        "benchmarks": benchmarks,
    }
    print(json.dumps(summary, indent=2))


if __name__ == "__main__":
    main()
