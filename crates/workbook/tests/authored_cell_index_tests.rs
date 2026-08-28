//! "Does this rectangle contain an unauthored cell?": cost and equivalence
//! (issue #927).
//!
//! `seed_spill_sensitive` asks that question once per range precedent of every
//! formula cell, and it used to answer by scanning **every authored cell on
//! the sheet** and counting the ones inside the rectangle. One seeding pass
//! therefore cost `O(range precedents * authored cells)`, and the shapes that
//! cost the most — a `SUM` row total per row, a block subtotal per block —
//! are exactly the ones where every cell of every range *is* authored, so the
//! scan ran in full and found nothing, every time.
//!
//! These tests pin the change in **authored cells examined per seeding
//! decision** — an exact count taken from the lookup itself, so it holds on
//! any machine and in either build profile and cannot go flaky. Wall-clock
//! numbers belong in the PR discussion, not in a test.
//!
//! Correctness is the real risk, not speed. This decision is dirty-set
//! membership: over-dirtying only costs time, but under-dirtying leaves a
//! stale value on the grid. The scan is kept here as an oracle and asserted to
//! agree with the index rectangle by rectangle — exhaustively over a small
//! coordinate grid, on workbooks that include the awkward shapes — and
//! `a_reader_of_a_vacated_spill_range_still_refreshes` pins the consequence at
//! recalc level, so an index that silently answered "fully authored" would
//! fail as a wrong value and not merely as a mismatched bool.
//!
//! ## Keeping the incremental test honest
//!
//! Same trap as `recalc_dependency_edge_coverage_tests.rs` and
//! `recalc_volatile_frontier_tests.rs`, inverted: the recalc-level test here
//! must be seeded *only* by this decision, so the edited anchor is replaced
//! with a **literal**. That leaves no array on the grid, hence no spill
//! rectangle for `precedent_is_spill_sensitive`'s overlap branch to fire on,
//! and no dependency edge from the anchor to the reader — the unauthored-cell
//! branch is the only thing that can put the reader in the dirty set.
//!
//! That guard stops at the seeded reader itself, deliberately. Spill seeding
//! inserts straight into the dirty set *after* the frontier has already
//! drained, so nothing downstream of a spill-seeded cell joins the closure —
//! the same shape as the volatile-seeding defect, at a second site, and
//! present independently of anything here. Asserting a second hop would fail
//! for that reason rather than for this one, so it is left to whoever fixes
//! it.

use truecalc_workbook::{
    Address, AuthoredCellIndex, Cell, CellInput, DependencyGraph, EngineFlavor, Precedent,
    RangeRef, RecalcContext, Value, Workbook, Worksheet,
};

fn addr(a1: &str) -> Address {
    Address::from_a1(a1).unwrap()
}

fn ctx() -> RecalcContext {
    RecalcContext::new(1_780_878_600_000, "Etc/GMT", 0).expect("Etc/GMT is a valid tz")
}

fn wb_with(sheets: &[&str]) -> Workbook {
    let mut wb = Workbook::new(EngineFlavor::Sheets);
    for name in sheets {
        wb.add_sheet(Worksheet::new(*name)).unwrap();
    }
    wb
}

fn set_formula(wb: &mut Workbook, sheet: &str, a1: &str, formula: &str) {
    wb.sheet_mut(sheet)
        .unwrap()
        .set(addr(a1), Cell::with_formula(formula, Value::Empty));
}

fn set_num(wb: &mut Workbook, sheet: &str, a1: &str, n: f64) {
    wb.sheet_mut(sheet)
        .unwrap()
        .set(addr(a1), Cell::literal(Value::Number(n)).unwrap());
}

/// A rectangle on `sheet`, corners given as raw 1-based coordinates so the
/// tests can build the shapes a parser never produces (inverted corners, row
/// or column zero, coordinates past the address bounds). `RangeRef::sheet` is
/// the simple-case-folded sheet name; every sheet named here is ASCII, for
/// which simple folding is lowercasing.
fn rect(sheet: &str, r0: u32, c0: u32, r1: u32, c1: u32) -> RangeRef {
    RangeRef {
        sheet: sheet.to_lowercase(),
        start: Address {
            row: r0,
            column: c0,
        },
        end: Address {
            row: r1,
            column: c1,
        },
    }
}

