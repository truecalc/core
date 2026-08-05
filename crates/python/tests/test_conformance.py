"""Sweep the Google Sheets conformance fixtures through the Python binding.

The engine's *correctness* is already enforced by the Rust conformance suite
against these same TSVs. What this file exists to catch is a different failure:
the binding silently corrupting a value on the way out to Python -- a boolean
arriving as ``1.0``, a serial date collapsing into a bare float, an error
becoming a string, a nested array losing a level.

So this is a translation-fidelity sweep over real ground truth, not a second
opinion on the engine. Fixture TSVs are immutable records produced by the
fixtures pipeline; nothing here writes to them or invents expected values.

Rows deliberately not enforced (counted and reported, never silently dropped):

* ``array`` -- the Rust harness compares these via ARRAYTOTEXT canonicalisation.
  Reimplementing that here would duplicate subtle logic and risk a weaker check.
* volatile formulas -- ``NOW``/``TODAY``/``RAND`` have no fixed expected value.

A recorded-empty ``expected_value`` is deliberately *not* skipped: core#767
established that an empty value is the observed value, and skipping it there
silently disabled 95 rows. Those rows are also the only ones that exercise the
``Empty -> None`` mapping.
"""

from __future__ import annotations

import csv
import math
import re
from pathlib import Path

import pytest

from truecalc.core import Date, Engine, Error, Sparkline, Zoned

# crates/python/tests/ -> parents[2] is crates/
FIXTURES = (
    Path(__file__).resolve().parents[2] / "core" / "tests" / "fixtures" / "google_sheets"
)

CONFORMANCE_RS = (
    Path(__file__).resolve().parents[2] / "core" / "tests" / "conformance.rs"
)

# Mirror the Rust runner's enforced file list exactly rather than globbing.
# Globbing would sweep in ai.tsv (functions the engine does not implement),
# bugs.tsv (acknowledged failures) and workbook.tsv (needs the workbook layer)
# and report their known non-conformance as binding defects.
ENFORCED_FILES = [
    "math.tsv", "logical.tsv", "info.tsv", "statistical.tsv", "operator.tsv",
    "text.tsv", "date.tsv", "engineering.tsv", "lookup.tsv", "parser.tsv",
    "database.tsv", "array.tsv", "filter.tsv", "web.tsv", "financial.tsv",
    "google.tsv",
]

# The five types the fixtures pipeline emits, plus `array`. A row carrying
# anything else has a damaged expected_type column (two in text.tsv hold an
# empty string and the *category* value "edge"). The Rust runner counts these
# as malformed against a pinned baseline rather than enforcing them; same here.
RECOGNIZED_TYPES = {"number", "string", "boolean", "error", "date", "array"}

REPRESENTABLE_ERRORS = {
    "#DIV/0!", "#VALUE!", "#REF!", "#NAME?", "#NUM!", "#N/A", "#NULL!",
    "#UNSUPPORTED!",
}

# Volatile functions read the ambient clock, so no fixed expectation can hold.
VOLATILE = ("NOW(", "TODAY(", "RAND(", "RANDBETWEEN(", "RANDARRAY(")

# Numeric tolerance copied from the Rust runner's `values_match`.
def _num_close(a: float, b: float) -> bool:
    return abs(a - b) <= abs(b) * 1e-4 + 1e-10


def known_engine_gaps() -> set[tuple[str, str]]:
    """Rows the Rust suite records as failing against a tracked engine issue.

    Parsed out of conformance.rs rather than duplicated, so a fix that removes
    an entry there cannot leave a stale exemption here.
    """
    src = CONFORMANCE_RS.read_text(encoding="utf-8")
    block = re.search(
        r"const KNOWN_ENGINE_GAPS:[^=]*=\s*&\[(.*?)\n\];", src, re.S
    )
    if not block:
        raise AssertionError(
            "KNOWN_ENGINE_GAPS not found in conformance.rs -- the const was "
            "renamed or reformatted, and this sweep would silently stop "
            "honouring engine-gap exemptions"
        )
    out = set()
    # A raw literal terminates on `"#`, not on the first `"` -- the fixture
    # formulas contain nested quotes (MIDB("农历新年",FINDB("新",...))) and a
    # naive non-greedy match truncates them into entries that match no row.
    for m in re.finditer(
        r'\(\s*"([^"]+\.tsv)"\s*,\s*(?:r#"(.*?)"#|"((?:[^"\\]|\\.)*)")\s*,\s*"[^"]*"\s*\)',
        block.group(1),
        re.S,
    ):
        out.add((m.group(1), m.group(2) if m.group(2) is not None else m.group(3)))
    return out


