//! Criterion benchmarks backing the CI performance regression gate.
//!
//! Two families of fixtures are measured:
//!
//! * **Cell-precedent fixtures** (`independent`) — formulas whose precedents
//!   are single cells. Cheap to build a dependency graph for.
//! * **Range-precedent fixtures** (`row_totals`, `block_subtotals`,
//!   `tall_sparse`) — formulas whose precedents are ranges. These drive the
//!   range-overlap index, which is a different (and historically much hotter)
//!   code path than single-cell precedents.
//! * **Multi-sheet fixtures** (`multi_sheet`, `multi_sheet_long_names`) — the
//!   same cell shape spread across many tabs. Every other fixture here builds
//!   one `Sheet1`, and that blind spot is exactly why an
//!   `O(cells × sheets × sheet-name-length)` sheet lookup — 90% of a 200-sheet
//!   recalc — stayed invisible to this suite for as long as it did (issue
//!   #952). Real models are many-sheet by nature; 200 tabs is a small financial
//!   model, not a stress test. `multi_sheet_long_names` is the same workbook
//!   with realistic tab names (`Cash Flow Statement 2027`) rather than `S17`,
//!   because sheet-name length was itself a performance parameter: naming tabs
//!   the way people actually name them cost 3.6x the wall clock. The two must
//!   now measure the same.
//!
//! `calibration/hash_alloc` is deliberately *not* a workbook benchmark. It is a
//! fixed allocate-and-hash workload used as a machine-speed probe: the
//! regression gate compares each benchmark against its baseline in units of
//! this reference, so that baselines recorded on one machine remain meaningful
//! on another. See `baselines.json` and `.github/scripts/check_perf_regression.py`.

use std::collections::HashMap;
use std::hint::black_box;
use std::time::Duration;

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
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

/// N *independent* single-precedent formulas.
///
/// Column A: literal row number. Column B: `=A{row}+1`.
///
/// Note this is deliberately **not** a dependency chain — every B cell depends
/// on its own A cell and on nothing else, so the recalc order is arbitrary and
/// editing A1 dirties exactly one formula. (It was named `chain` until #923;
/// the name overstated what it measured.)
fn build_independent(n: u32) -> Workbook {
    let mut wb = new_workbook();
    for row in 1..=n {
        set_number(&mut wb, row, 1, row as f64);
        set_formula(&mut wb, row, 2, format!("=A{row}+1"));
    }
    wb
}

/// N rows of 8 literal columns plus a `=SUM(A{r}:H{r})` row total in column I.
///
/// One range precedent per formula, each spanning a single row.
fn build_row_totals(n: u32) -> Workbook {
    let mut wb = new_workbook();
    for row in 1..=n {
        for col in 1..=8u32 {
            set_number(&mut wb, row, col, (row * col) as f64);
        }
        set_formula(&mut wb, row, 9, format!("=SUM(A{row}:H{row})"));
    }
    wb
}

/// N rows of literals in column A, with a `=SUM(A{r}:A{r+99})` block subtotal
/// in column C every 20 rows.
///
/// Range precedents that are *tall* (100 rows) and heavily overlapping — five
/// subtotals cover any given source row.
fn build_block_subtotals(n: u32) -> Workbook {
    let mut wb = new_workbook();
    for row in 1..=n {
        set_number(&mut wb, row, 1, row as f64);
    }
    let mut row = 1;
    while row + 99 <= n {
        set_formula(&mut wb, row, 3, format!("=SUM(A{row}:A{})", row + 99));
        row += 20;
    }
    wb
}

/// A pathological tall-sparse shape: N rows, exactly one formula per row.
///
/// The range-overlap index keys one map node and one `Vec` per *occupied* row,
/// so this shape maximises index construction cost per formula. Recorded so
/// that characteristic stays visible instead of being rediscovered later.
fn build_tall_sparse(n: u32) -> Workbook {
    let mut wb = new_workbook();
    for row in 1..=n {
        set_number(&mut wb, row, 1, row as f64);
        set_formula(&mut wb, row, 2, format!("=SUM(A{row}:A{row})"));
    }
    wb
}