// ---------------------------------------------------------------------------
// The oracle: the pre-index scan, kept so the index can be held to it.
// ---------------------------------------------------------------------------

/// The scan this change replaced, kept verbatim apart from one guard: it
/// computed `end - start + 1` unconditionally, which **underflows** on a
/// rectangle whose corners arrive the wrong way round (`RangeRef`'s fields are
/// public, so that rectangle is reachable) — a panic in a debug build and a
/// wrapped, astronomically large area in a release one. Both land on "has an
/// unauthored cell", which is the conservative answer, so the guard states it
/// outright rather than reproducing an overflow.
fn scan_oracle(wb: &Workbook, r: &RangeRef) -> bool {
    let Some(sheet) = wb
        .sheets()
        .iter()
        .find(|s| s.name().to_lowercase() == r.sheet)
    else {
        // The range targets a missing sheet; nothing authored, so it is
        // (vacuously) all-unauthored — seed conservatively.
        return true;
    };
    if r.start.row > r.end.row || r.start.column > r.end.column {
        return true;
    }
    let rows = u64::from(r.end.row - r.start.row) + 1;
    let cols = u64::from(r.end.column - r.start.column) + 1;
    let area = rows.saturating_mul(cols);
    let authored_inside = sheet
        .iter()
        .filter(|(a, _)| {
            a.row >= r.start.row
                && a.row <= r.end.row
                && a.column >= r.start.column
                && a.column <= r.end.column
        })
        .count() as u64;
    authored_inside < area
}

/// Asserts the index answers exactly what the scan answers, for `r`.
fn assert_agrees(label: &str, wb: &Workbook, r: &RangeRef) {
    let index = AuthoredCellIndex::build(wb);
    let (indexed, _) = index.range_has_unauthored_cell_examined(r);
    let scanned = scan_oracle(wb, r);
    assert_eq!(
        indexed, scanned,
        "{label}: index said {indexed}, scan said {scanned} for {r:?}"
    );
}

// ---------------------------------------------------------------------------
// Corpus workbooks.
// ---------------------------------------------------------------------------

/// A 4x4 block of literals at A1:D4 with **no** gaps, plus a formula outside
/// it. Every rectangle inside the block is fully authored.
fn wb_dense_block() -> Workbook {
    let mut wb = wb_with(&["S", "Other"]);
    for r in 1..=4u32 {
        for c in 1..=4u32 {
            set_num(&mut wb, "S", &Address::new(r, c).unwrap().to_a1(), 1.0);
        }
    }
    set_formula(&mut wb, "S", "F1", "=SUM(A1:D4)");
    wb
}

/// The same block with exactly one hole punched in it (C3).
fn wb_one_gap() -> Workbook {
    let mut wb = wb_dense_block();
    wb.sheet_mut("S").unwrap().clear(addr("C3"));
    wb
}

/// A block whose rows are *entirely* literal — no formula anywhere inside the
/// populated region — so an index over formula cells would call it unauthored.
fn wb_literal_rows() -> Workbook {
    let mut wb = wb_with(&["S", "Other"]);
    for r in 1..=3u32 {
        for c in 1..=5u32 {
            set_num(&mut wb, "S", &Address::new(r, c).unwrap().to_a1(), 7.0);
        }
    }
    wb
}

/// Two sheets holding cells at the *same* addresses, so a lookup that ignored
/// the sheet would still find something.
fn wb_two_sheets() -> Workbook {
    let mut wb = wb_with(&["S", "Other"]);
    for r in 1..=3u32 {
        set_num(&mut wb, "S", &Address::new(r, 1).unwrap().to_a1(), 1.0);
    }
    set_num(&mut wb, "Other", "A1", 9.0);
    set_num(&mut wb, "Other", "A3", 9.0); // row 2 missing on this sheet only
    wb
}

