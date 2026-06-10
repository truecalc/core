//! PRF-keyed per-cell RNG determinism tests (issue #588).
//!
//! Verifies that RAND/RANDBETWEEN/RANDARRAY produce byte-identical values
//! when evaluated under the same RecalcContext, and differ when the seed or
//! cell position changes.

use truecalc_workbook::{Address, CellInput, EngineFlavor, RecalcContext, Value, Workbook, Worksheet};

fn a1(s: &str) -> Address {
    Address::from_a1(s).unwrap()
}

fn ctx(seed: u64) -> RecalcContext {
    RecalcContext::new(1_780_878_600_000, "Etc/GMT", seed).unwrap()
}

fn wb_with_formula(formula: &str) -> Workbook {
    let mut wb = Workbook::new(EngineFlavor::Sheets);
    wb.add_sheet(Worksheet::new("Sheet1")).unwrap();
    wb.set("Sheet1", a1("A1"), CellInput::Formula(formula.into())).unwrap();
    wb
}

/// Same context, two workbooks with RAND() in A1 → identical values.
#[test]
fn rand_same_context_is_deterministic() {
    let c = ctx(42);
    let mut w1 = wb_with_formula("=RAND()");
    let mut w2 = wb_with_formula("=RAND()");
    w1.recalc(&c);
    w2.recalc(&c);
    assert_eq!(
        w1.get("Sheet1", a1("A1")).unwrap().value(),
        w2.get("Sheet1", a1("A1")).unwrap().value(),
        "same context => same RAND() value"
    );
}

/// Different rng_seed → different RAND() value.
#[test]
fn rand_different_seed_differs() {
    let mut w1 = wb_with_formula("=RAND()");
    let mut w2 = wb_with_formula("=RAND()");
    w1.recalc(&ctx(1));
    w2.recalc(&ctx(2));
    assert_ne!(
        w1.get("Sheet1", a1("A1")).unwrap().value(),
        w2.get("Sheet1", a1("A1")).unwrap().value(),
        "different seed => different RAND() value"
    );
}

/// RAND() in different cells (A1 vs B1) with the same context produces
/// different values (different col key).
#[test]
fn rand_different_cells_differ() {
    let mut wb = Workbook::new(EngineFlavor::Sheets);
    wb.add_sheet(Worksheet::new("Sheet1")).unwrap();
    wb.set("Sheet1", a1("A1"), CellInput::Formula("=RAND()".into())).unwrap();
    wb.set("Sheet1", a1("B1"), CellInput::Formula("=RAND()".into())).unwrap();
    wb.recalc(&ctx(99));
    let va = wb.get("Sheet1", a1("A1")).unwrap().value().clone();
    let vb = wb.get("Sheet1", a1("B1")).unwrap().value().clone();
    assert_ne!(va, vb, "RAND() in A1 vs B1 must differ (different col key)");
}

/// Two-sheet workbook: RAND() on Sheet1/A1 and Sheet2/A1 with the same context
/// produces different values (different sheet_index key).
#[test]
fn rand_different_sheets_differ() {
    let mut wb = Workbook::new(EngineFlavor::Sheets);
    wb.add_sheet(Worksheet::new("Sheet1")).unwrap();
    wb.add_sheet(Worksheet::new("Sheet2")).unwrap();
    wb.set("Sheet1", a1("A1"), CellInput::Formula("=RAND()".into())).unwrap();
    wb.set("Sheet2", a1("A1"), CellInput::Formula("=RAND()".into())).unwrap();
    wb.recalc(&ctx(7));
    let v1 = wb.get("Sheet1", a1("A1")).unwrap().value().clone();
    let v2 = wb.get("Sheet2", a1("A1")).unwrap().value().clone();
    assert_ne!(v1, v2, "RAND() on Sheet1 vs Sheet2 must differ (different sheet_index key)");
}

/// RANDARRAY(2,2) in A1 produces an identical array on two recalcs with the
/// same context.
#[test]
fn randarray_same_context_is_deterministic() {
    let c = ctx(55);
    let mut w1 = wb_with_formula("=RANDARRAY(2,2)");
    let mut w2 = wb_with_formula("=RANDARRAY(2,2)");
    w1.recalc(&c);
    w2.recalc(&c);
    assert_eq!(
        w1.get("Sheet1", a1("A1")).unwrap().value(),
        w2.get("Sheet1", a1("A1")).unwrap().value(),
        "same context => same RANDARRAY values"
    );
}

/// RANDBETWEEN(1,1000) same context → same result.
#[test]
fn randbetween_same_context_is_deterministic() {
    let c = ctx(13);
    let mut w1 = wb_with_formula("=RANDBETWEEN(1,1000)");
    let mut w2 = wb_with_formula("=RANDBETWEEN(1,1000)");
    w1.recalc(&c);
    w2.recalc(&c);
    assert_eq!(
        w1.get("Sheet1", a1("A1")).unwrap().value(),
        w2.get("Sheet1", a1("A1")).unwrap().value(),
        "same context => same RANDBETWEEN value"
    );
}