/// `sheets` tabs, each `rows` rows of an `A{r}` literal and a `B{r} = =A{r}+1`
/// formula, with tab names produced by `name`.
///
/// Deliberately the *same* per-cell shape as `independent`, so the only
/// difference between the two is how the cells are distributed across tabs.
fn build_multi_sheet(sheets: usize, rows: u32, name: impl Fn(usize) -> String) -> Workbook {
    let mut wb = Workbook::new(EngineFlavor::Sheets);
    for s in 0..sheets {
        wb.add_sheet(Worksheet::new(name(s))).unwrap();
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

/// Terse tab names, the shape a benchmark author reaches for.
fn short_sheet_name(i: usize) -> String {
    format!("S{i}")
}

/// Tab names of the length people actually use in a financial model.
fn long_sheet_name(i: usize) -> String {
    format!("Cash Flow Statement 20{i:02}")
}

/// [`build_multi_sheet`], but every formula is a **qualified cross-sheet**
/// reference into the first tab (`='Cash Flow Statement 2000'!A{r}+1`).
///
/// A bare reference names no sheet, so it exercises none of the sheet lookup
/// either during graph build (which resolves each reference's target sheet) or
/// during evaluation (which resolves it again per read). Only a qualified
/// reference does, and a many-sheet model is full of them.
fn build_multi_sheet_cross(sheets: usize, rows: u32, name: impl Fn(usize) -> String) -> Workbook {
    let mut wb = Workbook::new(EngineFlavor::Sheets);
    for s in 0..sheets {
        wb.add_sheet(Worksheet::new(name(s))).unwrap();
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

fn bench_full_recalc(c: &mut Criterion) {
    let ctx = make_ctx();

    let mut group = c.benchmark_group("full_recalc/independent");
    for n in [100u32, 1000, 5000] {
        let template = build_independent(n);
        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, _| {
            b.iter(|| {
                let mut wb = template.clone();
                wb.recalc(&ctx)
            });
        });
    }
    group.finish();

    let mut group = c.benchmark_group("full_recalc/row_totals");
    group.sample_size(20);
    for n in [500u32, 2000] {
        let template = build_row_totals(n);
        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, _| {
            b.iter(|| {
                let mut wb = template.clone();
                wb.recalc(&ctx)
            });
        });
    }
    group.finish();

    let mut group = c.benchmark_group("full_recalc/block_subtotals");
    group.sample_size(20);
    for n in [2000u32, 10000] {
        let template = build_block_subtotals(n);
        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, _| {
            b.iter(|| {
                let mut wb = template.clone();
                wb.recalc(&ctx)
            });
        });
    }
    group.finish();

    // 200 tabs x 50 rows = 10,000 formula cells, the same cell shape as
    // `independent/5000` twice over, only spread across tabs.
    for (label, name) in [
        (
            "full_recalc/multi_sheet",
            short_sheet_name as fn(usize) -> String,
        ),
        ("full_recalc/multi_sheet_long_names", long_sheet_name),
    ] {
        let mut group = c.benchmark_group(label);
        group.sample_size(20);
        let template = build_multi_sheet(200, 50, name);
        group.bench_function("200x50", |b| {
            b.iter(|| {
                let mut wb = template.clone();
                wb.recalc(&ctx)
            });
        });
        group.finish();
    }

    // Same many-sheet workbook, but every formula names its sheet, so
    // evaluation resolves a sheet name on every read.
    let mut group = c.benchmark_group("full_recalc/multi_sheet_cross_refs");
    group.sample_size(20);
    let template = build_multi_sheet_cross(200, 50, long_sheet_name);
    group.bench_function("200x50", |b| {
        b.iter(|| {
            let mut wb = template.clone();
            wb.recalc(&ctx)
        });
    });
    group.finish();

    let mut group = c.benchmark_group("full_recalc/tall_sparse");
    group.sample_size(10);
    group.measurement_time(Duration::from_secs(10));
    let n = 20000u32;
    let template = build_tall_sparse(n);
    group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, _| {
        b.iter(|| {
            let mut wb = template.clone();
            wb.recalc(&ctx)
        });
    });
    group.finish();
}

