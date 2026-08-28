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
# Criterion writes `test NAME ... ` without a trailing newline, so its stderr
# can splice text between `...` and `bench:`, pushing the reading onto a
# later line. This is a small state machine, matching check_perf_regression.py:
# keep looking for `bench: N ns/iter (+/- M)` on the lines following an
# unterminated `test NAME ... ` until it is found or another `test` line
# starts.
TEST_RE = re.compile(r"^test (.+?) \.\.\.\s*(.*)$")
BENCH_RE = re.compile(r"bench:\s+([\d,]+) ns/iter \(\+/- ([\d,]+)\)")


def parse_measurements(stream):
    measured = []
    pending_name = None
    for raw_line in stream:
        line = raw_line.strip()
        m = TEST_RE.match(line)
        if m:
            name, remainder = m.group(1).strip(), m.group(2)
            bm = BENCH_RE.search(remainder)
            if bm:
                measured.append(
                    {
                        "name": name,
                        "mean_ns": int(bm.group(1).replace(",", "")),
                        "stddev_ns": int(bm.group(2).replace(",", "")),
                    }
                )
                pending_name = None
            else:
                pending_name = name
            continue
        if pending_name is not None:
            bm = BENCH_RE.search(line)
            if bm:
                measured.append(
                    {
                        "name": pending_name,
                        "mean_ns": int(bm.group(1).replace(",", "")),
                        "stddev_ns": int(bm.group(2).replace(",", "")),
                    }
                )
                pending_name = None
    return measured


def main():
    with open(BASELINES_PATH) as f:
        data = json.load(f)
    baselines = data["benchmarks"]
    reference = data["reference"]
    recorded_at = data.get("recorded_at", "")

    measured = parse_measurements(sys.stdin)

    ref_ns = next((b["mean_ns"] for b in measured if b["name"] == reference), None)
    if ref_ns is None:
        print(
            f"Reference benchmark {reference!r} not found in bench output; "
            "ref_units and vs_baseline_pct will be null for every benchmark.",
            file=sys.stderr,
        )

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