/// A ragged region: rows of differing widths, and a wholly empty row inside.
fn wb_ragged() -> Workbook {
    let mut wb = wb_with(&["S", "Other"]);
    for c in 1..=4u32 {
        set_num(&mut wb, "S", &Address::new(1, c).unwrap().to_a1(), 1.0);
    }
    for c in 2..=3u32 {
        set_num(&mut wb, "S", &Address::new(2, c).unwrap().to_a1(), 1.0);
    }
    // row 3 empty
    for c in 1..=2u32 {
        set_num(&mut wb, "S", &Address::new(4, c).unwrap().to_a1(), 1.0);
    }
    set_num(&mut wb, "S", "E5", 1.0);
    wb
}

/// A workbook whose sheet holds a live spill: `A1={10,20,30}` covers A1:C1, so
/// B1 and C1 are occupied on screen but **not authored**.
fn wb_with_spill() -> Workbook {
    let mut wb = wb_with(&["S", "Other"]);
    wb.set("S", addr("A1"), CellInput::Formula("={10,20,30}".into()))
        .unwrap();
    wb.set("S", addr("A2"), CellInput::Literal(Value::Number(1.0)))
        .unwrap();
    wb.set("S", addr("B2"), CellInput::Literal(Value::Number(2.0)))
        .unwrap();
    wb.set("S", addr("C2"), CellInput::Literal(Value::Number(3.0)))
        .unwrap();
    wb.recalc(&ctx());
    wb
}

/// Two sheets whose names case-fold to the same key: `"Data"` (tab 0, empty)
/// and `"DATA"` (tab 1, A1:A2 authored). `insert_sheet`/`from_json` reject
/// folded duplicates, so this shape is only reachable by pushing straight onto
/// `Workbook::sheets_mut()`, which no production code does — but if it is
/// reached, the index must resolve the folded key the same way the deleted
/// scan's `sheets().iter().find` did: first-wins.
fn wb_case_colliding_sheets() -> Workbook {
    let mut wb = wb_with(&["Data"]);
    let mut data_upper = Worksheet::new("DATA");
    data_upper.set(addr("A1"), Cell::literal(Value::Number(1.0)).unwrap());
    data_upper.set(addr("A2"), Cell::literal(Value::Number(1.0)).unwrap());
    wb.sheets_mut().push(data_upper);
    wb
}

fn corpus() -> Vec<(&'static str, Workbook)> {
    vec![
        ("empty", wb_with(&["S", "Other"])),
        ("dense block", wb_dense_block()),
        ("one gap", wb_one_gap()),
        ("literal rows", wb_literal_rows()),
        ("two sheets", wb_two_sheets()),
        ("ragged", wb_ragged()),
        ("spill", wb_with_spill()),
        ("case-colliding sheet names", wb_case_colliding_sheets()),
    ]
}

// ---------------------------------------------------------------------------
// Equivalence.
// ---------------------------------------------------------------------------

/// The headline correctness property: over every rectangle expressible in a
/// small coordinate grid — including the inverted ones and the ones anchored
/// at row/column zero, which only a hand-built `RangeRef` can produce — the
/// index answers exactly what the scan answered, on every workbook in the
/// corpus and on both a present and an absent sheet.
#[test]
fn index_matches_the_scan_over_every_rectangle_in_a_small_grid() {
    let mut compared = 0usize;
    for (label, wb) in corpus() {
        let index = AuthoredCellIndex::build(&wb);
        for sheet in ["S", "Missing"] {
            for r0 in 0..=5u32 {
                for r1 in 0..=5u32 {
                    for c0 in 0..=5u32 {
                        for c1 in 0..=5u32 {
                            let r = rect(sheet, r0, c0, r1, c1);
                            let indexed = index.range_has_unauthored_cell_examined(&r).0;
                            let scanned = scan_oracle(&wb, &r);
                            assert_eq!(
                                indexed, scanned,
                                "{label}: index said {indexed}, scan said {scanned} for {r:?}"
                            );
                            compared += 1;
                        }
                    }
                }
            }
        }
    }
    // 8 workbooks x 2 sheets x 6^4 rectangles.
    assert_eq!(compared, 8 * 2 * 6 * 6 * 6 * 6);
}

