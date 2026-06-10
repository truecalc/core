//! Array-spill behavior (plan item 3.5, issue #537): Sheets spill semantics —
//! placement, blocked spill, spilled cells as recalc precedents, the 1×1
//! collapse, the spill-resolving `get`/`resolved` seam, and stale-spill cleanup
//! on shrink.
//!
//! These are **behavioral** tests derived from the documented Sheets semantics
//! of schema spec §5/§6/§12 (the authority cited by issue #537). The P3.6
//! fixtures pipeline produces immutable per-step spill grids in
//! `truecalc/fixtures`, but no `spill` fixture is exported into this crate's
//! reachable tree yet (the committed core fixtures cover cross-sheet, named
//! ranges, and date-type). When one lands, a conformance test drives recalc and
//! compares the spilled grid to those immutable expected steps; nothing here
//! hand-authors a "value observed from Sheets" — the array values come from the
//! engine evaluating the formula, and the assertions concern *spill geometry,
//! blocking, and serialization*, which are this crate's contract (§5).

use truecalc_workbook::{
    Address, CellInput, EngineFlavor, RecalcContext, Value, Workbook, Worksheet,
    BLOCKED_SPILL_ERROR,
};

fn a1(s: &str) -> Address {
    Address::from_a1(s).expect("valid A1")
}

fn ctx() -> RecalcContext {
    RecalcContext::new(1_780_000_000_000, "Etc/GMT", 0).expect("Etc/GMT is valid")
}

fn wb() -> Workbook {
    let mut wb = Workbook::new(EngineFlavor::Sheets);
    wb.add_sheet(Worksheet::new("Sheet1")).unwrap();
    wb
}

fn num(n: f64) -> Value {
    Value::Number(n)
}

#[test]
fn array_result_is_stored_at_the_anchor_and_spilled_cells_are_not_authored() {
    let mut wb = wb();
    // A 2x2 array literal spills into A1:B2 with anchor A1.
    wb.set("Sheet1", a1("A1"), CellInput::Formula("={1,2;3,4}".into()))
        .unwrap();
    wb.recalc(&ctx());

    // Anchor stores the full array (its serialized form, §6).
    assert_eq!(
        wb.get("Sheet1", a1("A1")).unwrap().value(),
        &Value::Array(vec![vec![num(1.0), num(2.0)], vec![num(3.0), num(4.0)]])
    );
    // Spilled cells are NOT authored — there is no Cell entry at B1/A2/B2 (§5).
    assert!(wb.get("Sheet1", a1("B1")).is_none());
    assert!(wb.get("Sheet1", a1("A2")).is_none());
    assert!(wb.get("Sheet1", a1("B2")).is_none());
    // Only one cell is physically present in the grid: the anchor.
    assert_eq!(wb.sheet("Sheet1").unwrap().len(), 1);
}

#[test]
fn resolved_reconstructs_spilled_cells_and_reports_the_anchor() {
    let mut wb = wb();
    wb.set("Sheet1", a1("A1"), CellInput::Formula("={1,2;3,4}".into()))
        .unwrap();
    wb.recalc(&ctx());

    // The anchor resolves to its array with no spilledFrom marker.
    let r = wb.resolved("Sheet1", a1("A1")).unwrap();
    assert_eq!(r.anchor, None);

    // Spilled cells resolve to the right element and report the anchor (§5).
    for (cell, expected, i, j) in [
        ("B1", 2.0, 0usize, 1usize),
        ("A2", 3.0, 1, 0),
        ("B2", 4.0, 1, 1),
    ] {
        let r = wb
            .resolved("Sheet1", a1(cell))
            .expect("spilled cell resolves");
        assert_eq!(r.value, num(expected), "value at {cell}");
        assert_eq!(r.anchor, Some(a1("A1")), "anchor at {cell}");
        let _ = (i, j);
    }
    // spill_anchor convenience agrees.
    assert_eq!(wb.spill_anchor("Sheet1", a1("B2")), Some(a1("A1")));
    assert_eq!(wb.spill_anchor("Sheet1", a1("A1")), None);

    // A genuinely empty cell resolves to nothing.
    assert!(wb.resolved("Sheet1", a1("Z9")).is_none());
}

