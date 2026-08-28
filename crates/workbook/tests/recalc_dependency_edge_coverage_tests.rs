//! Recalc-level guards for **range** dependency edges (issue #922).
//!
//! ## Why this file exists
//!
//! `recompute` iterates the evaluation pass to a fixpoint (`max_passes =
//! order.len() + 2`) so that a cell reading a *spilled* cell converges even
//! though a spilled cell is not a formula node. The recalc and conformance
//! suites nonetheless stayed green when the range index was sabotaged to
//! return an empty set and, separately, to drop an occupied row: nothing below
//! the two depgraph test files asserted a **value** that a missing range edge
//! changes, so the defect degraded into a slower recalc rather than a wrong
//! answer. This file closes that gap at the level a user observes.
//!
//! ## Why extra passes cannot repair a missing edge here
//!
//! Three separate reasons, and each test leans on one of them.
//!
//! **1. The fixpoint converges spills, not evaluation order.**
//! `GridResolver::cell_value` resolves an *authored* cell from the **stored
//! grid** before it ever consults the previous pass's values, and the stored
//! grid is not written until `apply_changes` runs after the final pass. The
//! `prev_values` / `prev_spills` fallback that makes the iteration converge is
//! reachable only for cells that are **not** authored — i.e. spilled cells.
//! So a formula cell that reads another formula cell too early sees the same
//! stale stored value on pass 1 and on pass N: repeating the pass changes
//! nothing about it.
//!
//! **2. The circular error is decided before the loop starts.**
//! It comes from `DependencyGraph::cycle_cells`, computed once, up front;
//! `recompute` skips those cells and stamps the error on whatever the order
//! could not place. Drop the range edge that closes a cycle and the graph is
//! acyclic: the cycle set is empty, every cell is placed, and the loop walks a
//! genuine circular reference numerically. There is no pass count at which a
//! number becomes `#REF!`, because no pass consults the cycle set again.
//!
//! **3. A pass only evaluates cells already in `to_eval`.**
//! `recalc_incremental` derives `to_eval` from `direct_dependents_of`. A
//! missing reverse range edge removes a cell from that closure entirely
//! (`if !to_eval.contains(cell) { continue; }`), so no pass ever visits it and
//! its stale stored value is what `diff_against_snapshot` reports. Passes
//! cannot heal a cell they never evaluate.
//!
//! Reason 3 is the shape that matters most going forward: *under*-dirtying
//! leaves a stale value, where over-dirtying only costs time, and a suite that
//! catches only the latter is guarding the wrong direction.
//!
//! ## Keeping the incremental tests honest
//!
//! `seed_spill_sensitive` widens the dirty set independently of the dependency
//! graph, which can mask a missing edge. Two rules avoid that:
//!
//!  * every cell of a range precedent under test is **authored**, because a
//!    range holding an unauthored cell seeds its reader unconditionally; and
//!  * the name-mediated test asserts a cell **downstream** of the name's
//!    reader, because a `Precedent::Name` seeds its own reader unconditionally
//!    — only the hop past it depends on the name edge.

use truecalc_workbook::{
    Address, CellInput, EngineFlavor, RecalcContext, Value, Workbook, Worksheet, CIRCULAR_ERROR,
};