def reads_sheet_qualified_ref(formula: str) -> bool:
    """A `Sheet!A1`-style reference resolves to empty with no workbook behind it.

    The Rust runner skips these because they do not merely fail -- they can also
    pass for the wrong reason when an empty read happens to produce the recorded
    value. Same hazard applies here.
    """
    pattern = r"[A-Za-z0-9_\u00c0-\uffff']\s*!"
    outside = re.sub(r'"[^"]*"', "", formula)
    if re.search(pattern, outside):
        return True
    # Also inside literals: INDIRECT("Sheet1!A1") reads a sheet-qualified ref
    # that stripping literals would hide (mirrors the Rust runner, which scans
    # literal contents for exactly this).
    return any(re.search(pattern, lit) for lit in formula.split('"')[1::2])


def top_left(value):
    """Anchor-cell view of an unspilled array (mirrors the Rust `top_left`)."""
    while isinstance(value, list) and value:
        value = value[0]
    return value


def _rows():
    for name in ENFORCED_FILES:
        tsv = FIXTURES / name
        with tsv.open(newline="", encoding="utf-8") as fh:
            for row in csv.DictReader(fh, delimiter="\t"):
                formula = (row.get("formula_text") or "").strip()
                if not formula.startswith("="):
                    continue
                yield tsv.name, row, formula


def _matches(actual, expected: str, expected_type: str) -> bool:
    """Compare a Python-side result against the fixture's recorded value."""
    # Scalar expectations compare against the top-left cell of an unspilled
    # array: the engine returns arrays whole, and collapsing to the anchor cell
    # is the surface layer's job (core P1.4 / #526).
    if expected_type != "array":
        actual = top_left(actual)

    if expected_type == "error":
        return isinstance(actual, Error) and actual.code == expected

    # An error where one was not recorded is always a mismatch -- never let an
    # error value coerce into a passing comparison.
    if isinstance(actual, Error):
        return False

    if expected_type == "number":
        if isinstance(actual, Date):
            actual = actual.serial
        try:
            want = float(expected)
        except ValueError:
            return False
        # The TSV stores numeric-looking text as a number, so a Text actual
        # against a number expectation is compared numerically (mirrors the
        # Rust `(Text, Number)` arm, which uses a tighter 1e-9 tolerance).
        if isinstance(actual, str):
            try:
                return abs(float(actual.strip()) - want) <= abs(want) * 1e-9 + 1e-10
            except ValueError:
                return False
        if not isinstance(actual, float):
            return False
        return _num_close(actual, want)

    if expected_type == "date":
        if not isinstance(actual, Date):
            return False
        try:
            return _num_close(actual.serial, float(expected))
        except ValueError:
            return False

    if expected_type == "boolean":
        # Guard the bool-is-a-subclass-of-int trap explicitly: 1.0 must not pass
        # as TRUE. This is precisely the corruption the sweep is here to catch.
        return isinstance(actual, bool) and actual is (expected.upper() == "TRUE")

    if expected_type == "string":
        if expected == "":
            # Three engine values project to nothing on screen, each with a
            # purpose-built arm in the Rust `values_match`:
            #   * blank            -> Empty
            #   * a sparkline      -> Sheets renders a chart, never text
            #   * control chars    -> Sheets strips them from the display
            if actual is None or isinstance(actual, Sparkline):
                return True
            if isinstance(actual, str):
                return all(ord(c) < 32 for c in actual)
        if isinstance(actual, Zoned):
            actual = str(actual)
        if actual is None:
            actual = ""
        if not isinstance(actual, str):
            return False
        if actual == expected:
            return True
        # Sheets' libm can differ from Rust's by 1 ULP in the 15th significant
        # digit for complex-component strings; compare numerically when both
        # sides parse (mirrors the Rust `(Text, Text)` arm).
        try:
            av, ev = float(actual.strip()), float(expected.strip())
        except ValueError:
            return False
        return abs(av - ev) <= abs(ev) * 1e-9 + 1e-15

    return False


@pytest.fixture(scope="module")
def engine():
    return Engine.sheets()