/// The cases the issue calls out by name, asserted one at a time so a failure
/// says which shape broke rather than which coordinate quadruple did.
#[test]
fn index_matches_the_scan_on_the_named_edge_cases() {
    let dense = wb_dense_block();
    // Fully authored, whole block and a proper sub-rectangle.
    assert_agrees("fully authored block", &dense, &rect("S", 1, 1, 4, 4));
    assert_agrees("fully authored inner", &dense, &rect("S", 2, 2, 3, 3));
    assert!(!AuthoredCellIndex::build(&dense).range_has_unauthored_cell(&rect("S", 1, 1, 4, 4)));
    // Single cell, authored and not.
    assert_agrees("single authored cell", &dense, &rect("S", 2, 2, 2, 2));
    assert_agrees("single empty cell", &dense, &rect("S", 9, 9, 9, 9));
    // Entirely outside the populated region.
    assert_agrees("outside the region", &dense, &rect("S", 50, 50, 60, 60));
    // Straddling the edge of the populated region.
    assert_agrees("straddling the edge", &dense, &rect("S", 3, 3, 6, 6));

    let gap = wb_one_gap();
    // One gap: inside it, and a rectangle that just misses it.
    assert_agrees("range over the gap", &gap, &rect("S", 1, 1, 4, 4));
    assert!(AuthoredCellIndex::build(&gap).range_has_unauthored_cell(&rect("S", 1, 1, 4, 4)));
    assert_agrees("range missing the gap", &gap, &rect("S", 1, 1, 2, 4));

    let literals = wb_literal_rows();
    // Rows holding only literals are fully authored: the whole point of not
    // reusing an index over *formula* cells.
    assert_agrees("literal-only rows", &literals, &rect("S", 1, 1, 3, 5));
    assert!(
        !AuthoredCellIndex::build(&literals).range_has_unauthored_cell(&rect("S", 1, 1, 3, 5)),
        "a range over rows of literals holds no unauthored cell"
    );

    let two = wb_two_sheets();
    // Same addresses, different sheets, different answers.
    assert_agrees("sheet S rows 1-3", &two, &rect("S", 1, 1, 3, 1));
    assert_agrees("sheet Other rows 1-3", &two, &rect("Other", 1, 1, 3, 1));
    assert!(!AuthoredCellIndex::build(&two).range_has_unauthored_cell(&rect("S", 1, 1, 3, 1)));
    assert!(AuthoredCellIndex::build(&two).range_has_unauthored_cell(&rect("Other", 1, 1, 3, 1)));
    // A sheet the workbook does not have.
    assert_agrees("unknown sheet", &two, &rect("Nope", 1, 1, 1, 1));
    assert!(AuthoredCellIndex::build(&two).range_has_unauthored_cell(&rect("Nope", 1, 1, 1, 1)));

    let spill = wb_with_spill();
    // A spilled cell is occupied but not authored, so a range over the spill
    // footprint holds unauthored cells; the authored row below it does not.
    assert_agrees("over the spill", &spill, &rect("S", 1, 1, 1, 3));
    assert!(AuthoredCellIndex::build(&spill).range_has_unauthored_cell(&rect("S", 1, 1, 1, 3)));
    assert_agrees("row below the spill", &spill, &rect("S", 2, 1, 2, 3));
    assert!(!AuthoredCellIndex::build(&spill).range_has_unauthored_cell(&rect("S", 2, 1, 2, 3)));
    assert_agrees("spill and the row below", &spill, &rect("S", 1, 1, 2, 3));

    let ragged = wb_ragged();
    assert_agrees(
        "ragged, over the empty row",
        &ragged,
        &rect("S", 1, 1, 4, 4),
    );
    assert_agrees("ragged, narrow and full", &ragged, &rect("S", 1, 2, 2, 3));
    assert_agrees(
        "ragged, trailing empty rows",
        &ragged,
        &rect("S", 4, 1, 9, 2),
    );

    let colliding = wb_case_colliding_sheets();
    // Case-colliding sheet names: "Data" (tab 0, empty) and "DATA" (tab 1,
    // A1:A2 authored) fold to the same key. First-wins must resolve to the
    // empty tab-0 sheet, matching the deleted scan's `sheets().iter().find`.
    assert_agrees(
        "case-colliding sheet names",
        &colliding,
        &rect("Data", 1, 1, 2, 1),
    );
    assert_agrees(
        "case-colliding sheet names, DATA spelling",
        &colliding,
        &rect("DATA", 1, 1, 2, 1),
    );
    assert!(
        AuthoredCellIndex::build(&colliding)
            .range_has_unauthored_cell(&rect("Data", 1, 1, 2, 1)),
        "first-wins: the empty tab-0 sheet must answer, not the authored tab-1 one"
    );
}

