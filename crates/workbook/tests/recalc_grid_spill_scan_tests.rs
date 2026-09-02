//! Whether `seed_spills_from_grid` and `GridSpillIndex::build` actually run
//! inside `recompute` (issue #985): both are full-grid scans for spills, and
//! their result is provably empty whenever the #984 anchor-rectangle cache
//! says the stored grid currently holds no array-valued cell — the identical
//! `Value::Array` predicate, checked an instant before either scan would run,
//! with nothing mutating the grid in between.
//!
//! ## Why behavioral, not structural
//!
//! Both functions are private, so this asserts through the same
//! instrumentation the rest of the incremental-recalc suite already uses:
//! [`Workbook::seed_spills_from_grid_calls`] and
//! [`Workbook::grid_spill_index_build_calls`], mirroring
//! `anchor_builds`/`anchor_cache_is_warm` (`spill_anchor_cache_tests.rs`'s own
//! idiom) — wall clock is too machine-dependent to assert a "was it skipped?"
//! question, so the tests below pin exact call counts instead.
//!
//! The critical case is the third one: a short-circuit gated on "does the
//! grid have zero spills *right now*" must not miss a spill *this exact
//! edit* is about to create, since that spill does not exist yet at the
//! moment the gate is checked.

use truecalc_workbook::{
    Address, CellInput, EngineFlavor, RecalcContext, Value, Workbook, Worksheet,
};

fn addr(a1: &str) -> Address {
    Address::from_a1(a1).expect("valid A1")
}

fn ctx() -> RecalcContext {
    RecalcContext::new(1_780_878_600_000, "Etc/GMT", 0).expect("Etc/GMT is a valid tz")
}

// ---------------------------------------------------------------------------
// Case 1: a workbook with zero spills skips both scans.
// ---------------------------------------------------------------------------

#[test]
fn a_zero_spill_workbook_skips_both_scans() {
    let mut wb = Workbook::new(EngineFlavor::Sheets);
    wb.add_sheet(Worksheet::new("S")).unwrap();
    wb.set("S", addr("A1"), CellInput::Literal(Value::Number(1.0)))
        .unwrap();
    wb.set("S", addr("B1"), CellInput::Formula("=A1+1".into()))
        .unwrap();
    wb.set("S", addr("C1"), CellInput::Literal(Value::Number(0.0)))
        .unwrap();

    // Warm the anchor cache first (as the incremental path always does) and
    // establish B1's value, confirming the cache is warm-and-empty before the
    // measured edit.
    wb.recalc_incremental(&ctx(), &[("S".to_owned(), addr("A1"))]);
    assert!(wb.anchor_cache_is_warm());

    wb.set("S", addr("C1"), CellInput::Literal(Value::Number(99.0)))
        .unwrap();
    wb.recalc_incremental(&ctx(), &[("S".to_owned(), addr("C1"))]);

    assert_eq!(
        wb.seed_spills_from_grid_calls(),
        0,
        "a zero-spill workbook must skip seed_spills_from_grid"
    );
    assert_eq!(
        wb.grid_spill_index_build_calls(),
        0,
        "a zero-spill workbook must skip GridSpillIndex::build"
    );
    assert_eq!(
        wb.get("S", addr("B1")).unwrap().value(),
        &Value::Number(2.0)
    );
}

// ---------------------------------------------------------------------------
// Case 2: a workbook that already has a spill still runs (and resolves)
// correctly.
// ---------------------------------------------------------------------------

