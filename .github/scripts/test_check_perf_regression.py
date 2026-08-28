#!/usr/bin/env python3
"""
Self-test for the performance regression gate.

The gate exists to catch order-of-magnitude drift in workbook recalculation. It
previously could not: it was one-sided, so it had no way to notice its own
baseline had been recorded while a defect was present. These cases pin the
behaviour that was missing, using the real measured magnitudes from the two
regressions the gate failed to catch:

* a 33x per-cell recalc regression, and the 33x improvement that removed it;
* a 47x range-lookup regression on range-precedent formulas.

Synthetic bench output is generated from baselines.json itself, so the test
cannot drift out of sync with the recorded numbers.

Run: python3 .github/scripts/test_check_perf_regression.py
"""

import json
import subprocess
import sys
from pathlib import Path

HERE = Path(__file__).parent
GATE = HERE / "check_perf_regression.py"
BASELINES_PATH = HERE.parent.parent / "crates" / "workbook" / "benches" / "baselines.json"

BASELINES = json.loads(BASELINES_PATH.read_text())
REFERENCE = BASELINES["reference"]
REF_NS = 3_400_000


def bench_output(multipliers=None, drop=(), extra=None):
    """Render bencher-format output sitting exactly on the recorded baselines,
    with per-benchmark multipliers applied."""
    multipliers = multipliers or {}
    lines = [f"test {REFERENCE} ... bench: {REF_NS} ns/iter (+/- 1000)"]
    for name, entry in BASELINES["benchmarks"].items():
        if name in drop:
            continue
        ns = int(entry["ref_units"] * REF_NS * multipliers.get(name, 1.0))
        lines.append(f"test {name} ... bench: {ns} ns/iter (+/- 1000)")
    for name, ns in (extra or {}).items():
        lines.append(f"test {name} ... bench: {ns} ns/iter (+/- 1000)")
    return "\n".join(lines) + "\n"


def run_gate(stdin_text):
    proc = subprocess.run(
        [sys.executable, str(GATE)],
        input=stdin_text,
        capture_output=True,
        text=True,
    )
    return proc.returncode, proc.stdout + proc.stderr


def names_matching(prefix):
    return [n for n in BASELINES["benchmarks"] if n.startswith(prefix)]


FAILURES = []


def check(label, condition, detail=""):
    if condition:
        print(f"ok   {label}")
    else:
        print(f"FAIL {label} {detail}")
        FAILURES.append(label)


def main():
    # A run sitting exactly on the baselines must pass.
    code, out = run_gate(bench_output())
    check("clean run passes", code == 0, out)

    # Criterion writes `test NAME ... ` to stdout without a trailing newline;
    # its stderr (e.g. "Failed to access file ... sample.json" on a first
    # run) can land on the same stream and splice text between `...` and
    # `bench:`, pushing the reading onto a later line:
    #
    #   test from_json/500rows ... Criterion.rs ERROR: error: Failed to
    #   access file ".../sample.json": No such file or directory (os error 2)
    #   bench:      812351 ns/iter (+/- 13236)
    #
    # This exact shape made every CI run silently check zero benchmarks for
    # weeks (#899) — the gate reported every baseline as MISSING even though
    # the benchmarks had run. Pin that it now parses.
    split_name = "from_json/500rows"
    split_ns = int(BASELINES["benchmarks"][split_name]["ref_units"] * REF_NS)
    lines = [f"test {REFERENCE} ... bench: {REF_NS} ns/iter (+/- 1000)"]
    for name, entry in BASELINES["benchmarks"].items():
        if name == split_name:
            lines.append(
                f"test {name} ... Criterion.rs ERROR: error: Failed to access file"
            )
            lines.append(
                f'".../target/criterion/{name}/base/sample.json": '
                "No such file or directory (os error 2)"
            )
            lines.append(f"bench:      {split_ns} ns/iter (+/- 13236)")
        else:
            ns = int(entry["ref_units"] * REF_NS)
            lines.append(f"test {name} ... bench: {ns} ns/iter (+/- 1000)")
    code, out = run_gate("\n".join(lines) + "\n")
    check(
        "split test/bench line across interleaved stderr still parses",
        code == 0 and f"MISSING   {split_name}" not in out,
        out,
    )

    # Noise inside the bands must pass. The bands are wide on purpose: a gate
    # that fires on ordinary CI noise gets switched off by whoever it annoys.
    band = {n: 2.4 for n in names_matching("full_recalc/")}
    band.update({n: 0.65 for n in names_matching("depgraph_build/")})
    code, out = run_gate(bench_output(band))
    check("noise inside the bands passes", code == 0, out)

    # A 33x per-cell recalc regression, the magnitude of the defect that
    # survived behind this gate for two months.
    mult = {n: 33.0 for n in names_matching("full_recalc/independent/")}
    code, out = run_gate(bench_output(mult))
    check("33x recalc regression fails", code == 1 and "SLOWER" in out, out)

    # The same defect being *removed*: a 33x improvement against a baseline
    # recorded with it present. This is the case a one-sided gate waves through,
    # which is precisely how the stale baseline went unnoticed.
    mult = {n: 1 / 33.0 for n in names_matching("full_recalc/independent/")}
    code, out = run_gate(bench_output(mult))
    check("33x improvement fails and demands a baseline update", code == 1 and "FASTER" in out, out)

    # A 47x range-lookup regression, on the range-precedent fixtures that did
    # not exist when that regression shipped.
    range_names = (
        names_matching("full_recalc/row_totals/")
        + names_matching("full_recalc/block_subtotals/")
        + names_matching("full_recalc/tall_sparse/")
        + names_matching("depgraph_build/")
    )
    check("range fixtures exist", len(range_names) >= 5, str(range_names))
    code, out = run_gate(bench_output({n: 47.0 for n in range_names}))
    check("47x range regression fails", code == 1 and "SLOWER" in out, out)

    # A benchmark that stops running is not coverage; it is a hole.
    victim = next(iter(BASELINES["benchmarks"]))
    code, out = run_gate(bench_output(drop=(victim,)))
    check("a benchmark that disappears fails", code == 1 and "MISSING" in out, out)

    # A benchmark added without a baseline is ungated; say so rather than
    # silently checking fewer things than the suite contains.
    code, out = run_gate(bench_output(extra={"full_recalc/brand_new/1": 12_345_678}))
    check("a benchmark with no baseline fails", code == 1 and "UNGATED" in out, out)

    # No reference means nothing can be normalised.
    code, out = run_gate("test full_recalc/independent/100 ... bench: 500000 ns/iter (+/- 1)\n")
    check("a run without the reference fails", code == 1 and "Reference benchmark" in out, out)

    if FAILURES:
        print(f"\n{len(FAILURES)} case(s) failed")
        sys.exit(1)
    print("\nall cases passed")


if __name__ == "__main__":
    main()