/// A fixed, DST-free context (GMT). Nothing here is volatile.
fn ctx() -> RecalcContext {
    RecalcContext::new(1_780_000_000_000, "Etc/GMT", 0).expect("Etc/GMT is a valid tz")
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

fn num(wb: &Workbook, sheet: &str, cell: &str) -> Value {
    wb.get(sheet, a1(cell)).unwrap().value().clone()
}

fn circular() -> Value {
    Value::Error(CIRCULAR_ERROR.to_owned())
}

// ---------------------------------------------------------------------------
// Shape 1: a cycle closed by a range precedent.
// ---------------------------------------------------------------------------

/// `A1` reads `B1:B3`; `B2` reads `A1`. The cycle is closed *only* by the range
/// edge `B2 -> A1`, so both cells must take the circular error.
///
/// Without that edge the graph is acyclic, the cycle set is empty, and the
/// loop evaluates the recurrence `A1 = B2, B2 = A1 + 1` as ordinary formulas,
/// storing a number.
#[test]
fn full_recalc_reports_a_cycle_closed_by_a_plain_range_precedent() {
    let mut wb = wb_with(&["Sheet1"]);
    wb.set("Sheet1", a1("A1"), CellInput::Formula("=SUM(B1:B3)".into()))
        .unwrap();
    wb.set("Sheet1", a1("B2"), CellInput::Formula("=A1+1".into()))
        .unwrap();

    wb.recalc(&ctx());

    assert_eq!(
        num(&wb, "Sheet1", "A1"),
        circular(),
        "A1 reads B1:B3, which contains the formula cell B2 that reads A1 back; \
         a numeric value here means the range edge B2 -> A1 is missing and the \
         fixpoint iterated the cycle instead of reporting it"
    );
    assert_eq!(num(&wb, "Sheet1", "B2"), circular());
}

/// The same cycle, but the range precedent is reached through a defined name.
/// The closing edge now depends on the name -> target indirection *and* the
/// range expansion behind it.
#[test]
fn full_recalc_reports_a_cycle_closed_by_a_name_mediated_range_precedent() {
    let mut wb = wb_with(&["Sheet1"]);
    wb.define_name("VALS", "Sheet1!B1:B3").unwrap();
    wb.set("Sheet1", a1("A1"), CellInput::Formula("=SUM(VALS)".into()))
        .unwrap();
    wb.set("Sheet1", a1("B2"), CellInput::Formula("=A1+1".into()))
        .unwrap();

    wb.recalc(&ctx());

    assert_eq!(
        num(&wb, "Sheet1", "A1"),
        circular(),
        "A1 reads the name VALS, whose target B1:B3 contains B2, which reads A1 \
         back; a numeric value here means the name-mediated range edge is missing"
    );
    assert_eq!(num(&wb, "Sheet1", "B2"), circular());
}

/// The same cycle spanning two sheets: `Sheet1!A1` reads `Sheet2!B1:B3` and
/// `Sheet2!B2` reads `Sheet1!A1`. The closing edge is a cross-sheet range edge.
#[test]
fn full_recalc_reports_a_cycle_closed_by_a_cross_sheet_range_precedent() {
    let mut wb = wb_with(&["Sheet1", "Sheet2"]);
    wb.set(
        "Sheet1",
        a1("A1"),
        CellInput::Formula("=SUM(Sheet2!B1:B3)".into()),
    )
    .unwrap();
    wb.set(
        "Sheet2",
        a1("B2"),
        CellInput::Formula("=Sheet1!A1+1".into()),
    )
    .unwrap();

    wb.recalc(&ctx());

    assert_eq!(
        num(&wb, "Sheet1", "A1"),
        circular(),
        "Sheet1!A1 reads Sheet2!B1:B3, which contains Sheet2!B2 reading Sheet1!A1 \
         back; a numeric value here means the cross-sheet range edge is missing"
    );
    assert_eq!(num(&wb, "Sheet2", "B2"), circular());
}

// ---------------------------------------------------------------------------
// Shape 2: under-dirtying on an incremental recalc.
// ---------------------------------------------------------------------------

/// `Z1 -> A2` (cell edge) `-> C1` (range edge, `A1:A3`) `-> D1` (cell edge).
///
/// The edit is to `Z1`, which lies **outside** `A1:A3`, so `C1` is reachable
/// only by first evaluating the formula cell `A2` and then following the range
/// edge out of it — exactly "a range precedent ordering two formula cells",
/// asserted as a value. Every cell of `A1:A3` is authored so that spill
/// seeding does not dirty `C1` for unrelated reasons.
#[test]
fn incremental_recalc_refreshes_a_reader_of_a_plain_range_precedent() {
    let mut wb = wb_with(&["Sheet1"]);
    wb.set("Sheet1", a1("Z1"), CellInput::Literal(Value::Number(1.0)))
        .unwrap();
    wb.set("Sheet1", a1("A1"), CellInput::Literal(Value::Number(5.0)))
        .unwrap();
    wb.set("Sheet1", a1("A2"), CellInput::Formula("=Z1*10".into()))
        .unwrap();
    wb.set("Sheet1", a1("A3"), CellInput::Literal(Value::Number(7.0)))
        .unwrap();
    wb.set("Sheet1", a1("C1"), CellInput::Formula("=SUM(A1:A3)".into()))
        .unwrap();
    wb.set("Sheet1", a1("D1"), CellInput::Formula("=C1*2".into()))
        .unwrap();

    wb.recalc(&ctx());
    assert_eq!(num(&wb, "Sheet1", "C1"), Value::Number(22.0));
    assert_eq!(num(&wb, "Sheet1", "D1"), Value::Number(44.0));

    wb.set("Sheet1", a1("Z1"), CellInput::Literal(Value::Number(2.0)))
        .unwrap();
    let changes = wb.recalc_incremental(&ctx(), &[("Sheet1".to_string(), a1("Z1"))]);

    assert_eq!(num(&wb, "Sheet1", "A2"), Value::Number(20.0));
    assert_eq!(
        num(&wb, "Sheet1", "C1"),
        Value::Number(32.0),
        "C1 is in the dirty closure only through the range edge A2 -> C1; a stale \
         22 means the closure missed it, and no extra fixpoint pass can help \
         because passes only evaluate cells already in the closure"
    );
    assert_eq!(num(&wb, "Sheet1", "D1"), Value::Number(64.0));

    let changed: Vec<(String, String)> = changes
        .iter()
        .map(|c| (c.sheet.clone(), format!("{:?}", c.addr)))
        .collect();
    assert_eq!(
        changed.len(),
        3,
        "A2, C1 and D1 all changed value: {changed:?}"
    );
}

/// The same chain with the range precedent reached through a defined name.
///
/// The assertion that discriminates is on `D1`, not `C1`: `C1` reads a
/// `Precedent::Name`, which `seed_spill_sensitive` dirties unconditionally, so
/// `C1` alone proves nothing. `D1` is dirtied only by walking *out* of `C1`,
/// which requires the name edge `A2 -> C1` to have put `C1` in the closure in
/// the first place.
#[test]
fn incremental_recalc_refreshes_a_downstream_reader_of_a_name_mediated_range() {
    let mut wb = wb_with(&["Sheet1"]);
    wb.define_name("VALS", "Sheet1!A1:A3").unwrap();
    wb.set("Sheet1", a1("Z1"), CellInput::Literal(Value::Number(1.0)))
        .unwrap();
    wb.set("Sheet1", a1("A1"), CellInput::Literal(Value::Number(5.0)))
        .unwrap();
    wb.set("Sheet1", a1("A2"), CellInput::Formula("=Z1*10".into()))
        .unwrap();
    wb.set("Sheet1", a1("A3"), CellInput::Literal(Value::Number(7.0)))
        .unwrap();
    wb.set("Sheet1", a1("C1"), CellInput::Formula("=SUM(VALS)".into()))
        .unwrap();
    wb.set("Sheet1", a1("D1"), CellInput::Formula("=C1*2".into()))
        .unwrap();

    wb.recalc(&ctx());
    assert_eq!(num(&wb, "Sheet1", "C1"), Value::Number(22.0));
    assert_eq!(num(&wb, "Sheet1", "D1"), Value::Number(44.0));

    wb.set("Sheet1", a1("Z1"), CellInput::Literal(Value::Number(2.0)))
        .unwrap();
    wb.recalc_incremental(&ctx(), &[("Sheet1".to_string(), a1("Z1"))]);

    assert_eq!(num(&wb, "Sheet1", "A2"), Value::Number(20.0));
    assert_eq!(num(&wb, "Sheet1", "C1"), Value::Number(32.0));
    assert_eq!(
        num(&wb, "Sheet1", "D1"),
        Value::Number(64.0),
        "D1 enters the dirty closure only by walking out of C1, which the \
         name-mediated range edge A2 -> C1 must have put there; a stale 44 means \
         that edge is missing"
    );
}

/// The same chain across two sheets: the range precedent `Sheet2!A1:A3` sits on
/// a different sheet from the cell that reads it.
#[test]
fn incremental_recalc_refreshes_a_reader_of_a_cross_sheet_range_precedent() {
    let mut wb = wb_with(&["Sheet1", "Sheet2"]);
    wb.set("Sheet1", a1("Z1"), CellInput::Literal(Value::Number(1.0)))
        .unwrap();
    wb.set("Sheet2", a1("A1"), CellInput::Literal(Value::Number(5.0)))
        .unwrap();
    wb.set(
        "Sheet2",
        a1("A2"),
        CellInput::Formula("=Sheet1!Z1*10".into()),
    )
    .unwrap();
    wb.set("Sheet2", a1("A3"), CellInput::Literal(Value::Number(7.0)))
        .unwrap();
    wb.set(
        "Sheet1",
        a1("C1"),
        CellInput::Formula("=SUM(Sheet2!A1:A3)".into()),
    )
    .unwrap();
    wb.set("Sheet1", a1("D1"), CellInput::Formula("=C1*2".into()))
        .unwrap();

    wb.recalc(&ctx());
    assert_eq!(num(&wb, "Sheet1", "C1"), Value::Number(22.0));
    assert_eq!(num(&wb, "Sheet1", "D1"), Value::Number(44.0));

    wb.set("Sheet1", a1("Z1"), CellInput::Literal(Value::Number(2.0)))
        .unwrap();
    wb.recalc_incremental(&ctx(), &[("Sheet1".to_string(), a1("Z1"))]);

    assert_eq!(num(&wb, "Sheet2", "A2"), Value::Number(20.0));
    assert_eq!(
        num(&wb, "Sheet1", "C1"),
        Value::Number(32.0),
        "Sheet1!C1 is in the dirty closure only through the cross-sheet range \
         edge Sheet2!A2 -> Sheet1!C1; a stale 22 means that edge is missing"
    );
    assert_eq!(num(&wb, "Sheet1", "D1"), Value::Number(64.0));
}