#[test]
fn a_workbook_with_existing_spills_still_scans() {
    let mut wb = Workbook::new(EngineFlavor::Sheets);
    wb.add_sheet(Worksheet::new("S")).unwrap();
    wb.set("S", addr("A1"), CellInput::Literal(Value::Number(3.0)))
        .unwrap();
    wb.set(
        "S",
        addr("B1"),
        CellInput::Formula("=SEQUENCE(1, A1)".into()),
    )
    .unwrap();
    wb.set("S", addr("Z1"), CellInput::Literal(Value::Number(0.0)))
        .unwrap();

    // First incremental recalc places the spill (B1:D1) and warms the cache.
    wb.recalc_incremental(&ctx(), &[("S".to_owned(), addr("A1"))]);
    assert_eq!(
        wb.resolved("S", addr("D1")).unwrap().anchor,
        Some(addr("B1"))
    );

    let seed_before = wb.seed_spills_from_grid_calls();
    let build_before = wb.grid_spill_index_build_calls();

    // An unrelated edit, elsewhere on the sheet.
    wb.set("S", addr("Z1"), CellInput::Literal(Value::Number(1.0)))
        .unwrap();
    wb.recalc_incremental(&ctx(), &[("S".to_owned(), addr("Z1"))]);

    assert!(
        wb.seed_spills_from_grid_calls() > seed_before,
        "a workbook with an existing spill must still run seed_spills_from_grid"
    );
    assert!(
        wb.grid_spill_index_build_calls() > build_before,
        "a workbook with an existing spill must still run GridSpillIndex::build"
    );
    // The pre-existing spill is still correctly resolved after the "does run"
    // branch — not merely reached, but still correct.
    assert_eq!(
        wb.resolved("S", addr("C1")).unwrap().anchor,
        Some(addr("B1"))
    );
    assert_eq!(
        wb.resolved("S", addr("D1")).unwrap().anchor,
        Some(addr("B1"))
    );
}

// ---------------------------------------------------------------------------
// Case 3: the critical one — an edit that creates a brand-new spill on a
// previously zero-spill workbook must not be missed by the short-circuit.
// ---------------------------------------------------------------------------

#[test]
fn an_edit_that_creates_a_brand_new_spill_is_not_missed() {
    let mut wb = Workbook::new(EngineFlavor::Sheets);
    wb.add_sheet(Worksheet::new("S")).unwrap();
    wb.set("S", addr("A1"), CellInput::Literal(Value::Number(1.0)))
        .unwrap();
    // SEQUENCE(1, 1) collapses to a scalar (schema spec §6): no spill yet.
    wb.set(
        "S",
        addr("B1"),
        CellInput::Formula("=SEQUENCE(1, A1)".into()),
    )
    .unwrap();

    // Warm the anchor cache warm-and-empty first.
    wb.recalc_incremental(&ctx(), &[("S".to_owned(), addr("A1"))]);
    assert!(wb.anchor_cache_is_warm());
    assert!(matches!(
        wb.get("S", addr("B1")).unwrap().value(),
        Value::Number(_)
    ));

    // This exact edit is what creates B1's spill: the pre-pass short-circuit
    // sees zero pre-existing spills, and must not use that to skip placing
    // the new one (placement happens unconditionally in the evaluation loop,
    // not in either scanned function).
    wb.set("S", addr("A1"), CellInput::Literal(Value::Number(3.0)))
        .unwrap();
    wb.recalc_incremental(&ctx(), &[("S".to_owned(), addr("A1"))]);

    assert!(matches!(
        wb.get("S", addr("B1")).unwrap().value(),
        Value::Array(_)
    ));
    assert_eq!(
        wb.resolved("S", addr("C1")).unwrap().anchor,
        Some(addr("B1"))
    );
    assert_eq!(
        wb.resolved("S", addr("D1")).unwrap().anchor,
        Some(addr("B1"))
    );

    // `incremental ≡ full`: an equivalent fresh workbook recalculated from
    // scratch with A1 = 3 from the start must land on byte-identical values.
    let mut fresh = Workbook::new(EngineFlavor::Sheets);
    fresh.add_sheet(Worksheet::new("S")).unwrap();
    fresh
        .set("S", addr("A1"), CellInput::Literal(Value::Number(3.0)))
        .unwrap();
    fresh
        .set(
            "S",
            addr("B1"),
            CellInput::Formula("=SEQUENCE(1, A1)".into()),
        )
        .unwrap();
    fresh.recalc(&ctx());

    assert_eq!(
        wb.get("S", addr("B1")).unwrap().value(),
        fresh.get("S", addr("B1")).unwrap().value()
    );
    assert_eq!(
        wb.resolved("S", addr("C1")).unwrap().value,
        fresh.resolved("S", addr("C1")).unwrap().value
    );
    assert_eq!(
        wb.resolved("S", addr("D1")).unwrap().value,
        fresh.resolved("S", addr("D1")).unwrap().value
    );
}
