//! Recalc-level guards for issue #926: a volatile cell's dirtiness must
//! propagate through the frontier, not just land in `dirty`.
//!
//! ## Why this file exists
//!
//! `recalc_incremental` seeds `dirty` from the edited cells, then drains a
//! `VecDeque` frontier to compute the transitive closure of
//! `direct_dependents_of`. Volatile cells (`TODAY`, `NOW`, ...) are always
//! dirty regardless of what was edited (scope ADR Decision 3), but the
//! volatile-seeding loop ran *after* the frontier was already drained and
//! inserted straight into `dirty` without ever pushing onto the frontier. The
//! volatile cell itself still recomputed (every pass evaluates whatever is in
//! `dirty`), but nothing that reads it — directly or transitively — was ever
//! added to the closure, so it kept its stale stored value.
//!
//! `volatile_cells_are_always_dirty_in_incremental_recalc`
//! (`recalc_incremental_property_tests.rs`) asserts only the volatile cell
//! itself and stops there, so it stayed green with this bug present. A fix
//! that seeds only the volatile cell's *direct* dependents (rather than
//! restoring real frontier propagation) would satisfy a one-hop assertion
//! while still being wrong for anything further downstream — so every test
//! here asserts at least two hops past the volatile cell, and one goes to
//! five hops specifically to catch a fixed-depth half-measure.
//!
//! ## Keeping the incremental tests honest
//!
//! Same two traps as `recalc_dependency_edge_coverage_tests.rs`:
//!
//!  * every cell of a range precedent under test is **authored**, because an
//!    unauthored range cell (or one overlapping a spill rectangle) makes
//!    `seed_spill_sensitive` dirty its reader unconditionally, independent of
//!    whether the volatile-frontier edge was actually walked; and
//!  * the name-mediated assertion is on a cell **downstream** of the name's
//!    reader, because `precedent_is_spill_sensitive` treats every
//!    `Precedent::Name` as spill-sensitive unconditionally — the name reader
//!    itself is always in `dirty` regardless of this bug, so only the hop
//!    past it is discriminating.

use truecalc_workbook::{
    Address, CellInput, EngineFlavor, RecalcContext, Value, Workbook, Worksheet,
};

/// `t0`: 2026-06-08 (matches `recalc_incremental_property_tests.rs`, so the
/// resulting serial dates are already known-good). `t1` is one day later.
fn ctx_t0() -> RecalcContext {
    RecalcContext::new(1_780_878_600_000, "Etc/GMT", 0).expect("Etc/GMT is a valid tz")
}

fn ctx_t1() -> RecalcContext {
    RecalcContext::new(1_780_878_600_000 + 86_400_000, "Etc/GMT", 0).expect("Etc/GMT is a valid tz")
}

fn a1(s: &str) -> Address {
    Address::from_a1(s).expect("valid A1")
}

fn wb_with(sheets: &[&str]) -> Workbook {
    let mut wb = Workbook::new(EngineFlavor::Sheets);
    for name in sheets {
        wb.add_sheet(Worksheet::new(*name)).unwrap();
    }
    wb
}

fn val(wb: &Workbook, sheet: &str, cell: &str) -> Value {
    wb.get(sheet, a1(cell)).unwrap().value().clone()
}

/// Edits an unrelated literal cell so `recalc_incremental` has a normal,
/// non-volatile edit to seed its frontier from, matching the issue's
/// reproduction shape ("edit an unrelated A1").
fn touch_unrelated_cell(wb: &mut Workbook, sheet: &str) {
    wb.set(sheet, a1("A1"), CellInput::Literal(Value::Number(2.0)))
        .unwrap();
}

// ---------------------------------------------------------------------------
// Shape 1: two hops downstream of `=TODAY()` (the issue's own reproduction).
// ---------------------------------------------------------------------------

