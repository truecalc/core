use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use truecalc_workbook::{Address, CellInput, EngineFlavor, RecalcContext, Value, Workbook, Worksheet};

fn make_ctx() -> RecalcContext {
    RecalcContext::new(0, "UTC", 0).expect("UTC is valid")
}

/// Build a chain workbook with N rows.
/// Column A: literal row number, column B: formula =A{row}+1.
fn build_chain(n: u32) -> Workbook {
    let mut wb = Workbook::new(EngineFlavor::Sheets);
    wb.add_sheet(Worksheet::new("Sheet1")).unwrap();
    for row in 1..=n {
        let a_addr = Address::new(row, 1).unwrap();
        let b_addr = Address::new(row, 2).unwrap();
        wb.set("Sheet1", a_addr, CellInput::Literal(Value::Number(row as f64)))
            .unwrap();
        wb.set(
            "Sheet1",
            b_addr,
            CellInput::Formula(format!("=A{row}+1")),
        )
        .unwrap();
    }
    wb
}

fn bench_full_recalc(c: &mut Criterion) {
    let mut group = c.benchmark_group("full_recalc/chain");
    for n in [100u32, 1000, 5000] {
        let template = build_chain(n);
        let ctx = make_ctx();
        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, _| {
            b.iter(|| {
                let mut wb = template.clone();
                wb.recalc(&ctx)
            });
        });
    }
    group.finish();
}

fn bench_incremental_recalc(c: &mut Criterion) {
    let mut group = c.benchmark_group("incremental_recalc/edit_root");
    for n in [100u32, 1000] {
        let mut template = build_chain(n);
        let ctx = make_ctx();
        // Pre-recalc so incremental starts from a fully-calculated state.
        template.recalc(&ctx);
        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, _| {
            b.iter(|| {
                let mut wb = template.clone();
                // Edit A1 (the root of the chain).
                let a1 = Address::new(1, 1).unwrap();
                wb.set("Sheet1", a1, CellInput::Literal(Value::Number(99.0)))
                    .unwrap();
                wb.recalc_incremental(&ctx, &[("Sheet1".to_string(), a1)])
            });
        });
    }
    group.finish();
}

fn build_500row_json() -> String {
    let wb = build_chain(500);
    wb.to_json().unwrap()
}

fn bench_from_json(c: &mut Criterion) {
    let mut group = c.benchmark_group("from_json");
    let json = build_500row_json();
    group.bench_function("500rows", |b| {
        b.iter(|| Workbook::from_json(json.as_bytes()).unwrap());
    });
    group.finish();
}

fn bench_to_json(c: &mut Criterion) {
    let mut group = c.benchmark_group("to_json");
    let wb = build_chain(500);
    group.bench_function("500rows", |b| {
        b.iter(|| wb.to_json().unwrap());
    });
    group.finish();
}

criterion_group!(
    benches,
    bench_full_recalc,
    bench_incremental_recalc,
    bench_from_json,
    bench_to_json
);
criterion_main!(benches);
