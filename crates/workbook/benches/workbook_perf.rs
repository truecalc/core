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

    group.finish();
}

fn bench_incremental_recalc(c: &mut Criterion) {
    let ctx = make_ctx();

    // Editing A1 in the independent fixture dirties exactly one formula (B1).
    // This guards the *minimality* of the dirty set, not chain propagation.
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
/// on any truecalc code**, so it never legitimately changes: any movement is
/// the machine, not the engine.
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