def test_fixture_sweep(engine):
    checked = failures = 0
    skipped = {
        "array": 0, "volatile": 0, "sheet_qualified": 0, "known_engine_gap": 0,
        "unrepresentable_error": 0, "malformed_row": 0,
    }
    gaps = known_engine_gaps()
    unexpected_pass: list[str] = []
    detail: list[str] = []

    for filename, row, formula in _rows():
        expected_type = (row.get("expected_type") or "").strip()
        expected = row.get("expected_value")

        if expected_type not in RECOGNIZED_TYPES:
            skipped["malformed_row"] += 1
            continue
        if expected_type == "array":
            skipped["array"] += 1
            continue
        if any(v in formula.upper() for v in VOLATILE):
            skipped["volatile"] += 1
            continue
        if reads_sheet_qualified_ref(formula):
            skipped["sheet_qualified"] += 1
            continue
        if expected_type == "error" and expected not in REPRESENTABLE_ERRORS:
            # `#ERROR!` is Sheets' parse-failure display; the engine has no such
            # ErrorKind, so the row cannot be enforced either side.
            skipped["unrepresentable_error"] += 1
            continue
        actual = engine.evaluate(formula)

        if (filename, formula) in gaps:
            skipped["known_engine_gap"] += 1
            if _matches(actual, expected, expected_type):
                unexpected_pass.append(f"{filename}: {formula!r}")
            continue

        checked += 1
        if not _matches(actual, expected, expected_type):
            failures += 1
            if len(detail) < 15:
                detail.append(
                    f"{filename}: {formula!r} -> {actual!r} "
                    f"(expected {expected!r} as {expected_type})"
                )

    print(
        f"\nconformance sweep: {checked} rows enforced, {failures} failed, "
        f"skipped {skipped}"
    )
    # Pinned, not a floor: a regression that silently stopped enforcing rows --
    # or a skip bucket that quietly grew -- must fail rather than pass smaller.
    # Raise these numbers when fixtures are added; never lower them silently.
    assert checked >= 10660, f"sweep enforced only {checked} rows (expected >= 10660)"
    assert skipped["array"] <= 39, f"array skips grew to {skipped['array']}"
    # Pinned exactly as the Rust runner pins its own malformed-row baseline: a
    # fourth damaged row must fail the suite, not be absorbed silently.
    assert skipped["malformed_row"] <= 2, (
        f"malformed fixture rows grew to {skipped['malformed_row']}"
    )
    assert skipped["unrepresentable_error"] <= 1, (
        f"unrepresentable-error skips grew to {skipped['unrepresentable_error']}"
    )
    assert skipped["sheet_qualified"] <= 103, (
        f"sheet-qualified skips grew to {skipped['sheet_qualified']}"
    )
    assert failures == 0, "translation mismatches:\n" + "\n".join(detail)
    # An entry must not outlive its fix, same rule the Rust suite enforces.
    assert not unexpected_pass, (
        "rows listed in KNOWN_ENGINE_GAPS now pass:\n" + "\n".join(unexpected_pass)
    )


def test_bool_is_not_a_number(engine):
    """`bool` subclasses `int` in Python -- the binding must not conflate them."""
    assert engine.evaluate("=TRUE()") is True
    assert engine.evaluate("=AND(A1,TRUE)", {"A1": True}) is True
    # A passed-in True must reach the engine as a boolean, not as 1.
    assert engine.evaluate("=ISNUMBER(A1)", {"A1": True}) is False
    assert engine.evaluate("=ISNUMBER(A1)", {"A1": 1}) is True


def test_wrapper_types_survive_a_round_trip(engine):
    """A `Date` passed back in must still be a date to the engine.

    Regression test: `Date` defines `__float__`, so a numeric extraction ordered
    before the `Date` check silently converted it to a plain number on the way
    in and `ISDATE` went False -- erasing the exact distinction the class exists
    to carry. The wrapper types are now matched before any native coercion.
    """
    d = engine.evaluate("=DATE(2026,8,5)")
    assert isinstance(d, Date)
    assert engine.evaluate("=ISDATE(A1)", {"A1": d}) is True
    # Date arithmetic preserves date-ness, so this comes back a Date, and
    # Date != float by design -- compare the serial.
    assert engine.evaluate("=A1+1", {"A1": d}).serial == d.serial + 1

    err = engine.evaluate("=1/0")
    assert engine.evaluate("=ISERROR(A1)", {"A1": err}) is True
    # `#UNSUPPORTED!` has no literal form, so it must be refused rather than
    # silently downgraded to `#VALUE!`.
    import pytest as _pytest
    with _pytest.raises(ValueError):
        engine.evaluate("=A1", {"A1": Engine.excel().evaluate("=1+1")})


def test_unparsable_formula_is_a_value_not_an_exception(engine):
    """Parity with the Rust and JS surfaces, and with Sheets itself."""
    assert isinstance(engine.evaluate("=SUM("), Error)
    assert engine.validate("=SUM(") is not None


def test_date_stays_distinct_from_number(engine):
    d = engine.evaluate("=DATE(2026,8,5)")
    assert isinstance(d, Date)
    assert engine.evaluate("=ISDATE(DATE(2026,8,5))") is True
    # Equality with a bare float must not hold, or the distinction is cosmetic.
    assert d != float(d.serial)


def test_known_engine_gaps_all_match_a_live_row():
    """Every parsed gap entry must match a real fixture row.

    Mirrors the Rust suite's check of the same name. Without it, a regex that
    silently truncates an entry (or a formula edited in the fixture) leaves a
    dead exemption behind, and the sweep reports a tracked engine gap as a
    binding translation defect.
    """
    gaps = known_engine_gaps()
    assert gaps, "KNOWN_ENGINE_GAPS parsed to nothing"
    live = {(name, formula) for name, _row, formula in _rows()}
    orphans = sorted(g for g in gaps if g not in live)
    assert not orphans, (
        "KNOWN_ENGINE_GAPS entries match no fixture row (stale or mis-parsed):\n"
        + "\n".join(f"  {f}: {formula!r}" for f, formula in orphans)
    )