#[test]
fn a_formula_reading_a_spilled_cell_reads_the_spilled_value() {
    let mut wb = wb();
    // Anchor A1 spills {1,2;3,4} across A1:B2; D1 reads the spilled B2 (=4).
    wb.set("Sheet1", a1("A1"), CellInput::Formula("={1,2;3,4}".into()))
        .unwrap();
    wb.set("Sheet1", a1("D1"), CellInput::Formula("=B2+10".into()))
        .unwrap();
    wb.recalc(&ctx());

    // The reader saw the spilled value (4), not Empty (spilled cells are
    // precedents — §5).
    assert_eq!(
        wb.get("Sheet1", a1("D1")).unwrap().value(),
        &num(14.0),
        "reader of a spilled cell must read the spilled value"
    );
}

#[test]
fn reader_before_anchor_in_topo_order_still_sees_the_spill() {
    // D1 (reader) precedes A3 (anchor) only by chance of address order; the
    // fixpoint pass guarantees the reader sees the spill regardless of order.
    let mut wb = wb();
    wb.set("Sheet1", a1("A3"), CellInput::Formula("={10,20}".into()))
        .unwrap(); // spills A3:B3
    wb.set("Sheet1", a1("D1"), CellInput::Formula("=B3*2".into()))
        .unwrap(); // reads spilled B3 = 20
    wb.recalc(&ctx());
    assert_eq!(wb.get("Sheet1", a1("D1")).unwrap().value(), &num(40.0));
}

#[test]
fn blocked_spill_by_authored_literal_yields_the_blocked_error_and_spills_nothing() {
    let mut wb = wb();
    // C1 spills {1,2;3,4} into C1:D2, but D2 is an authored literal -> blocked.
    wb.set("Sheet1", a1("D2"), CellInput::Literal(num(99.0)))
        .unwrap();
    wb.set("Sheet1", a1("C1"), CellInput::Formula("={1,2;3,4}".into()))
        .unwrap();
    wb.recalc(&ctx());

    // Anchor takes the Sheets blocked-spill error; no array is stored (§5/§12).
    assert_eq!(
        wb.get("Sheet1", a1("C1")).unwrap().value(),
        &Value::Error(BLOCKED_SPILL_ERROR.to_owned())
    );
    // The obstructing literal is untouched; the other cells stay empty.
    assert_eq!(wb.get("Sheet1", a1("D2")).unwrap().value(), &num(99.0));
    assert!(wb.resolved("Sheet1", a1("C2")).is_none());
    assert!(wb.resolved("Sheet1", a1("D1")).is_none());
    // No phantom spilled value is produced for D2 (it stays authored).
    assert_eq!(wb.spill_anchor("Sheet1", a1("D2")), None);
}

#[test]
fn blocked_spill_by_another_formula_in_the_path() {
    let mut wb = wb();
    // A1 wants to spill {1;2} into A1:A2, but A2 is itself a formula -> blocked.
    wb.set("Sheet1", a1("A2"), CellInput::Formula("=5+5".into()))
        .unwrap();
    wb.set("Sheet1", a1("A1"), CellInput::Formula("={1;2}".into()))
        .unwrap();
    wb.recalc(&ctx());
    assert_eq!(
        wb.get("Sheet1", a1("A1")).unwrap().value(),
        &Value::Error(BLOCKED_SPILL_ERROR.to_owned())
    );
    // The blocking formula evaluated normally.
    assert_eq!(wb.get("Sheet1", a1("A2")).unwrap().value(), &num(10.0));
}

