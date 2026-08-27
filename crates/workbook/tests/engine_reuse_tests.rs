//! Engine reuse across a recalc pass (issue #886).
//!
//! `Recalc::eval_formula_cell` used to construct a brand-new [`Engine`] — and
//! therefore a brand-new 518-function `Registry` — once per formula cell, per
//! pass. The engine is now built once per `recompute` call and shared by every
//! cell in the pass. These tests pin both halves of that change:
//!
//! 1. **Reuse is safe.** A shared engine must produce exactly the values a
//!    fresh-engine-per-formula would, in any evaluation order, and must not
//!    drift as evaluations accumulate on it. (`Engine` holds only a `Copy`
//!    flavor and a `Registry` of `fn`-pointer entries, every evaluation entry
//!    point takes `&self`, and `crates/core/src` contains no interior
//!    mutability — so this is a type-level guarantee. The test guards it
//!    against future drift.)
//!
//! 2. **Reuse actually happens.** A full recalc must not cost a registry
//!    construction per cell. The budget is expressed in units of
//!    `Engine::sheets()` so it is machine- and profile-independent: before the
//!    fix a recalc of N formula cells cost ~2N registry builds (two passes of
//!    the spill-convergence fixpoint); it now costs a small constant.

use std::hint::black_box;
use std::time::{Duration, Instant};

use truecalc_core::{Engine, Ref, Resolver, Value as CoreValue};
use truecalc_workbook::{
    Address, CellInput, EngineFlavor, RecalcContext, Value, Workbook, Worksheet,
};

// ── 1. Reuse is safe ──────────────────────────────────────────────────────

/// A fixed grid so reference-bearing formulas have something to read.
/// `A1..A5` = 1..5, `B1..B5` = 10..50, everything else empty.
struct FixedGrid;

impl Resolver for FixedGrid {
    fn resolve(&mut self, r: &Ref) -> CoreValue {
        match r {
            Ref::Cell { addr, .. } => cell_value(addr.row, addr.col),
            Ref::Range { start, end, .. } => {
                let rows = (start.row..=end.row)
                    .map(|row| {
                        CoreValue::Array(
                            (start.col..=end.col)
                                .map(|col| cell_value(row, col))
                                .collect(),
                        )
                    })
                    .collect();
                CoreValue::Array(rows)
            }
            _ => CoreValue::Empty,
        }
    }
}

fn cell_value(row: u32, col: u32) -> CoreValue {
    match (row, col) {
        (1..=5, 1) => CoreValue::Number(row as f64),
        (1..=5, 2) => CoreValue::Number(row as f64 * 10.0),
        _ => CoreValue::Empty,
    }
}

/// A deliberately state-hungry mix: aggregation over ranges, lazy/short-circuit
/// functions, LAMBDA and the higher-order array functions (which bind and
/// rebind parameters), volatile date functions, the keyed RNG, text and lookup.
/// If the engine carried any per-cell mutable state, these are the formulas
/// that would leak it.
const FORMULAS: &[&str] = &[
    "=A1+B1",
    "=SUM(A1:A5)",
    "=SUMPRODUCT(A1:A5, B1:B5)",
    "=IF(A1>0, \"pos\", \"neg\")",
    "=AND(A1=1, B1=10)",
    "=MAP(A1:A5, LAMBDA(x, x*2))",
    "=REDUCE(0, A1:A5, LAMBDA(acc, x, acc+x))",
    "=SCAN(0, A1:A5, LAMBDA(acc, x, acc+x))",
    "=BYROW(A1:B5, LAMBDA(r, SUM(r)))",
    "=MAKEARRAY(2, 3, LAMBDA(r, c, r*10+c))",
    "=LAMBDA(x, x+1)(41)",
    "=TEXTJOIN(\"-\", TRUE, A1:A5)",
    "=VLOOKUP(3, A1:B5, 2, FALSE)",
    "=SORT(B1:B5, 1, FALSE)",
    "=FILTER(A1:A5, A1:A5>2)",
    "=TODAY()",
    "=NOW()",
    "=RAND()",
    "=RANDBETWEEN(1, 1000000)",
    "=CONCATENATE(\"a\", TEXT(A1, \"0.00\"))",
    "=SEQUENCE(3, 2)",
    "=XLOOKUP(2, A1:A5, B1:B5)",
    "=IFERROR(1/0, \"boom\")",
    "=SUMIF(A1:A5, \">2\", B1:B5)",
];

