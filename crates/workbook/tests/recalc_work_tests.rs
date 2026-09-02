//! Recalculation **work**, asserted as an exact count rather than timed.
//!
//! `crates/workbook/benches/workbook_perf.rs` measures how long a recalc of
//! each fixture shape takes; it never checks how much of the workbook that
//! recalc actually touched. So the failure mode that matters most for an
//! incremental engine — dirtying more cells than the edit requires — surfaces
//! only as a slightly slower benchmark, which reads as noise and gets
//! dismissed. Here the same property is an exact number: over-dirtying is a
//! hard failure that names the count it expected.
//!
//! This is the same instrument `depgraph_range_index_tests.rs` uses for
//! cells-examined-per-range-reference, pointed at recalculation instead:
//! counts taken from the run itself, so they hold on any machine and in either
//! build profile and cannot go flaky.
//!
//! Two counts are asserted per shape, because they are not the same number:
//!
//! * **the dirty closure** — how many formula cells an incremental recalc
//!   recomputed, from `Workbook::recalc_incremental_measured`. This is the
//!   work done.
//! * **the changes emitted** — `Vec<Change>`'s length. Recalc's write-back
//!   skips a cell whose recomputed value equals its stored one (`if old == new
//!   { continue; }`), so this is the number of formulas whose value *moved*,
//!   which is `<=` the closure and is not a measure of work. On a
//!   never-recalculated workbook every formula cell still stores
//!   `Value::Empty`, so there the two coincide and a first full recalc's
//!   change count is exactly the fixture's formula count.
//!
//! The fixtures mirror the benchmark file's builders, deliberately
//! reconstructed here rather than shared: a benchmark is free to change its
//! sizes and its shapes for measurement reasons, and these expectations are
//! pinned to the shape, not to whatever the bench happens to build today.
//! Sizes are small — these are correctness assertions, not performance runs.

use truecalc_workbook::{
    Address, CellInput, DependencyGraph, EngineFlavor, RecalcContext, Value, Workbook, Worksheet,
};

fn make_ctx() -> RecalcContext {
    RecalcContext::new(0, "UTC", 0).expect("UTC is valid")
}

fn new_workbook() -> Workbook {
    let mut wb = Workbook::new(EngineFlavor::Sheets);
    wb.add_sheet(Worksheet::new("Sheet1")).unwrap();
    wb
}

fn set_number(wb: &mut Workbook, row: u32, col: u32, n: f64) {
    let addr = Address::new(row, col).unwrap();
    wb.set("Sheet1", addr, CellInput::Literal(Value::Number(n)))
        .unwrap();
}

fn set_formula(wb: &mut Workbook, row: u32, col: u32, formula: String) {
    let addr = Address::new(row, col).unwrap();
    wb.set("Sheet1", addr, CellInput::Formula(formula)).unwrap();
}

// --- fixtures (the benchmark shapes) ----------------------------------------

/// N independent single-precedent formulas: `A{r}` a literal, `B{r} = =A{r}+1`.
fn build_independent(n: u32) -> Workbook {
    let mut wb = new_workbook();
    for row in 1..=n {
        set_number(&mut wb, row, 1, f64::from(row));
        set_formula(&mut wb, row, 2, format!("=A{row}+1"));
    }
    wb
}

/// N rows of 8 literal columns plus a `=SUM(A{r}:H{r})` row total in column I.
fn build_row_totals(n: u32) -> Workbook {
    let mut wb = new_workbook();
    for row in 1..=n {
        for col in 1..=8u32 {
            set_number(&mut wb, row, col, f64::from(row * col));
        }
        set_formula(&mut wb, row, 9, format!("=SUM(A{row}:H{row})"));
    }
    wb
}

/// N literals in column A, with a `=SUM(A{r}:A{r+99})` subtotal in column C
/// every 20 rows.
fn build_block_subtotals(n: u32) -> Workbook {
    let mut wb = new_workbook();
    for row in 1..=n {
        set_number(&mut wb, row, 1, f64::from(row));
    }
    let mut row = 1;
    while row + 99 <= n {
        set_formula(&mut wb, row, 3, format!("=SUM(A{row}:A{})", row + 99));
        row += 20;
    }
    wb
}

/// N rows, exactly one single-cell-range formula per row.
fn build_tall_sparse(n: u32) -> Workbook {
    let mut wb = new_workbook();
    for row in 1..=n {
        set_number(&mut wb, row, 1, f64::from(row));
        set_formula(&mut wb, row, 2, format!("=SUM(A{row}:A{row})"));
    }
    wb
}