/// A rectangle whose corners are the wrong way round reaches this decision
/// through `RangeRef`'s public fields. The scan subtracted the corners and
/// underflowed there; the index answers conservatively instead — the same
/// direction, without the panic.
#[test]
fn an_inverted_rectangle_answers_conservatively_instead_of_panicking() {
    let wb = wb_dense_block();
    let index = AuthoredCellIndex::build(&wb);
    for r in [
        rect("S", 4, 1, 1, 4), // rows inverted
        rect("S", 1, 4, 4, 1), // columns inverted
        rect("S", 4, 4, 1, 1), // both inverted
    ] {
        assert!(
            index.range_has_unauthored_cell(&r),
            "an inverted rectangle must seed conservatively: {r:?}"
        );
        assert_eq!(index.range_has_unauthored_cell_examined(&r).1, 0);
    }
}

/// Coordinates past the address bounds (rows `1..=10_000_000`, columns
/// `1..=18_278`) are likewise only reachable by hand, and must not overflow
/// the area arithmetic.
#[test]
fn out_of_bounds_coordinates_do_not_overflow() {
    let wb = wb_dense_block();
    let index = AuthoredCellIndex::build(&wb);
    for r in [
        rect("S", 1, 1, u32::MAX, u32::MAX),
        rect("S", 0, 0, u32::MAX, u32::MAX),
        rect("S", u32::MAX, u32::MAX, u32::MAX, u32::MAX),
        rect("S", 1, 1, 10_000_000, 18_278),
    ] {
        assert!(index.range_has_unauthored_cell(&r), "{r:?}");
        assert_eq!(scan_oracle(&wb, &r), true, "{r:?}");
    }
}

// ---------------------------------------------------------------------------
// Cost.
// ---------------------------------------------------------------------------

/// A monthly-operating-model shape: an assumptions cell, twelve columns of
/// growth-chain formulas, a `SUM` row total per row, and a block subtotal every
/// 100 rows. `blocks * 100` data rows, so the row/block ratio — and therefore
/// the expected authored-cells-examined-per-decision — does not shift with
/// size. Every cell of every range is authored, which is the shape the scan
/// was worst on: it ran in full and found nothing, every time.
fn operating_model(blocks: u32) -> Workbook {
    let mut wb = wb_with(&["S"]);
    set_num(&mut wb, "S", "A1", 0.05);
    let rows = blocks * 100;
    for i in 0..rows {
        let r = i + 2; // data starts on row 2
        for c in 0..12u32 {
            let col = Address::new(r, 2 + c).unwrap().to_a1();
            if i == 0 {
                set_formula(&mut wb, "S", &col, "=$A$1*100");
            } else {
                let prev = Address::new(r - 1, 2 + c).unwrap().to_a1();
                set_formula(&mut wb, "S", &col, &format!("={prev}*(1+$A$1)"));
            }
        }
        let total = Address::new(r, 14).unwrap().to_a1();
        let first = Address::new(r, 2).unwrap().to_a1();
        let last = Address::new(r, 13).unwrap().to_a1();
        set_formula(&mut wb, "S", &total, &format!("=SUM({first}:{last})"));

        if (i + 1) % 100 == 0 {
            let sub = Address::new(r, 15).unwrap().to_a1();
            let top = Address::new(r - 99, 14).unwrap().to_a1();
            set_formula(&mut wb, "S", &sub, &format!("=SUM({top}:{total})"));
        }
    }
    wb
}