#[test]
fn two_anchors_competing_for_a_cell_block_deterministically() {
    let mut wb1 = wb();
    // Two disjoint spills both place: A1 -> A1:B1, A2 -> A2:A3.
    wb1.set("Sheet1", a1("A1"), CellInput::Formula("={1,2}".into()))
        .unwrap();
    wb1.set("Sheet1", a1("A2"), CellInput::Formula("={5;6}".into()))
        .unwrap();
    wb1.recalc(&ctx());
    assert!(matches!(
        wb1.get("Sheet1", a1("A1")).unwrap().value(),
        Value::Array(_)
    ));
    assert!(matches!(
        wb1.get("Sheet1", a1("A2")).unwrap().value(),
        Value::Array(_)
    ));

    // Two anchors whose ranges collide: A1 -> A1:C1 would spill across the
    // authored anchor cell B1, and B1 -> B1:B2 is its own array. An *authored*
    // cell (B1 is a formula) in a spill path blocks the spreading anchor
    // (Sheets semantics, §5: "any non-anchor cell ... is authored"). So A1
    // blocks; B1, with a clear path (B2 empty), places. The outcome is
    // deterministic regardless of evaluation order.
    let mut wb2 = wb();
    wb2.set("Sheet1", a1("A1"), CellInput::Formula("={1,2,3}".into()))
        .unwrap(); // A1:C1, but B1 authored in the path
    wb2.set("Sheet1", a1("B1"), CellInput::Formula("={9;9}".into()))
        .unwrap(); // anchor B1 -> B1:B2
    wb2.recalc(&ctx());
    assert_eq!(
        wb2.get("Sheet1", a1("A1")).unwrap().value(),
        &Value::Error(BLOCKED_SPILL_ERROR.to_owned()),
        "an anchor whose spill path crosses an authored cell blocks"
    );
    assert!(
        matches!(
            wb2.get("Sheet1", a1("B1")).unwrap().value(),
            Value::Array(_)
        ),
        "the unobstructed anchor still spills"
    );
}

#[test]
fn one_by_one_array_collapses_to_a_scalar_and_does_not_spill() {
    let mut wb = wb();
    // TRANSPOSE of a single value is 1x1 -> collapsed to scalar (§6); no spill.
    wb.set(
        "Sheet1",
        a1("A1"),
        CellInput::Formula("=TRANSPOSE({7})".into()),
    )
    .unwrap();
    wb.recalc(&ctx());
    assert_eq!(wb.get("Sheet1", a1("A1")).unwrap().value(), &num(7.0));
    assert!(wb.get("Sheet1", a1("A2")).is_none());
}

#[test]
fn out_of_bounds_spill_is_blocked() {
    let mut wb = wb();
    // Anchor in the last column spilling two columns wide leaves the bounds.
    // Last column is ZZZ (18278). Put a wide array there.
    let last_col_a1 = format!("{}1", col_to_letters(18278));
    wb.set(
        "Sheet1",
        a1(&last_col_a1),
        CellInput::Formula("={1,2,3}".into()),
    )
    .unwrap();
    wb.recalc(&ctx());
    assert_eq!(
        wb.get("Sheet1", a1(&last_col_a1)).unwrap().value(),
        &Value::Error(BLOCKED_SPILL_ERROR.to_owned())
    );
}

#[test]
fn shrinking_an_array_clears_stale_spilled_cells() {
    let mut wb = wb();
    wb.set("Sheet1", a1("A1"), CellInput::Formula("={1,2;3,4}".into()))
        .unwrap();
    wb.recalc(&ctx());
    assert!(wb.resolved("Sheet1", a1("B2")).is_some()); // spilled

    // Re-author the anchor with a smaller (1x2) array.
    wb.set("Sheet1", a1("A1"), CellInput::Formula("={5,6}".into()))
        .unwrap();
    wb.recalc(&ctx());
    // A1:B1 now spilled; A2/B2 are stale and must resolve to empty again (§5:
    // the spill is reconstructed from the current anchor array only).
    assert_eq!(wb.resolved("Sheet1", a1("B1")).unwrap().value, num(6.0));
    assert!(
        wb.resolved("Sheet1", a1("A2")).is_none(),
        "stale spilled cell must clear when the array shrinks"
    );
    assert!(wb.resolved("Sheet1", a1("B2")).is_none());
}