/// `sheets` tabs, each `rows` rows of an `A{r}` literal and a `B{r} = =A{r}+1`
/// formula. Every reference is bare, so it reads the formula's own tab.
fn build_multi_sheet(sheets: usize, rows: u32) -> Workbook {
    let mut wb = Workbook::new(EngineFlavor::Sheets);
    for s in 0..sheets {
        wb.add_sheet(Worksheet::new(format!("S{s}"))).unwrap();
    }
    for s in 0..sheets {
        let sheet = wb.sheets()[s].name().to_owned();
        for row in 1..=rows {
            wb.set(
                &sheet,
                Address::new(row, 1).unwrap(),
                CellInput::Literal(Value::Number(f64::from(row))),
            )
            .unwrap();
            wb.set(
                &sheet,
                Address::new(row, 2).unwrap(),
                CellInput::Formula(format!("=A{row}+1")),
            )
            .unwrap();
        }
    }
    wb
}

/// [`build_multi_sheet`], but every formula is a qualified cross-sheet
/// reference into the first tab, so one literal there is read by one formula
/// on every tab.
fn build_multi_sheet_cross(sheets: usize, rows: u32) -> Workbook {
    let mut wb = Workbook::new(EngineFlavor::Sheets);
    for s in 0..sheets {
        wb.add_sheet(Worksheet::new(format!("S{s}"))).unwrap();
    }
    let source = wb.sheets()[0].name().to_owned();
    for s in 0..sheets {
        let sheet = wb.sheets()[s].name().to_owned();
        for row in 1..=rows {
            wb.set(
                &sheet,
                Address::new(row, 1).unwrap(),
                CellInput::Literal(Value::Number(f64::from(row))),
            )
            .unwrap();
            wb.set(
                &sheet,
                Address::new(row, 2).unwrap(),
                CellInput::Formula(format!("='{source}'!A{row}+1")),
            )
            .unwrap();
        }
    }
    wb
}

/// A single linear dependency chain of depth `n`, plus one tail literal.
///
/// `A1` is a literal, `A2 = =A1+1`, `A3 = =A2+1`, … through `A{n+1}` — `n`
/// formulas, each reading the one above it, so editing `A1` must propagate
/// through every one of them. This is the shape the benchmark suite had no
/// fixture for: every other builder here is one level deep.
///
/// The last link is `A{n+1} = =A{n}+B{n+1}`, with `B{n+1}` a literal read by
/// nothing else. That single extra literal is what makes a *cheap* edit
/// expressible on this fixture: in an otherwise pure chain the only
/// non-formula cell is `A1`, and editing it dirties everything. Writing a
/// literal over a chain cell instead would destroy a formula node and
/// invalidate the dependency-graph cache, so the cheap-edit measurement would
/// quietly become a graph rebuild rather than a one-cell recompute. Editing
/// `B{n+1}` dirties exactly one formula over the same warm graph the root edit
/// uses, so the two edits differ only in how far the dirt travels.
fn build_chain(n: u32) -> Workbook {
    let mut wb = new_workbook();
    set_number(&mut wb, 1, 1, 1.0);
    let last = n + 1;
    set_number(&mut wb, last, 2, 1.0);
    for row in 2..=last {
        let prev = row - 1;
        if row == last {
            set_formula(&mut wb, row, 1, format!("=A{prev}+B{last}"));
        } else {
            set_formula(&mut wb, row, 1, format!("=A{prev}+1"));
        }
    }
    wb
}

// --- work probes ------------------------------------------------------------

/// Formula cells in `wb`, and the changes a first full recalc emits.
///
/// On a workbook that has never been recalculated the two are equal: every
/// formula cell still stores `Value::Empty`, so every recomputed cell is also
/// a changed one.
fn full_recalc_work(wb: &Workbook) -> (usize, usize) {
    let formula_cells = DependencyGraph::build(wb).formula_cells().count();
    let mut wb = wb.clone();
    let changes = wb.recalc(&make_ctx()).len();
    (formula_cells, changes)
}

/// Fully recalculates `wb`, writes `value` into `sheet!addr`, then recalculates
/// incrementally: returns (dirty-closure size, changes emitted).
fn incremental_work(wb: &Workbook, sheet: &str, addr: Address, value: f64) -> (usize, usize) {
    let ctx = make_ctx();
    let mut wb = wb.clone();
    wb.recalc(&ctx);
    wb.set(sheet, addr, CellInput::Literal(Value::Number(value)))
        .unwrap();
    let (changes, closure) = wb.recalc_incremental_measured(&ctx, &[(sheet.to_string(), addr)]);
    (closure, changes.len())
}

fn a1() -> Address {
    Address::new(1, 1).unwrap()
}

// --- one test per benchmark shape -------------------------------------------