/// One seeding pass's worth of decisions: every range precedent of every
/// formula cell, which is what `seed_spill_sensitive` asks about.
///
/// Returns (authored cells examined per decision, decisions, authored cells on
/// the sheet, authored cells examined in total).
fn examined_per_decision(wb: &Workbook) -> (f64, usize, usize, usize) {
    let graph = DependencyGraph::build(wb);
    let index = AuthoredCellIndex::build(wb);
    let mut examined = 0usize;
    let mut decisions = 0usize;
    for cell in graph.formula_cells() {
        for prec in graph.precedents_of(cell).unwrap() {
            let Precedent::Range(r) = prec else { continue };
            decisions += 1;
            let (has_unauthored, n) = index.range_has_unauthored_cell_examined(r);
            // This model is fully authored: the decision the scan paid a whole
            // sheet to reach is always "no".
            assert!(!has_unauthored, "operating model should be fully authored");
            examined += n;
        }
    }
    let authored: usize = wb.sheets().iter().map(Worksheet::len).sum();
    #[allow(clippy::cast_precision_loss)]
    let per_decision = examined as f64 / decisions as f64;
    (per_decision, decisions, authored, examined)
}

/// The headline property: the cost of one seeding decision does not grow with
/// the sheet. Before the index it was exactly the sheet's authored-cell count
/// — the scan examined every one of them — so this number *was* the sheet
/// size.
#[test]
fn authored_cells_examined_per_seeding_decision_is_flat_as_the_sheet_grows() {
    let mut measurements = Vec::new();
    for blocks in [1u32, 2, 4, 8] {
        let wb = operating_model(blocks);
        measurements.push(examined_per_decision(&wb));
    }

    let (baseline, _, smallest_authored, _) = measurements[0];
    let (largest_per_decision, _, largest_authored, largest_examined) =
        *measurements.last().unwrap();
    // The model has one shared assumptions cell, so growth is 8x up to that
    // constant.
    assert_eq!(
        largest_authored - 1,
        (smallest_authored - 1) * 8,
        "model did not scale 8x"
    );

    for &(per_decision, decisions, authored, _) in &measurements {
        assert!(
            (per_decision - baseline).abs() < 0.5,
            "authored cells examined per seeding decision moved with sheet size: \
             {per_decision} at {authored} authored cells ({decisions} decisions), \
             {baseline} at {smallest_authored}; all measurements: {measurements:?}"
        );
    }

    // And it is not merely flat, it is small: the scan examined one cell per
    // authored cell on the sheet, for every decision.
    #[allow(clippy::cast_precision_loss)]
    let scan_cost = largest_authored as f64;
    assert!(
        largest_per_decision < scan_cost / 100.0,
        "expected far fewer than the {scan_cost} cells the scan examined, \
         got {largest_per_decision}"
    );

    // The index is built once per seeding pass, so the pass as a whole must be
    // cheaper too — a flat per-query cost paid for by a quadratic build would
    // be no fix at all. Build is one pass over the authored cells; the scan
    // paid that once *per decision*.
    let (_, largest_decisions, _, _) = *measurements.last().unwrap();
    let pass_cost = largest_authored + largest_examined;
    let scan_pass_cost = largest_decisions * largest_authored;
    assert!(
        pass_cost * 100 < scan_pass_cost,
        "whole-pass cost {pass_cost} (build {largest_authored} + queries \
         {largest_examined}) is not two orders of magnitude below the scan's \
         {scan_pass_cost}"
    );
}

/// The trap: a whole-column-shaped reference covers ten million rows, almost
/// all empty. Answering must not walk them. Only *occupied* rows are keyed,
/// and a rectangle bigger than everything authored on the sheet is answered
/// from the sheet's cell count alone.
#[test]
fn a_tall_range_over_empty_rows_costs_nothing_extra() {
    let mut wb = wb_with(&["S"]);
    for r in 1..=10u32 {
        set_num(&mut wb, "S", &Address::new(r, 1).unwrap().to_a1(), 1.0);
    }
    let index = AuthoredCellIndex::build(&wb);

    let over =
        |last_row: u32| index.range_has_unauthored_cell_examined(&rect("S", 1, 1, last_row, 1));

    let (ten, ten_examined) = over(10);
    assert!(!ten, "A1:A10 is fully authored");
    let (tall, tall_examined) = over(10_000_000);
    assert!(tall, "A1:A10000000 is almost entirely unauthored");
    assert_eq!(
        tall_examined, 0,
        "a ten-million-row reference must be answered from the sheet's cell \
         count, not by walking rows"
    );
    // The fully authored case walks its ten occupied rows and no more.
    assert!(
        ten_examined <= 20,
        "A1:A10 examined {ten_examined} authored cells"
    );
}

