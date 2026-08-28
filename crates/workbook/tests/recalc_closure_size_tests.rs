//! The exact-count metric behind incremental recalc: **how many cells end up in
//! the dirty closure per edit** (issue #925).
//!
//! ## Why a count and not a stopwatch
//!
//! Wall-clock is machine-dependent; the closure size is not. It is also the
//! thing that actually decides the cost — every cell in the closure is
//! re-evaluated, once per widen pass. A change that claims to narrow the dirty
//! set has to move this number, and a regression that re-broadens the seeding
//! shows up here as an exact integer rather than as noise in a benchmark.
//!
//! ## The three shapes
//!
//! Each is a shape a real model is full of, and each isolates one seeding rule:
//!
//!  * **named assumption** — one defined name read by every row. Before the
//!    narrowing, `Precedent::Name` was unconditionally spill-sensitive, so every
//!    reader of the name was dirty on every incremental recalc.
//!  * **sparse range** — a column with gaps, aggregated by a rolling `SUM`.
//!    Before the narrowing, *any* unauthored cell in a range seeded its reader,
//!    so every one of those aggregations was dirty on every recalc.
//!  * **spill-heavy** — array anchors and their readers, where the seeding is
//!    load-bearing and must **not** narrow.
//!
//! The counts are upper bounds (`<=`), not equalities, in the two narrowed
//! shapes: an over-dirtying regression must fail, but a future change that
//! narrows *further* should not have to edit this file. The spill shape asserts
//! a lower bound instead — under-dirtying there is the failure mode.

use truecalc_workbook::{
    Address, CellInput, EngineFlavor, RecalcContext, Value, Workbook, Worksheet,
};

fn ctx() -> RecalcContext {
    RecalcContext::new(1_780_000_000_000, "Etc/GMT", 0).expect("Etc/GMT is a valid tz")
}

fn addr(row: u32, col: u32) -> Address {
    Address::new(row, col).expect("in-bounds address")
}

const ROWS: u32 = 200;

// ---------------------------------------------------------------------------
// Shape 1: a named assumption read by every row.
// ---------------------------------------------------------------------------

/// `A1` is the assumption, `RATE` names it, and `C2..C200` each read `RATE`
/// alongside their own row's authored input. `E1`/`E2` are an unrelated pair;
/// editing `E1` is an edit that touches nothing the name feeds.
fn named_assumption_workbook() -> Workbook {
    let mut wb = Workbook::new(EngineFlavor::Sheets);
    wb.add_sheet(Worksheet::new("S")).unwrap();
    wb.set("S", addr(1, 1), CellInput::Literal(Value::Number(0.05)))
        .unwrap();
    wb.define_name("RATE", "S!A1").unwrap();
    for r in 2..=ROWS {
        wb.set("S", addr(r, 2), CellInput::Literal(Value::Number(r as f64)))
            .unwrap();
        wb.set("S", addr(r, 3), CellInput::Formula(format!("=B{r}*RATE")))
            .unwrap();
    }
    wb.set("S", addr(1, 5), CellInput::Literal(Value::Number(1.0)))
        .unwrap();
    wb.set("S", addr(2, 5), CellInput::Formula("=E1+1".into()))
        .unwrap();
    wb
}

// ---------------------------------------------------------------------------
// Shape 2: a rolling aggregation over a column with gaps.
// ---------------------------------------------------------------------------

/// Column `A` holds a value on even rows only — a sparse column, the ordinary
/// case in a real sheet. `C2..C190` roll a ten-row `SUM` over it. `A1` is an
/// assumption read only by `B1`, so editing it is an edit no aggregation
/// depends on.
fn sparse_range_workbook() -> Workbook {
    let mut wb = Workbook::new(EngineFlavor::Sheets);
    wb.add_sheet(Worksheet::new("S")).unwrap();
    wb.set("S", addr(1, 1), CellInput::Literal(Value::Number(3.0)))
        .unwrap();
    wb.set("S", addr(1, 2), CellInput::Formula("=A1*2".into()))
        .unwrap();
    for r in 2..=ROWS {
        if r % 2 == 0 {
            wb.set("S", addr(r, 1), CellInput::Literal(Value::Number(r as f64)))
                .unwrap();
        }
    }
    for r in 2..=(ROWS - 10) {
        wb.set(
            "S",
            addr(r, 3),
            CellInput::Formula(format!("=SUM(A{r}:A{})", r + 9)),
        )
        .unwrap();
    }
    wb
}