const NOW_SERIAL: f64 = 46_000.25;
const NOW_UTC_NANOS: i64 = 1_780_878_600_000_000_000;

fn eval_one(engine: &Engine, formula: &str, index: usize) -> CoreValue {
    let mut resolver = FixedGrid;
    // Vary the RNG key by formula index exactly the way recalc keys it by cell
    // position, so the volatile functions are pinned but still position-distinct.
    let rng_cell = Some((7_u64, 0_u32, index as u32 + 1, 1_u32));
    engine.evaluate_with_resolver_at_keyed(
        formula,
        &mut resolver,
        Some(NOW_SERIAL),
        Some(NOW_UTC_NANOS),
        rng_cell,
    )
}

/// One engine reused across every formula must match a fresh engine per
/// formula, value for value.
#[test]
fn reused_engine_matches_a_fresh_engine_per_formula() {
    let shared = Engine::sheets();
    for (i, formula) in FORMULAS.iter().enumerate() {
        let fresh = Engine::sheets();
        let expected = eval_one(&fresh, formula, i);
        let actual = eval_one(&shared, formula, i);
        assert_eq!(
            actual, expected,
            "reused engine diverged from a fresh engine on {formula}"
        );
    }
}

/// Evaluation order through a shared engine must not matter: running the whole
/// list forwards, then backwards, then forwards again on the *same* engine must
/// give the same value for each formula every time. This is what would catch
/// state accumulating on the engine across cells.
#[test]
fn shared_engine_is_order_and_repetition_independent() {
    let shared = Engine::sheets();

    let forward: Vec<CoreValue> = FORMULAS
        .iter()
        .enumerate()
        .map(|(i, f)| eval_one(&shared, f, i))
        .collect();

    let mut backward: Vec<(usize, CoreValue)> = FORMULAS
        .iter()
        .enumerate()
        .rev()
        .map(|(i, f)| (i, eval_one(&shared, f, i)))
        .collect();
    backward.sort_by_key(|(i, _)| *i);

    for (i, value) in &backward {
        assert_eq!(
            value, &forward[*i],
            "reverse-order evaluation on the shared engine changed {}",
            FORMULAS[*i]
        );
    }

    for (i, formula) in FORMULAS.iter().enumerate() {
        assert_eq!(
            eval_one(&shared, formula, i),
            forward[i],
            "third pass on the shared engine changed {formula}"
        );
    }
}