#[test]
fn independent_recalculates_one_formula_per_row_and_dirties_one_per_edit() {
    let wb = build_independent(50);

    // 50 rows x 1 formula per row (column B) = 50.
    assert_eq!(full_recalc_work(&wb), (50, 50));

    // A1 is read by B1 and by nothing else, so the closure is that one cell.
    assert_eq!(incremental_work(&wb, "Sheet1", a1(), 99.0), (1, 1));
}

/// Design A (issue #991): the pre-image map `recalc_incremental_measured`
/// accumulates should hold exactly the cells the call actually touched, not
/// every formula cell in the workbook — the whole point of replacing the old
/// up-front `snapshot_formula_values`. Wall clock cannot prove this; an exact
/// count can, the same rationale `full_recalc_work`/`incremental_work` above
/// give for theirs.
#[test]
fn a_one_cell_edit_into_a_thousand_formula_workbook_records_one_pre_image() {
    let ctx = make_ctx();
    let mut wb = build_independent(1000);
    wb.recalc(&ctx);
    wb.set("Sheet1", a1(), CellInput::Literal(Value::Number(99.0)))
        .unwrap();
    let changes = wb.recalc_incremental(&ctx, &[("Sheet1".to_string(), a1())]);
    assert_eq!(changes.len(), 1, "A1 is read by B1 and by nothing else");
    assert_eq!(
        wb.pre_image_count(),
        1,
        "a one-cell edit into a 1,000-formula workbook must record exactly \
         one pre-image, not one per formula cell"
    );
}

#[test]
fn row_totals_recalculates_one_total_per_row_and_dirties_one_per_edit() {
    let wb = build_row_totals(20);

    // 20 rows x 1 total per row (column I) = 20.
    assert_eq!(full_recalc_work(&wb), (20, 20));

    // A1 is inside exactly one row total's range, `I1 = SUM(A1:H1)`.
    assert_eq!(incremental_work(&wb, "Sheet1", a1(), 99.0), (1, 1));
}

/// The subtotal windows are `A{r}:A{r+99}` at `r = 1, 21, 41, …`, so how many
/// cover a given source row depends on where that row sits: five is the
/// maximum (reached once five windows have opened above it and none has closed
/// — rows 101..=120 at n=200), and row 1 sits inside exactly **one**, because
/// no window starts above it. The benchmark that edits A1 on this fixture is
/// therefore measuring the *cheapest* edit the shape admits, not the five-way
/// fan-out its builder's docstring describes; the interior edit below is the
/// one that exercises overlapping range-precedent invalidation.
#[test]
fn block_subtotals_dirty_only_the_windows_covering_the_edited_row() {
    let wb = build_block_subtotals(200);

    // Subtotals sit at rows r = 1, 21, 41, 61, 81, 101 — the r in the
    // arithmetic sequence stepping by 20 with r + 99 <= 200, i.e. r <= 101,
    // which is 6 formulas.
    assert_eq!(full_recalc_work(&wb), (6, 6));

    // Row 1 is covered only by A1:A100.
    assert_eq!(incremental_work(&wb, "Sheet1", a1(), 99.0), (1, 1));

    // Row 101 is covered by the windows starting at 21, 41, 61, 81 and 101
    // (r <= 101 <= r + 99 means 2 <= r <= 101) = 5.
    let a101 = Address::new(101, 1).unwrap();
    assert_eq!(incremental_work(&wb, "Sheet1", a101, 99.0), (5, 5));
}

#[test]
fn tall_sparse_recalculates_one_formula_per_row_and_dirties_one_per_edit() {
    let wb = build_tall_sparse(30);

    // 30 rows x 1 formula per row (column B) = 30.
    assert_eq!(full_recalc_work(&wb), (30, 30));

    // `B1 = SUM(A1:A1)` is the only reader of A1.
    assert_eq!(incremental_work(&wb, "Sheet1", a1(), 99.0), (1, 1));
}

#[test]
fn multi_sheet_bare_refs_keep_an_edit_on_its_own_tab() {
    let wb = build_multi_sheet(4, 10);

    // 4 tabs x 10 rows x 1 formula per row = 40.
    assert_eq!(full_recalc_work(&wb), (40, 40));

    // Every reference is bare, so S0!A1 is read only by S0!B1 — the other
    // three tabs are untouched however many there are.
    assert_eq!(incremental_work(&wb, "S0", a1(), 99.0), (1, 1));
}

