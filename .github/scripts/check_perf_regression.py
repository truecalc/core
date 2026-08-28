#!/usr/bin/env python3
"""
Performance regression gate.

Reads `cargo bench --output-format bencher` lines from stdin:

    test NAME ... bench:  N ns/iter (+/- M)

and checks them against crates/workbook/benches/baselines.json. Exits 1 on any
failure.

Three things are checked, and all three are failures:

1. **Regression** — a benchmark got materially slower than its baseline.
2. **Unexpected improvement** — a benchmark got materially faster than its
   baseline. This is a failure on purpose. A one-sided gate cannot notice that
   its own baseline was wrong: if a baseline is recorded while a defect is
   present, the gate then protects the defect. That is exactly how a 33-42x
   recalc regression survived for two months behind a green gate. Requiring an
   explicit baseline update on a large win means the recorded numbers can never
   silently drift far from reality again, and it also catches the other cause
   of a sudden win — a benchmark that quietly stopped measuring anything.
3. **Coverage drift** — a baseline with no matching benchmark (renamed or
   deleted), or a benchmark with no baseline entry (added but never recorded).
   Either one means part of the suite is ungated.

Comparisons are made in units of the reference benchmark measured *in the same
run*, not in absolute nanoseconds. The reference (`calibration/hash_alloc`) is a
fixed allocate-and-hash workload with no dependency on truecalc code, so it
moves only with the machine. Dividing by it is what lets a baseline recorded on
one machine mean something on another; an absolute-ns baseline is only valid on
the hardware that recorded it.
"""

import json
import sys
from pathlib import Path
import re

BASELINES_PATH = (
    Path(__file__).parent.parent.parent / "crates" / "workbook" / "benches" / "baselines.json"
)
LINE_RE = re.compile(r"^test (.+?) \.\.\. bench:\s+([\d,]+) ns/iter")


def parse_measurements(stream):
    measured = {}
    for line in stream:
        m = LINE_RE.match(line.strip())
        if m:
            measured[m.group(1).strip()] = int(m.group(2).replace(",", ""))
    return measured


def main():
    with open(BASELINES_PATH) as f:
        data = json.load(f)

    baselines = data["benchmarks"]
    reference = data["reference"]
    regression_pct = data["regression_pct"]
    improvement_pct = data["improvement_pct"]

    measured = parse_measurements(sys.stdin)

    if reference not in measured:
        print(
            f"Reference benchmark {reference!r} not found in bench output; "
            "cannot normalise. Did the bench run fail?",
            file=sys.stderr,
        )
        sys.exit(1)
    ref_ns = measured[reference]

    failures = []

    missing = sorted(set(baselines) - set(measured))
    for name in missing:
        failures.append(
            f"  MISSING   {name}: has a baseline but did not run. "
            "If it was renamed or removed, update baselines.json."
        )

    unrecorded = sorted(set(measured) - set(baselines) - {reference})
    for name in unrecorded:
        ratio = measured[name] / ref_ns
        failures.append(
            f"  UNGATED   {name}: ran but has no baseline entry, so nothing is "
            f'checking it. Add: "{name}": {{ "ref_units": {ratio:.4f}, '
            f'"mean_ns_at_record": {measured[name]} }}'
        )

    checked = 0
    for name in sorted(set(baselines) & set(measured)):
        baseline_units = baselines[name]["ref_units"]
        units = measured[name] / ref_ns
        checked += 1

        change_pct = (units / baseline_units - 1) * 100
        detail = (
            f"{units:.4f} ref-units vs baseline {baseline_units:.4f} "
            f"({change_pct:+.1f}%; {measured[name]:,} ns measured, "
            f"reference {ref_ns:,} ns)"
        )

        if units > baseline_units * (1 + regression_pct / 100):
            failures.append(
                f"  SLOWER    {name}: {detail}. Ceiling is +{regression_pct}%."
            )
        elif units < baseline_units * (1 - improvement_pct / 100):
            failures.append(
                f"  FASTER    {name}: {detail}. Floor is -{improvement_pct}%. "
                "A win this large must be recorded, not passed silently: set "
                f'"ref_units": {units:.4f} in baselines.json (and check the '
                "benchmark still measures what it claims to)."
            )

    if failures:
        print("Perf gate failed:", file=sys.stderr)
        for f in failures:
            print(f, file=sys.stderr)
        print(
            "\nBaselines live in crates/workbook/benches/baselines.json. "
            "Update them in the same PR as the change that moved them, and say "
            "in the PR body why the number moved.",
            file=sys.stderr,
        )
        sys.exit(1)

    print(
        f"Perf OK ({checked} benchmarks within -{improvement_pct}%/+{regression_pct}% "
        f"of baseline, normalised to {reference} = {ref_ns:,} ns)"
    )


if __name__ == "__main__":
    main()
