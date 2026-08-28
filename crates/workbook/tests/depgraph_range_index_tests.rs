//! Range-precedent lookup: cost and equivalence (issue #908).
//!
//! Answering "which formula cells does this range cover?" used to scan **every
//! formula cell in the workbook**, so a workbook of `N` formulas with `R` range
//! references cost `O(R * N)` — the dominant term in dependency-graph
//! construction, which is itself rebuilt on every recalculation. The graph now
//! indexes formula cells by sheet and row, so a range visits only the formula
//! cells on the rows it spans.
//!
//! These tests pin the change in **formula cells examined per range
//! reference** — an exact count taken from the lookup itself, so it holds on
//! any machine and in either build profile and cannot go flaky. Wall-clock
//! numbers belong in the PR discussion, not in a test.
//!
//! Correctness, not speed, is the real risk: an index that returns a subtly
//! different set than the scan would change evaluation order and therefore
//! results. The scan is kept here as an oracle and asserted to agree with the
//! indexed lookup, precedent by precedent, over a set of workbooks that
//! includes the awkward cases.

use truecalc_workbook::{
    Address, Cell, CellRef, DependencyGraph, EngineFlavor, NameTarget, NamedRange, Precedent,
    RangeRef, Value, Workbook, Worksheet,
};

fn addr(a1: &str) -> Address {
    Address::from_a1(a1).unwrap()
}