// ---------------------------------------------------------------------------
// Shape 3: spill-heavy. The seeding here is load-bearing.
// ---------------------------------------------------------------------------

/// Forty array anchors in column `A`, each spilling three rows, with a reader
/// of a spilled cell and a range reader per anchor. `H1`/`H2` are the unrelated
/// pair the edit touches.
fn spill_heavy_workbook() -> Workbook {
    let mut wb = Workbook::new(EngineFlavor::Sheets);
    wb.add_sheet(Worksheet::new("S")).unwrap();
    for i in 0..40u32 {
        let anchor = 1 + i * 4;
        wb.set(
            "S",
            addr(anchor, 1),
            CellInput::Formula(format!("={{{};{};{}}}", i + 1, i + 2, i + 3)),
        )
        .unwrap();
        // Reads a *spilled* cell (no dependency edge to the anchor).
        wb.set(
            "S",
            addr(anchor, 2),
            CellInput::Formula(format!("=A{}+1", anchor + 1)),
        )
        .unwrap();
        // Range-reads across the footprint.
        wb.set(
            "S",
            addr(anchor, 3),
            CellInput::Formula(format!("=SUM(A{anchor}:A{})", anchor + 2)),
        )
        .unwrap();
    }
    wb.set("S", addr(1, 8), CellInput::Literal(Value::Number(1.0)))
        .unwrap();
    wb.set("S", addr(2, 8), CellInput::Formula("=H1+1".into()))
        .unwrap();
    wb
}

// ---------------------------------------------------------------------------
// Measurement
// ---------------------------------------------------------------------------

/// Recalculates `wb` fully, then applies `reps` literal edits at `at`, and
/// returns the closure size of the last one plus the total wall clock of all of
/// them. The closure size is the assertable number; the duration is printed for
/// a human reading `--nocapture`, never asserted.
fn measure(
    wb: &mut Workbook,
    sheet: &str,
    at: Address,
    reps: usize,
) -> (usize, std::time::Duration) {
    wb.recalc(&ctx());
    let edited = [(sheet.to_owned(), at)];
    let mut closure = 0usize;
    let start = std::time::Instant::now();
    for i in 0..reps {
        wb.set(sheet, at, CellInput::Literal(Value::Number(i as f64)))
            .unwrap();
        closure = wb.recalc_incremental_measured(&ctx(), &edited).1;
    }
    (closure, start.elapsed())
}

fn report(shape: &str, closure: usize, elapsed: std::time::Duration, reps: usize) {
    println!(
        "{shape}: closure={closure} cells, {reps} edits in {:?} ({:?}/edit)",
        elapsed,
        elapsed / reps as u32
    );
}

#[test]
fn a_named_assumption_does_not_dirty_every_reader_of_the_name() {
    let mut wb = named_assumption_workbook();
    let (closure, elapsed) = measure(&mut wb, "S", addr(1, 5), 30);
    report("named-assumption", closure, elapsed, 30);
    // Only `E2` reads the edited cell. The 199 `RATE` readers must not be in
    // the closure: `RATE` targets an authored, non-spilling cell.
    assert!(
        closure <= 1,
        "editing an unrelated cell must not dirty the readers of a name whose \
         target is an ordinary authored cell; closure held {closure} cells"
    );
}

#[test]
fn a_sparse_column_aggregation_is_not_dirty_on_every_edit() {
    let mut wb = sparse_range_workbook();
    let (closure, elapsed) = measure(&mut wb, "S", addr(1, 1), 30);
    report("sparse-range", closure, elapsed, 30);
    // Only `B1` reads `A1`. The 189 rolling `SUM`s read ranges full of gaps,
    // but no spill can reach those gaps from `A1` — `A2` is authored, so an
    // array at `A1` would be blocked before it got there.
    assert!(
        closure <= 1,
        "a rolling SUM over a column with gaps must not be dirty just for \
         having gaps; closure held {closure} cells"
    );
}

#[test]
fn spill_seeding_still_covers_anchors_and_their_readers() {
    let mut wb = spill_heavy_workbook();
    let (closure, elapsed) = measure(&mut wb, "S", addr(1, 8), 30);
    report("spill-heavy", closure, elapsed, 30);
    // 40 anchors + 40 spilled-cell readers + 40 range readers, none of which
    // has a dependency edge from the edited cell — the seeding is what puts
    // them in the closure, and narrowing it away would leave stale values.
    assert!(
        closure >= 120,
        "spill-occupancy seeding must still cover every anchor and its \
         readers; closure held only {closure} cells"
    );
}