/// The workbook-level counterpart, stated without reaching into core: the
/// values a sheet full of formula cells gets from **one** shared engine must
/// equal the values each formula gets when it is the only formula in its own
/// workbook — i.e. when it has an engine to itself. Same sheet name, same
/// literal grid, same cell address, same `RecalcContext`, so the per-cell RNG
/// key and the volatile clock are identical on both sides; only the number of
/// formulas sharing the engine differs.
#[test]
fn shared_engine_recalc_matches_one_engine_per_formula() {
    // Formulas read only literals, never each other, so each can stand alone.
    let cases = [
        "=A1+B1",
        "=SUM(A1:A5)",
        "=SUMPRODUCT(A1:A5, B1:B5)",
        "=IF(A1>0, \"pos\", \"neg\")",
        "=MAP(A1:A5, LAMBDA(x, x*2))",
        "=REDUCE(0, A1:A5, LAMBDA(acc, x, acc+x))",
        "=BYROW(A1:B5, LAMBDA(r, SUM(r)))",
        "=TEXTJOIN(\"-\", TRUE, A1:A5)",
        "=VLOOKUP(3, A1:B5, 2, FALSE)",
        "=SORT(B1:B5, 1, FALSE)",
        "=SUMIF(A1:A5, \">2\", B1:B5)",
        "=IFERROR(1/0, \"boom\")",
        "=TODAY()",
        "=NOW()",
        "=RAND()",
        "=RANDBETWEEN(1, 1000000)",
    ];
    // Column D, spaced out so array results have room to spill without
    // colliding with the next formula.
    let addr = |i: usize| Address::new(i as u32 * 8 + 1, 4).unwrap();

    let ctx = RecalcContext::new(1_780_878_600_000, "UTC", 7).unwrap();

    // One workbook holding every formula: all of them share a single engine.
    let mut shared = literal_grid();
    for (i, formula) in cases.iter().enumerate() {
        shared
            .set(
                "Sheet1",
                addr(i),
                CellInput::Formula((*formula).to_string()),
            )
            .unwrap();
    }
    shared.recalc(&ctx);

    for (i, formula) in cases.iter().enumerate() {
        // The same formula, alone in its own workbook at the same address:
        // one engine, one formula.
        let mut solo = literal_grid();
        solo.set(
            "Sheet1",
            addr(i),
            CellInput::Formula((*formula).to_string()),
        )
        .unwrap();
        solo.recalc(&ctx);

        assert_eq!(
            shared.get("Sheet1", addr(i)).unwrap().value(),
            solo.get("Sheet1", addr(i)).unwrap().value(),
            "sharing the engine across cells changed the value of {formula}"
        );
    }
}

/// `Sheet1` with `A1:A5` = 1..5 and `B1:B5` = 10..50 and no formulas.
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
        wb.set(
            "Sheet1",
            Address::new(row, 2).unwrap(),
            CellInput::Literal(Value::Number(f64::from(row) * 10.0)),
        )
        .unwrap();
    }
    wb
}

// ── 2. Reuse actually happens ─────────────────────────────────────────────

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

fn build_chain(n: u32) -> Workbook {
    let mut wb = Workbook::new(EngineFlavor::Sheets);
    wb.add_sheet(Worksheet::new("Sheet1")).unwrap();
    for row in 1..=n {
        wb.set(
            "Sheet1",
            Address::new(row, 1).unwrap(),
            CellInput::Literal(Value::Number(row as f64)),
        )
        .unwrap();
        wb.set(
            "Sheet1",
            Address::new(row, 2).unwrap(),
            CellInput::Formula(format!("=A{row}+1")),
        )
        .unwrap();
    }
    wb
}

/// A full recalc must not build a function registry per formula cell.
///
/// The budget is stated in registry-construction units rather than wall-clock
/// microseconds, so it holds on any machine and in either build profile. Before
/// the engine was hoisted out of `eval_formula_cell`, a recalc of `CELLS`
/// formula cells cost ~2 × `CELLS` registry builds (the spill-convergence
/// fixpoint always runs at least two passes over a freshly imported workbook).
///
/// Measured at `CELLS = 200`: 404.4 builds before the hoist, 11.8 after. The
/// budget below — a third of `CELLS`, i.e. 66.7 builds — sits between them with
/// ~6× of margin on each side, which is what keeps it from being a flaky
/// wall-clock assertion.
#[test]
fn full_recalc_does_not_build_a_registry_per_cell() {
    const CELLS: u32 = 200;

    let unit = registry_build_cost();
    let ctx = RecalcContext::new(0, "UTC", 0).unwrap();
    let template = build_chain(CELLS);

    // Warm up the recalc path once, then measure.
    let mut warm = template.clone();
    warm.recalc(&ctx);

    let mut wb = template.clone();
    let start = Instant::now();
    black_box(wb.recalc(&ctx));
    let elapsed = start.elapsed();

    let units = elapsed.as_secs_f64() / unit.as_secs_f64();
    let budget = f64::from(CELLS) / 3.0;
    assert!(
        units <= budget,
        "recalc of {CELLS} formula cells cost {units:.1} registry builds \
         ({elapsed:?} at {unit:?} per build); budget is {budget:.1}. \
         An engine is being constructed per cell (issue #886)."
    );
}