fn wb_one_sheet() -> Workbook {
    let mut wb = Workbook::new(EngineFlavor::Sheets);
    wb.add_sheet(Worksheet::new("Sheet1")).unwrap();
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

/// The pre-index lookup, kept verbatim as an oracle: test every formula cell
/// in the workbook for membership. Whatever the index returns must equal this.
fn scan_oracle(graph: &DependencyGraph, prec: &Precedent) -> Vec<CellRef> {
    let in_range = |r: &RangeRef| -> Vec<CellRef> {
        graph
            .formula_cells()
            .filter(|c| r.contains(c))
            .cloned()
            .collect()
    };
    match prec {
        Precedent::Cell(c) => {
            if graph.is_formula(c) {
                vec![c.clone()]
            } else {
                Vec::new()
            }
        }
        Precedent::Range(r) => in_range(r),
        Precedent::Name(name) => match graph.name_target_of(name) {
            Some(NameTarget::Cell(c)) if graph.is_formula(&c) => vec![c],
            Some(NameTarget::Cell(_)) | None => Vec::new(),
            Some(NameTarget::Range(r)) => in_range(&r),
        },
        Precedent::Unresolved(_) => Vec::new(),
    }
}

/// Asserts the indexed lookup returns exactly what the scan returns, for every
/// precedent of every formula cell in `wb`.
fn assert_matches_scan(label: &str, wb: &Workbook) {
    let graph = DependencyGraph::build(wb);
    let mut precedents = 0usize;
    for cell in graph.formula_cells() {
        for prec in graph.precedents_of(cell).unwrap() {
            precedents += 1;
            assert_eq!(
                graph.formula_precedent_cells(prec),
                scan_oracle(&graph, prec),
                "{label}: indexed lookup differs from the scan for {prec:?} (read by {cell:?})"
            );
        }
    }
    assert!(
        precedents > 0 || graph.formula_cells().next().is_none(),
        "{label}: nothing was compared"
    );
}

/// A monthly-operating-model shape: an assumptions cell, twelve columns of
/// growth-chain formulas, a `SUM` row total per row, and a block subtotal every
/// 100 rows. `blocks * 100` data rows, so the row/block ratio — and therefore
/// the expected cells-examined-per-range-reference — does not shift with size.
fn operating_model(blocks: u32) -> Workbook {
    let mut wb = wb_one_sheet();
    set_num(&mut wb, "Sheet1", "A1", 0.05);
    let rows = blocks * 100;
    for i in 0..rows {
        let r = i + 2; // data starts on row 2
        for c in 0..12u32 {
            let col = Address::new(r, 2 + c).unwrap().to_a1();
            if i == 0 {
                set_formula(&mut wb, "Sheet1", &col, "=$A$1*100");
            } else {
                let prev = Address::new(r - 1, 2 + c).unwrap().to_a1();
                set_formula(&mut wb, "Sheet1", &col, &format!("={prev}*(1+$A$1)"));
            }
        }
        let total = Address::new(r, 14).unwrap().to_a1();
        let first = Address::new(r, 2).unwrap().to_a1();
        let last = Address::new(r, 13).unwrap().to_a1();
        set_formula(&mut wb, "Sheet1", &total, &format!("=SUM({first}:{last})"));

        if (i + 1) % 100 == 0 {
            let sub = Address::new(r, 15).unwrap().to_a1();
            let top = Address::new(r - 99, 14).unwrap().to_a1();
            set_formula(&mut wb, "Sheet1", &sub, &format!("=SUM({top}:{total})"));
        }
    }
    wb
}

/// Formula cells examined across every range reference in the workbook, and
/// how many range references there were.
fn examined_per_range_reference(wb: &Workbook) -> (f64, usize, usize) {
    let graph = DependencyGraph::build(wb);
    let mut examined = 0usize;
    let mut refs = 0usize;
    for cell in graph.formula_cells() {
        for prec in graph.precedents_of(cell).unwrap() {
            if !matches!(prec, Precedent::Range(_)) {
                continue;
            }
            refs += 1;
            examined += graph.formula_precedent_cells_examined(prec).1;
        }
    }
    let formula_cells = graph.formula_cells().count();
    #[allow(clippy::cast_precision_loss)]
    let per_ref = examined as f64 / refs as f64;
    (per_ref, refs, formula_cells)
}

/// The headline property: the cost of one range reference does not grow with
/// the workbook. Before the index it was exactly the formula-cell count, so
/// this number was the model size.
#[test]
fn cells_examined_per_range_reference_is_flat_as_the_model_grows() {
    let mut measurements = Vec::new();
    for blocks in [1u32, 2, 4, 8] {
        let wb = operating_model(blocks);
        let (per_ref, refs, formula_cells) = examined_per_range_reference(&wb);
        measurements.push((formula_cells, refs, per_ref));
    }

    let (smallest_cells, _, baseline) = measurements[0];
    let (largest_cells, _, largest_per_ref) = *measurements.last().unwrap();
    assert_eq!(largest_cells, smallest_cells * 8, "model did not scale 8x");

    for &(formula_cells, refs, per_ref) in &measurements {
        assert!(
            (per_ref - baseline).abs() < 0.5,
            "cells examined per range reference moved with model size: \
             {per_ref} at {formula_cells} formula cells ({refs} range refs), \
             {baseline} at {smallest_cells}; all measurements: {measurements:?}"
        );
    }

    // And it is not merely flat, it is small: the scan examined one cell per
    // formula cell in the workbook.
    #[allow(clippy::cast_precision_loss)]
    let scan_cost = largest_cells as f64;
    assert!(
        largest_per_ref < scan_cost / 100.0,
        "expected far fewer than the {scan_cost} cells the scan examined, got {largest_per_ref}"
    );
}

/// The trap: a whole-column-shaped reference covers ten million rows, almost
/// all of them empty. The index must key only rows that actually hold formula
/// cells, so the empty rows cost nothing — an eighteen-thousand-row-tall
/// reference examines exactly what a ten-row one does when the rows between
/// hold nothing.
#[test]
fn a_tall_range_over_empty_rows_costs_nothing_extra() {
    let mut wb = wb_one_sheet();
    set_num(&mut wb, "Sheet1", "A1", 1.0);
    for r in 2..=10u32 {
        set_formula(
            &mut wb,
            "Sheet1",
            &Address::new(r, 1).unwrap().to_a1(),
            "=A1",
        );
    }
    let graph = DependencyGraph::build(&wb);

    let over = |last_row: u32| -> (Vec<CellRef>, usize) {
        graph.formula_precedent_cells_examined(&Precedent::Range(RangeRef {
            sheet: "sheet1".to_string(),
            start: addr("A1"),
            end: Address::new(last_row, 1).unwrap(),
        }))
    };

    let (ten_rows, ten_examined) = over(10);
    for last_row in [18_000u32, 1_000_000, 10_000_000] {
        let (cells, examined) = over(last_row);
        assert_eq!(cells, ten_rows, "A1:A{last_row} covered a different set");
        assert_eq!(
            examined, ten_examined,
            "A1:A{last_row} examined {examined} cells, but A1:A10 examined {ten_examined} — \
             the index is keying empty rows"
        );
    }
    assert_eq!(
        ten_examined, 9,
        "only the nine formula cells should be seen"
    );
}

/// A range that spans rows holding no formulas at all examines nothing.
#[test]
fn a_range_over_rows_with_no_formulas_examines_nothing() {
    let mut wb = wb_one_sheet();
    set_formula(&mut wb, "Sheet1", "A1", "=1");
    for r in 100..=110u32 {
        set_num(&mut wb, "Sheet1", &Address::new(r, 1).unwrap().to_a1(), 1.0);
    }
    let graph = DependencyGraph::build(&wb);
    let (cells, examined) = graph.formula_precedent_cells_examined(&Precedent::Range(RangeRef {
        sheet: "sheet1".to_string(),
        start: addr("A100"),
        end: addr("A110"),
    }));
    assert!(cells.is_empty());
    assert_eq!(examined, 0);
}

/// `RangeRef`'s fields are public, so a range can reach the lookup with its
/// corners the wrong way round. `contains` calls such a range empty; so must
/// the index — and it must not panic on the inverted bound.
#[test]
fn an_inverted_range_is_empty_just_as_the_membership_test_says() {
    let mut wb = wb_one_sheet();
    for r in 1..=5u32 {
        set_formula(
            &mut wb,
            "Sheet1",
            &Address::new(r, 1).unwrap().to_a1(),
            "=1",
        );
    }
    let graph = DependencyGraph::build(&wb);
    for (start, end) in [("A5", "A1"), ("C1", "A1"), ("C5", "A1")] {
        let range = RangeRef {
            sheet: "sheet1".to_string(),
            start: addr(start),
            end: addr(end),
        };
        assert!(
            graph.formula_cells().all(|c| !range.contains(c)),
            "{start}:{end} should contain nothing"
        );
        assert_eq!(
            graph.formula_precedent_cells(&Precedent::Range(range)),
            Vec::<CellRef>::new(),
            "{start}:{end} should look up as empty"
        );
    }
}

/// Equivalence with the scan across the awkward cases: empty ranges,
/// single-cell ranges, cross-sheet ranges, ranges over formula-free rows, a
/// range containing the reading cell itself, name indirection (to a cell, to a
/// range, and dangling), and unparseable formulas.
#[test]
fn indexed_lookup_equals_the_scan_on_edge_cases() {
    // An empty workbook: no formula cells at all.
    assert_matches_scan("empty workbook", &wb_one_sheet());

    // Single-cell range, self-referential range, formula-free range.
    let mut wb = wb_one_sheet();
    set_num(&mut wb, "Sheet1", "A1", 1.0);
    set_formula(&mut wb, "Sheet1", "B1", "=SUM(A1:A1)");
    set_formula(&mut wb, "Sheet1", "B2", "=SUM(B1:B3)"); // contains its own cell
    set_formula(&mut wb, "Sheet1", "B3", "=SUM(A10:A20)"); // rows with no formulas
    set_formula(&mut wb, "Sheet1", "B4", "=SUM(Z900:AB1000)"); // nothing at all there
    set_formula(&mut wb, "Sheet1", "B5", "=SUM(D5:B1)"); // corners the wrong way round
    set_formula(&mut wb, "Sheet1", "B6", "=this is not a formula");
    assert_matches_scan("single sheet, awkward ranges", &wb);

    // Cross-sheet ranges, including a range on a sheet whose rows are
    // populated at the same addresses as another sheet's.
    let mut wb = wb_one_sheet();
    wb.add_sheet(Worksheet::new("Data")).unwrap();
    for r in 1..=5u32 {
        set_formula(&mut wb, "Data", &Address::new(r, 1).unwrap().to_a1(), "=1");
        set_formula(
            &mut wb,
            "Sheet1",
            &Address::new(r, 1).unwrap().to_a1(),
            "=2",
        );
    }
    set_formula(&mut wb, "Sheet1", "C1", "=SUM(Data!A1:A5)");
    set_formula(&mut wb, "Sheet1", "C2", "=SUM(A1:A5)");
    set_formula(&mut wb, "Sheet1", "C3", "=SUM(Nope!A1:A5)"); // unknown sheet
    assert_matches_scan("cross-sheet ranges", &wb);

    // Name indirection: to a range, to a cell, and dangling.
    let mut wb = wb_one_sheet();
    for r in 1..=5u32 {
        set_formula(
            &mut wb,
            "Sheet1",
            &Address::new(r, 1).unwrap().to_a1(),
            "=1",
        );
    }
    wb.names_mut().push(NamedRange {
        name: "Block".to_string(),
        r#ref: "Sheet1!A1:A5".to_string(),
    });
    wb.names_mut().push(NamedRange {
        name: "One".to_string(),
        r#ref: "Sheet1!A1".to_string(),
    });
    set_formula(&mut wb, "Sheet1", "C1", "=SUM(Block)");
    set_formula(&mut wb, "Sheet1", "C2", "=One+1");
    set_formula(&mut wb, "Sheet1", "C3", "=Missing+1");
    assert_matches_scan("named ranges", &wb);

    // A larger, structured model — the shape the metric is measured on.
    assert_matches_scan("operating model", &operating_model(2));
}

/// The graph is a derived value: two builds of the same workbook must be equal,
/// index included.
#[test]
fn the_index_does_not_break_build_determinism() {
    let wb = operating_model(1);
    assert_eq!(DependencyGraph::build(&wb), DependencyGraph::build(&wb));
}