#[test]
fn two_hops_downstream_of_a_volatile_cell_are_refreshed() {
    let mut wb = wb_with(&["S"]);
    wb.set("S", a1("A1"), CellInput::Literal(Value::Number(1.0)))
        .unwrap();
    wb.set("S", a1("B1"), CellInput::Formula("=TODAY()".into()))
        .unwrap();
    wb.set("S", a1("B2"), CellInput::Formula("=B1+0".into()))
        .unwrap();
    wb.set("S", a1("B3"), CellInput::Formula("=B2+0".into()))
        .unwrap();
    wb.recalc(&ctx_t0());
    assert_eq!(val(&wb, "S", "B1"), Value::Date(46181.0));
    assert_eq!(val(&wb, "S", "B3"), Value::Date(46181.0));

    touch_unrelated_cell(&mut wb, "S");
    let changes = wb.recalc_incremental(&ctx_t1(), &[("S".to_string(), a1("A1"))]);

    assert_eq!(val(&wb, "S", "B1"), Value::Date(46182.0));
    assert_eq!(
        val(&wb, "S", "B2"),
        Value::Date(46182.0),
        "B2 is one hop downstream of the volatile B1; a stale 46181 means the \
         volatile seed never entered the frontier"
    );
    assert_eq!(
        val(&wb, "S", "B3"),
        Value::Date(46182.0),
        "B3 is two hops downstream of the volatile B1 (via B2); a stale 46181 \
         here specifically catches a fix that seeds only B1's *direct* \
         dependents instead of restoring real frontier propagation"
    );

    let touched: Vec<String> = changes.iter().map(|c| c.addr.to_a1()).collect();
    assert!(touched.contains(&"B2".to_string()), "{touched:?}");
    assert!(touched.contains(&"B3".to_string()), "{touched:?}");
}

// ---------------------------------------------------------------------------
// Shape 2: a volatile cell's dependent reached through a range precedent.
// ---------------------------------------------------------------------------

/// `B1` (volatile) sits inside the authored range `B1:B3` that `C1` sums;
/// `D1` reads `C1` directly. `D1`, not `C1`, is the discriminating
/// assertion — `C1`'s value already reflects the graph edge, but `D1` can
/// only be reached by walking *out* of `C1`, which requires `C1` to have
/// entered the frontier in the first place.
#[test]
fn a_volatile_cells_range_mediated_dependent_is_refreshed_two_hops_out() {
    let mut wb = wb_with(&["S"]);
    wb.set("S", a1("A1"), CellInput::Literal(Value::Number(1.0)))
        .unwrap();
    wb.set("S", a1("B1"), CellInput::Formula("=TODAY()".into()))
        .unwrap();
    wb.set("S", a1("B2"), CellInput::Literal(Value::Number(10.0)))
        .unwrap();
    wb.set("S", a1("B3"), CellInput::Literal(Value::Number(20.0)))
        .unwrap();
    wb.set("S", a1("C1"), CellInput::Formula("=SUM(B1:B3)".into()))
        .unwrap();
    wb.set("S", a1("D1"), CellInput::Formula("=C1*2".into()))
        .unwrap();
    wb.recalc(&ctx_t0());
    assert_eq!(val(&wb, "S", "C1"), Value::Number(46211.0));
    assert_eq!(val(&wb, "S", "D1"), Value::Number(92422.0));

    touch_unrelated_cell(&mut wb, "S");
    wb.recalc_incremental(&ctx_t1(), &[("S".to_string(), a1("A1"))]);

    assert_eq!(val(&wb, "S", "B1"), Value::Date(46182.0));
    assert_eq!(val(&wb, "S", "C1"), Value::Number(46212.0));
    assert_eq!(
        val(&wb, "S", "D1"),
        Value::Number(92424.0),
        "D1 enters the dirty closure only by walking out of C1, which the \
         range edge B1 -> C1 (through the volatile B1) must have put there; a \
         stale 92422 means the volatile frontier never reached the range \
         reader"
    );
}

// ---------------------------------------------------------------------------
// Shape 3: a volatile cell's dependent reached through a defined name.
// ---------------------------------------------------------------------------

