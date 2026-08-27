//! Parsing must not build a function registry (issue #900).
//!
//! Three workbook call sites built a full 518-function `Registry` — via
//! `Engine::sheets()` / `Engine::excel()` — only to reach the parser, which
//! never consults it: `Workbook::validate_formula` (once per formula cell
//! written), `GridResolver::resolve_name_ref` (once per named-range reference
//! resolved during recalc) and `DependencyGraph::build` (once per graph build).
//! All three now call the registry-free parser entry point directly.
//!
//! The budgets below are expressed in units of `Engine::sheets()` measured in
//! the same run — the pattern established for the recalc hoist — so they are
//! machine- and profile-independent rather than wall-clock assertions.

use std::hint::black_box;
use std::time::{Duration, Instant};

use truecalc_core::Engine;
use truecalc_workbook::{
    Address, CellInput, EngineFlavor, RecalcContext, Value, Workbook, Worksheet,
};

/// Cost of one `Engine::sheets()` (i.e. one `Registry::new()`), averaged.
fn registry_build_cost() -> Duration {
    const N: u32 = 40;
    // Warm up: first construction pays page faults / lazy statics.
    black_box(Engine::sheets());
    let start = Instant::now();
    for _ in 0..N {
        black_box(Engine::sheets());
    }
    start.elapsed() / N
}

/// `Sheet1` with `A1:A5` = 1..5 and no formulas.
fn literal_grid() -> Workbook {
    let mut wb = Workbook::new(EngineFlavor::Sheets);
    wb.add_sheet(Worksheet::new("Sheet1")).unwrap();
    for row in 1..=5u32 {
        wb.set(
            "Sheet1",
            Address::new(row, 1).unwrap(),
            CellInput::Literal(Value::Number(f64::from(row))),
        )
        .unwrap();
    }
    wb
}

/// Writing formula cells must not cost a registry construction per write.
///
/// `Workbook::set` validates a formula before storing it, and validation is a
/// parse — the registry it used to build was never read. Before the fix a write
/// of `CELLS` formula cells cost `CELLS` registry builds; it now costs none.
/// The budget of `CELLS / 3` sits well below the pre-fix cost and far above the
/// post-fix one, which is what keeps it from being flaky.
#[test]
fn writing_formula_cells_does_not_build_a_registry_per_cell() {
    const CELLS: u32 = 200;

    let unit = registry_build_cost();

    // Warm up the write path once, then measure a fresh workbook.
    let mut warm = literal_grid();
    warm.set(
        "Sheet1",
        Address::new(1, 2).unwrap(),
        CellInput::Formula("=A1+1".to_string()),
    )
    .unwrap();

    let mut wb = literal_grid();
    let start = Instant::now();
    for row in 1..=CELLS {
        black_box(
            wb.set(
                "Sheet1",
                Address::new(row, 2).unwrap(),
                CellInput::Formula(format!("=A1+{row}")),
            )
            .unwrap(),
        );
    }
    let elapsed = start.elapsed();

    let units = elapsed.as_secs_f64() / unit.as_secs_f64();
    let budget = f64::from(CELLS) / 3.0;
    assert!(
        units <= budget,
        "writing {CELLS} formula cells cost {units:.1} registry builds \
         ({elapsed:?} at {unit:?} per build); budget is {budget:.1}. \
         A registry is being built to validate each formula (issue #900)."
    );
}

/// Resolving a named-range reference must not cost a registry construction per
/// reference. `GridResolver::resolve_name_ref` parses the name's stored `ref`
/// string; the registry it used to build was never read.
#[test]
fn resolving_named_ranges_does_not_build_a_registry_per_reference() {
    const CELLS: u32 = 200;

    let unit = registry_build_cost();
    let ctx = RecalcContext::new(0, "UTC", 0).unwrap();

    let mut template = literal_grid();
    template.define_name("MYNAME", "Sheet1!A1").unwrap();
    for row in 1..=CELLS {
        template
            .set(
                "Sheet1",
                Address::new(row, 2).unwrap(),
                CellInput::Formula("=MYNAME+1".to_string()),
            )
            .unwrap();
    }

    // Warm up the recalc path once, then measure.
    let mut warm = template.clone();
    warm.recalc(&ctx);

    let mut wb = template.clone();
    let start = Instant::now();
    black_box(wb.recalc(&ctx));
    let elapsed = start.elapsed();

    // The values must still be right: every cell reads A1 = 1 through the name.
    assert_eq!(
        wb.get("Sheet1", Address::new(1, 2).unwrap())
            .unwrap()
            .value(),
        &Value::Number(2.0),
        "named-range resolution changed value"
    );

    let units = elapsed.as_secs_f64() / unit.as_secs_f64();
    let budget = f64::from(CELLS) / 3.0;
    assert!(
        units <= budget,
        "recalc over {CELLS} named-range references cost {units:.1} registry \
         builds ({elapsed:?} at {unit:?} per build); budget is {budget:.1}. \
         A registry is being built per named-range reference (issue #900)."
    );
}

/// Not building a registry must not weaken validation: `set` still rejects a
/// syntactically invalid formula, and still accepts a valid one, under both
/// engine flavors (the parser is flavor-independent — `Engine::parse` ignores
/// the flavor — so validation behaves identically for Sheets and Excel).
///
/// This asserts a **current** fact about the grammar, not an invariant: it
/// holds because Sheets and Excel share one grammar today. If Excel ever
/// gains a divergent grammar, the fix is to re-thread flavor through the
/// parse path (parsing, `extract_refs`, and `validate_formula` all currently
/// assume a single flavor-agnostic grammar — see the module docs) — not to
/// relax this assertion.
#[test]
fn set_still_validates_formula_syntax() {
    for flavor in [EngineFlavor::Sheets, EngineFlavor::Excel] {
        let mut wb = Workbook::new(flavor);
        wb.add_sheet(Worksheet::new("Sheet1")).unwrap();

        let bad = wb.set(
            "Sheet1",
            Address::new(1, 1).unwrap(),
            CellInput::Formula("=SUM(".to_string()),
        );
        assert!(bad.is_err(), "{flavor:?}: invalid formula was accepted");
        assert!(
            format!("{}", bad.unwrap_err()).contains("is invalid"),
            "{flavor:?}: unexpected error message for an invalid formula"
        );

        wb.set(
            "Sheet1",
            Address::new(1, 1).unwrap(),
            CellInput::Formula("=SUM(A1:A5)".to_string()),
        )
        .expect("valid formula must still be accepted");
    }
}