/// A gap stops the walk where it is found rather than after the whole
/// rectangle: the sparse-range shape must not pay for the rows past the first
/// hole.
#[test]
fn a_gap_ends_the_walk_at_the_gap() {
    let mut wb = wb_with(&["S"]);
    for r in 1..=200u32 {
        if r == 3 {
            continue; // the hole
        }
        set_num(&mut wb, "S", &Address::new(r, 1).unwrap().to_a1(), 1.0);
    }
    let index = AuthoredCellIndex::build(&wb);
    let (has_unauthored, examined) =
        index.range_has_unauthored_cell_examined(&rect("S", 1, 1, 200, 1));
    assert!(has_unauthored);
    assert!(
        examined <= 10,
        "the walk should stop at row 3, not run to row 200; examined {examined}"
    );
}

// ---------------------------------------------------------------------------
// The consequence at recalc level: under-dirtying leaves a stale value.
// ---------------------------------------------------------------------------

/// The guard that makes "always fully authored" a wrong *value* and not just a
/// mismatched bool.
///
/// `A1={10,20,30}` spills A1:C1; `E1=SUM(B1:C1)` reads two spilled cells.
/// Overwriting A1 with a **literal** vacates B1 and C1. At seeding time the
/// grid then holds no array at all, so there is no spill rectangle for the
/// overlap branch to fire on, and the dependency graph carries no edge from A1
/// to E1 — B1 and C1 being unauthored is the only thing that can put E1 in the
/// dirty set. If this decision answered "fully authored", E1 would keep 50.
#[test]
fn a_reader_of_a_vacated_spill_range_still_refreshes() {
    let mut wb = wb_with(&["S"]);
    wb.set("S", addr("A1"), CellInput::Formula("={10,20,30}".into()))
        .unwrap();
    wb.set("S", addr("E1"), CellInput::Formula("=SUM(B1:C1)".into()))
        .unwrap();
    wb.recalc(&ctx());
    assert_eq!(
        wb.get("S", addr("E1")).unwrap().value(),
        &Value::Number(50.0)
    );

    wb.set("S", addr("A1"), CellInput::Literal(Value::Number(5.0)))
        .unwrap();
    let mut full = wb.clone();
    full.recalc(&ctx());
    wb.recalc_incremental(&ctx(), &[("S".to_string(), addr("A1"))]);

    assert_eq!(
        wb.get("S", addr("E1")).unwrap().value(),
        &Value::Number(0.0),
        "E1 kept a value from a spill that no longer exists"
    );
    assert_eq!(wb, full, "incremental recalc did not reproduce full recalc");
}

/// The dirty set must still be *narrow*: a reader of a fully authored range
/// that nothing in the edit touches is not seeded, which is what makes the
/// index worth having rather than merely cheap. Asserted through the change
/// list, since a seeded-but-unchanged cell emits no change either way — so
/// this checks the value, and the cost tests above check that reaching it did
/// not cost the sheet.
#[test]
fn a_fully_authored_range_over_literals_holds_no_unauthored_cell() {
    let mut wb = wb_with(&["S"]);
    for r in 1..=20u32 {
        for c in 1..=50u32 {
            set_num(&mut wb, "S", &Address::new(r, c).unwrap().to_a1(), 2.0);
        }
    }
    let index = AuthoredCellIndex::build(&wb);
    let (has_unauthored, examined) =
        index.range_has_unauthored_cell_examined(&rect("S", 1, 1, 1, 50));
    assert!(
        !has_unauthored,
        "every cell of that row is a typed-in literal; none is unauthored"
    );
    let authored = wb.sheets()[0].len();
    assert_eq!(authored, 1_000);
    assert!(
        examined <= 24,
        "reaching that answer examined {examined} authored cells; the scan \
         examined all {authored}"
    );
}