/// Dependency-graph construction in isolation.
///
/// `full_recalc` includes graph construction, but dilutes it with evaluation.
/// These cases pin the build cost of the range-overlap index directly.
fn bench_depgraph_build(c: &mut Criterion) {
    let mut group = c.benchmark_group("depgraph_build");
    group.sample_size(10);
    group.measurement_time(Duration::from_secs(5));

    let tall = build_tall_sparse(20000);
    group.bench_function("tall_sparse/20000", |b| {
        b.iter(|| DependencyGraph::build(black_box(&tall)));
    });

    let totals = build_row_totals(2000);
    group.bench_function("row_totals/2000", |b| {
        b.iter(|| DependencyGraph::build(black_box(&totals)));
    });

    // Graph build resolves each reference's target sheet, so it carried the
    // same sheet-lookup term the recalc path did (issue #952). It only carries
    // it for *qualified* references, so this fixture uses them - a bare-ref
    // many-sheet workbook measures nothing here.
    let multi = build_multi_sheet_cross(200, 50, long_sheet_name);
    group.bench_function("multi_sheet_cross_refs/200x50", |b| {
        b.iter(|| DependencyGraph::build(black_box(&multi)));
    });

    group.finish();
}

fn bench_incremental_recalc(c: &mut Criterion) {
    let ctx = make_ctx();

    // Editing A1 in the independent fixture dirties exactly one formula (B1).
    // This guards the *minimality* of the dirty set, not chain propagation.
    //
    // `template` is pre-recalculated, so its graph cache is warm, and
    // `Workbook::clone` inherits the cache `Arc` (graph_cache module docs) -
    // every iteration below starts warm and never rebuilds the graph. That is
    // deliberate: it isolates the dirty-set question from graph-build cost,
    // which `depgraph_build` above already covers on its own. It is *not* a
    // measurement of a host's first incremental recalc after loading a
    // workbook and editing a cell - that path is cold, and is measured
    // separately by `incremental_recalc_cold` below. Do not read a change
    // here as "incremental recalc got faster" without checking which of the
    // two moved.
    let mut group = c.benchmark_group("incremental_recalc/independent_edit_root");
    for n in [100u32, 1000] {
        let mut template = build_independent(n);
        template.recalc(&ctx);
        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, _| {
            b.iter(|| {
                let mut wb = template.clone();
                let a1 = Address::new(1, 1).unwrap();
                wb.set("Sheet1", a1, CellInput::Literal(Value::Number(99.0)))
                    .unwrap();
                wb.recalc_incremental(&ctx, &[("Sheet1".to_string(), a1)])
            });
        });
    }
    group.finish();

    // Same edit, same fixture, but `template` here is never recalculated
    // before the loop, so its graph cache starts cold and a clone inherits
    // that empty cache slot (same reasoning as `full_recalc`'s templates,
    // which are never pre-recalc'd either). `recalc_incremental` must build
    // the graph inside the timed closure on every iteration - this is the
    // cost the warm group above cannot see: a host's *first* incremental
    // recalc after loading a workbook and editing a cell.
    let mut group = c.benchmark_group("incremental_recalc_cold/independent_edit_root");
    for n in [100u32, 1000] {
        let template = build_independent(n);
        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, _| {
            b.iter(|| {
                let mut wb = template.clone();
                let a1 = Address::new(1, 1).unwrap();
                wb.set("Sheet1", a1, CellInput::Literal(Value::Number(99.0)))
                    .unwrap();
                wb.recalc_incremental(&ctx, &[("Sheet1".to_string(), a1)])
            });
        });
    }
    group.finish();

    // The many-sheet shape, edited the same way. Incremental recalc used to
    // carry a *steeper* sheet-count slope than a full recalc did — it ran three
    // further per-formula-cell sheet scans (the volatile sweep, the pre-edit
    // snapshot, and spill seeding) on top of evaluation's (issue #952).
    let mut group = c.benchmark_group("incremental_recalc/multi_sheet_edit_root");
    group.sample_size(20);
    let mut template = build_multi_sheet(200, 50, long_sheet_name);
    template.recalc(&ctx);
    let first = template.sheets()[0].name().to_owned();
    group.bench_function("200x50", |b| {
        b.iter(|| {
            let mut wb = template.clone();
            let a1 = Address::new(1, 1).unwrap();
            wb.set(&first, a1, CellInput::Literal(Value::Number(99.0)))
                .unwrap();
            wb.recalc_incremental(&ctx, &[(first.clone(), a1)])
        });
    });
    group.finish();

    // Editing A1 in the block-subtotal fixture dirties every subtotal whose
    // 100-row window covers row 1, exercising range-precedent invalidation.
    let mut group = c.benchmark_group("incremental_recalc/block_subtotals_edit_root");
    group.sample_size(20);
    let n = 10000u32;
    let mut template = build_block_subtotals(n);
    template.recalc(&ctx);
    group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, _| {
        b.iter(|| {
            let mut wb = template.clone();
            let a1 = Address::new(1, 1).unwrap();
            wb.set("Sheet1", a1, CellInput::Literal(Value::Number(99.0)))
                .unwrap();
            wb.recalc_incremental(&ctx, &[("Sheet1".to_string(), a1)])
        });
    });
    group.finish();
}

