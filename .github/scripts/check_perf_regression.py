#!/usr/bin/env python3
"""
Read cargo bench --output-format bencher lines from stdin.
Lines look like: test NAME ... bench: N ns/iter (+/- M)
Load crates/workbook/benches/baselines.json, check each matched benchmark
against baseline * (1 + threshold_pct/100). Exit 1 if any regression found.
"""
import json
import re
import sys
from pathlib import Path

BASELINES_PATH = Path(__file__).parent.parent.parent / "crates" / "workbook" / "benches" / "baselines.json"
LINE_RE = re.compile(r"^test (.+?) \.\.\. bench:\s+([\d,]+) ns/iter")


def main():
    with open(BASELINES_PATH) as f:
        data = json.load(f)
    baselines = data["benchmarks"]

    regressions = []
    checked = 0

    for line in sys.stdin:
        m = LINE_RE.match(line.strip())
        if not m:
            continue
        name = m.group(1).strip()
        measured_ns = int(m.group(2).replace(",", ""))

        if name not in baselines:
            continue

        entry = baselines[name]
        baseline_ns = entry["mean_ns"]
        threshold_pct = entry["threshold_pct"]
        limit_ns = baseline_ns * (1 + threshold_pct / 100)
        checked += 1

        if measured_ns > limit_ns:
            pct_over = (measured_ns / baseline_ns - 1) * 100
            regressions.append(
                f"  REGRESSION {name}: {measured_ns:,} ns vs baseline {baseline_ns:,} ns "
                f"(+{pct_over:.1f}%, threshold {threshold_pct}%)"
            )

    if regressions:
        print("Perf regressions detected:", file=sys.stderr)
        for r in regressions:
            print(r, file=sys.stderr)
        sys.exit(1)

    print(f"Perf OK ({checked} benchmarks within thresholds)")


if __name__ == "__main__":
    main()