/// The cross-sheet fixture is the suite's only high-fan-out shape: one literal
/// on the first tab is read by one formula on *every* tab. It is still one
/// level deep — the formulas read a literal, and nothing reads the formulas —
/// which is why it does not substitute for a chain.
#[test]
fn multi_sheet_cross_refs_dirty_one_formula_per_tab() {
    let wb = build_multi_sheet_cross(4, 10);

    // 4 tabs x 10 rows x 1 formula per row = 40.
    assert_eq!(full_recalc_work(&wb), (40, 40));

    // S0!A1 is read by B1 on each of the 4 tabs = 4, and by nothing else.
    assert_eq!(incremental_work(&wb, "S0", a1(), 99.0), (4, 4));
}

/// The shape the suite had no fixture for. A root edit must propagate the whole
/// depth; an edit at the far end must not travel at all. Between them they
/// bracket what propagation costs, and the gap between them is exactly what an
/// over-dirtying regression would close.
#[test]
fn a_chain_propagates_a_root_edit_end_to_end_and_a_leaf_edit_nowhere() {
    let n = 50u32;
    let wb = build_chain(n);

    // A2..A51 = 50 formulas (the chain), fed by 2 literals (A1 and B51).
    assert_eq!(full_recalc_work(&wb), (50, 50));

    // Worst case: A1 is read by A2, which is read by A3, … every formula in
    // the chain is downstream of the root, so the closure is all 50 — and
    // every one of their values shifts by the same delta, so all 50 change.
    assert_eq!(incremental_work(&wb, "Sheet1", a1(), 99.0), (50, 50));

    // Best case: B51 is read only by the last link, so one formula recomputes
    // however deep the chain above it is.
    let tail = Address::new(n + 1, 2).unwrap();
    assert_eq!(incremental_work(&wb, "Sheet1", tail, 99.0), (1, 1));
}

/// A chain of a few thousand links, to check that depth alone costs nothing
/// unbounded: evaluation order (Kahn's algorithm) and the dirty-closure walk
/// are both iterative, and a formula reads its precedents' *stored* values
/// rather than recursing into them, so no stage of a recalc recurses with the
/// chain's depth. A deep chain is the shape that would find it if one did.
#[test]
fn a_deep_chain_recalculates_without_recursing_on_its_depth() {
    let n = 5_000u32;
    let wb = build_chain(n);

    assert_eq!(full_recalc_work(&wb), (5_000, 5_000));
    assert_eq!(incremental_work(&wb, "Sheet1", a1(), 99.0), (5_000, 5_000));

    let tail = Address::new(n + 1, 2).unwrap();
    assert_eq!(incremental_work(&wb, "Sheet1", tail, 99.0), (1, 1));
}

/// `incremental ≡ full` on these exact fixture shapes.
///
/// The guarantee itself is proven generically by
/// `recalc_incremental_property_tests.rs` and `recalc_differential_tests.rs`,
/// which generate random workbooks and compare whole grids. What those
/// generators do not reach is this file's shapes at this file's sizes — their
/// grids are at most a dozen cells on one sheet, so a 50-deep chain, a
/// four-tab cross-reference fan-out and overlapping 100-row subtotal windows
/// are all outside what they can draw. Compared as canonical JSON, so every
/// stored value on every sheet is included.
#[test]
fn an_incremental_recalc_of_each_shape_agrees_with_a_full_one() {
    let ctx = make_ctx();
    let cases: Vec<(&str, Workbook, &str, Address)> = vec![
        ("independent", build_independent(50), "Sheet1", a1()),
        ("row_totals", build_row_totals(20), "Sheet1", a1()),
        (
            "block_subtotals",
            build_block_subtotals(200),
            "Sheet1",
            a1(),
        ),
        (
            "block_subtotals interior",
            build_block_subtotals(200),
            "Sheet1",
            Address::new(101, 1).unwrap(),
        ),
        ("tall_sparse", build_tall_sparse(30), "Sheet1", a1()),
        ("multi_sheet", build_multi_sheet(4, 10), "S0", a1()),
        (
            "multi_sheet_cross",
            build_multi_sheet_cross(4, 10),
            "S0",
            a1(),
        ),
        ("chain root edit", build_chain(50), "Sheet1", a1()),
        (
            "chain leaf edit",
            build_chain(50),
            "Sheet1",
            Address::new(51, 2).unwrap(),
        ),
    ];

    for (label, mut wb, sheet, addr) in cases {
        wb.recalc(&ctx);
        wb.set(sheet, addr, CellInput::Literal(Value::Number(99.0)))
            .unwrap();
        let mut full = wb.clone();
        full.recalc(&ctx);
        wb.recalc_incremental(&ctx, &[(sheet.to_string(), addr)]);
        assert_eq!(
            wb.to_json().unwrap(),
            full.to_json().unwrap(),
            "{label}: incremental and full grids diverged after editing {sheet}!{}",
            addr.to_a1()
        );
    }
}