/// Same shape as above, but `C1` reads the range through the defined name
/// `VOL` rather than a bare `B1:B3` literal. `C1` is *not* the discriminating
/// assertion here either: `precedent_is_spill_sensitive` treats every
/// `Precedent::Name` as spill-sensitive unconditionally, so `C1` is always in
/// `dirty` regardless of this bug. `D1`, reached only by walking out of `C1`,
/// is what actually depends on the volatile-frontier fix.
#[test]
fn a_volatile_cells_name_mediated_dependent_is_refreshed_two_hops_out() {
    let mut wb = wb_with(&["S"]);
    wb.define_name("VOL", "S!B1:B3").unwrap();
    wb.set("S", a1("A1"), CellInput::Literal(Value::Number(1.0)))
        .unwrap();
    wb.set("S", a1("B1"), CellInput::Formula("=TODAY()".into()))
        .unwrap();
    wb.set("S", a1("B2"), CellInput::Literal(Value::Number(10.0)))
        .unwrap();
    wb.set("S", a1("B3"), CellInput::Literal(Value::Number(20.0)))
        .unwrap();
    wb.set("S", a1("C1"), CellInput::Formula("=SUM(VOL)".into()))
        .unwrap();
    wb.set("S", a1("D1"), CellInput::Formula("=C1*2".into()))
        .unwrap();
    wb.recalc(&ctx_t0());
    assert_eq!(val(&wb, "S", "C1"), Value::Number(46211.0));
    assert_eq!(val(&wb, "S", "D1"), Value::Number(92422.0));

    touch_unrelated_cell(&mut wb, "S");
    wb.recalc_incremental(&ctx_t1(), &[("S".to_string(), a1("A1"))]);

    assert_eq!(val(&wb, "S", "B1"), Value::Date(46182.0));
    assert_eq!(val(&wb, "S", "C1"), Value::Number(46212.0));
    assert_eq!(
        val(&wb, "S", "D1"),
        Value::Number(92424.0),
        "D1 enters the dirty closure only by walking out of C1, which the \
         name edge VOL -> C1 (through the volatile B1) must have put there; a \
         stale 92422 means the volatile frontier never reached the \
         name-mediated reader"
    );
}

// ---------------------------------------------------------------------------
// Shape 4: a chain long enough that a fixed-depth fix would fail.
// ---------------------------------------------------------------------------

/// Five hops downstream of the volatile `B1` (`B2` through `B6`). A fix that
/// seeds only the volatile cell's direct dependents, or that walks a fixed
/// small number of extra hops instead of restoring real frontier
/// propagation, refreshes some prefix of this chain and then stops short of
/// `B6`.
#[test]
fn a_five_hop_chain_downstream_of_a_volatile_cell_is_fully_refreshed() {
    let mut wb = wb_with(&["S"]);
    wb.set("S", a1("A1"), CellInput::Literal(Value::Number(1.0)))
        .unwrap();
    wb.set("S", a1("B1"), CellInput::Formula("=TODAY()".into()))
        .unwrap();
    wb.set("S", a1("B2"), CellInput::Formula("=B1+0".into()))
        .unwrap();
    wb.set("S", a1("B3"), CellInput::Formula("=B2+0".into()))
        .unwrap();
    wb.set("S", a1("B4"), CellInput::Formula("=B3+0".into()))
        .unwrap();
    wb.set("S", a1("B5"), CellInput::Formula("=B4+0".into()))
        .unwrap();
    wb.set("S", a1("B6"), CellInput::Formula("=B5+0".into()))
        .unwrap();
    wb.recalc(&ctx_t0());
    assert_eq!(val(&wb, "S", "B6"), Value::Date(46181.0));

    touch_unrelated_cell(&mut wb, "S");
    let changes = wb.recalc_incremental(&ctx_t1(), &[("S".to_string(), a1("A1"))]);

    assert_eq!(val(&wb, "S", "B1"), Value::Date(46182.0));
    for (cell, hop) in [("B2", 1), ("B3", 2), ("B4", 3), ("B5", 4), ("B6", 5)] {
        assert_eq!(
            val(&wb, "S", cell),
            Value::Date(46182.0),
            "{cell} is {hop} hop(s) downstream of the volatile B1; a stale \
             46181 here means propagation stopped before reaching it"
        );
    }

    let touched: Vec<String> = changes.iter().map(|c| c.addr.to_a1()).collect();
    for cell in ["B2", "B3", "B4", "B5", "B6"] {
        assert!(touched.contains(&cell.to_string()), "{touched:?}");
    }
}