fn bench_from_json(c: &mut Criterion) {
    let mut group = c.benchmark_group("from_json");
    let json = build_independent(500).to_json().unwrap();
    group.bench_function("500rows", |b| {
        b.iter(|| Workbook::from_json(json.as_bytes()).unwrap());
    });
    group.finish();
}

fn bench_to_json(c: &mut Criterion) {
    let mut group = c.benchmark_group("to_json");
    let wb = build_independent(500);
    group.bench_function("500rows", |b| {
        b.iter(|| wb.to_json().unwrap());
    });
    group.finish();
}

/// Machine-speed probe. Fixed allocate-and-hash workload with **no dependency
/// on any truecalc code**, so any movement is the machine (or the Rust
/// toolchain), not the engine. It is not immune to change from *outside*
/// truecalc: it still depends on `std`'s `HashMap`, `SipHash`, `format!` and
/// the platform allocator, so a rustc/std upgrade can shift every baseline at
/// once — that would show up as every benchmark moving together, not one.
///
/// The regression gate divides every other benchmark's measured time by this
/// one, which cancels the bulk of the difference between a developer laptop and
/// a shared CI runner. Do not "optimise" this function — its value is that it
/// is frozen.
fn calibration_workload() -> f64 {
    let mut map: HashMap<String, f64> = HashMap::new();
    for i in 0..20_000u32 {
        map.insert(format!("key{i}"), i as f64);
    }
    let mut sum = 0.0;
    for i in 0..20_000u32 {
        sum += map[&format!("key{i}")];
    }
    sum
}

fn bench_calibration(c: &mut Criterion) {
    let mut group = c.benchmark_group("calibration");
    group.bench_function("hash_alloc", |b| {
        b.iter(|| black_box(calibration_workload()));
    });
    group.finish();
}

criterion_group!(
    benches,
    bench_calibration,
    bench_full_recalc,
    bench_depgraph_build,
    bench_incremental_recalc,
    bench_from_json,
    bench_to_json
);
criterion_main!(benches);