#[test]
fn replacing_a_spill_anchor_with_a_scalar_clears_the_whole_spill() {
    let mut wb = wb();
    wb.set("Sheet1", a1("A1"), CellInput::Formula("={1,2}".into()))
        .unwrap();
    wb.recalc(&ctx());
    assert!(wb.resolved("Sheet1", a1("B1")).is_some());

    wb.set("Sheet1", a1("A1"), CellInput::Formula("=42".into()))
        .unwrap();
    wb.recalc(&ctx());
    assert_eq!(wb.get("Sheet1", a1("A1")).unwrap().value(), &num(42.0));
    assert!(wb.resolved("Sheet1", a1("B1")).is_none());
}

#[test]
fn sum_over_range_overlapping_spill_does_not_double_count() {
    // Regression for issue #594: A1 spills {10,20,30} into A1:C1. A SUM over
    // A1:C1 must return 60 (10+20+30), not 110 (10+20+30+20+30 — double-count
    // from the anchor returning the full array while spilled cells also contribute).
    let mut wb = wb();
    wb.set("Sheet1", a1("A1"), CellInput::Formula("={10,20,30}".into()))
        .unwrap();
    wb.set("Sheet1", a1("E1"), CellInput::Formula("=SUM(A1:C1)".into()))
        .unwrap();
    wb.recalc(&ctx());
    assert_eq!(
        wb.get("Sheet1", a1("E1")).unwrap().value(),
        &num(60.0),
        "SUM(A1:C1) over a 1x3 spill anchor must return 60, not 110"
    );
}

/// 1-based column index to A1 letters (mirrors the address module's encoding).
fn col_to_letters(mut col: u32) -> String {
    let mut out = Vec::new();
    while col > 0 {
        let rem = (col - 1) % 26;
        out.push((b'A' + rem as u8) as char);
        col = (col - 1) / 26;
    }
    out.iter().rev().collect()
}

#[test]
fn incremental_recalc_reads_a_spill_whose_anchor_is_not_dirty() {
    // A1 spills {10,20,30} into A1:C1 (full recalc places it). Then a *new*
    // formula D1 reads the spilled C1; an incremental recalc of just D1 must
    // still see the spilled value even though A1 is not in the dirty closure
    // (its array is on the grid — §5).
    let mut wb = wb();
    wb.set("Sheet1", a1("A1"), CellInput::Formula("={10,20,30}".into()))
        .unwrap();
    wb.recalc(&ctx());
    assert!(matches!(
        wb.get("Sheet1", a1("A1")).unwrap().value(),
        Value::Array(_)
    ));

    wb.set("Sheet1", a1("D1"), CellInput::Formula("=C1+1".into()))
        .unwrap();
    let changes = wb.recalc_incremental(&ctx(), &[("Sheet1".to_string(), a1("D1"))]);
    assert_eq!(wb.get("Sheet1", a1("D1")).unwrap().value(), &num(31.0));
    assert!(changes.iter().any(|c| c.addr == a1("D1")));
}

#[test]
fn incremental_equals_full_for_a_spill_and_its_reader() {
    // The incremental result must match a from-scratch full recalc (the
    // P3.3 `incremental ≡ full` guarantee, extended over spills).
    let mut inc = wb();
    inc.set("Sheet1", a1("A1"), CellInput::Formula("={1,2}".into()))
        .unwrap();
    inc.set("Sheet1", a1("D1"), CellInput::Formula("=B1*5".into()))
        .unwrap();
    inc.recalc(&ctx());
    inc.set("Sheet1", a1("A1"), CellInput::Formula("={3,4}".into()))
        .unwrap();
    inc.recalc_incremental(&ctx(), &[("Sheet1".to_string(), a1("A1"))]);

    let mut full = wb();
    full.set("Sheet1", a1("A1"), CellInput::Formula("={3,4}".into()))
        .unwrap();
    full.set("Sheet1", a1("D1"), CellInput::Formula("=B1*5".into()))
        .unwrap();
    full.recalc(&ctx());

    assert_eq!(inc.to_json().unwrap(), full.to_json().unwrap());
    // B1 spilled = 4, so D1 = 20.
    assert_eq!(inc.get("Sheet1", a1("D1")).unwrap().value(), &num(20.0));
}
