//! P1.4 (issue #526) — value-level completeness.
//!
//! Blocking coverage for the two P1.4 concerns:
//!
//! 1. **Date serial semantics per engine flavor** — the Sheets date system
//!    (day 0 = 1899-12-30, no 1900 leap-year bug, +1900 year rule) and the
//!    date-type production rule (JSON schema spec §6: a value is date-typed
//!    iff the fixtures pipeline observed Google Sheets producing a Date).
//! 2. **Unspilled arrays** — array values flow through evaluation whole;
//!    spilling (or collapsing to the top-left element for a single-cell
//!    view) is the workbook/surface layer's job.
//!
//! Every expected value in the date tests below is pipeline-verified: it is
//! taken verbatim from `tests/fixtures/google_sheets/workbook.tsv` (DATE_TYPE
//! block, PR #559) — none are self-confirmed.  The full fixture file also runs
//! through the non-blocking `workbook_conformance_report` harness; these
//! assertions make the P1.4 subset blocking without waiting for the
//! cross-sheet/named-range support (P1.2/P1.3) the rest of the file needs.

use std::collections::HashMap;
use truecalc_core::{Engine, ErrorKind, Value};

/// Pinned "now" for volatile rows: workbook.tsv DATE_TYPE meta
/// `evaluatedAt = 2026-06-07T23:50:56.808Z`, sheet timezone `Etc/GMT`
/// (= UTC), i.e. local day serial 46180 plus the time-of-day fraction.
const PINNED_NOW: f64 = 46180.0 + (23.0 * 3600.0 + 50.0 * 60.0 + 56.808) / 86400.0;

fn eval(formula: &str) -> Value {
    Engine::sheets().evaluate(formula, &HashMap::new())
}

fn eval_at(formula: &str, now_serial: f64) -> Value {
    Engine::sheets().evaluate_at(formula, &HashMap::new(), now_serial)
}

// ── Date-type production (workbook.tsv DATE_TYPE rows) ──────────────────────

#[test]
fn date_returns_a_date_typed_value() {
    // workbook.tsv: =DATE(2026,6,7) → 46180 (date)
    assert_eq!(eval("=DATE(2026,6,7)"), Value::Date(46180.0));
}

#[test]
fn n_of_date_is_a_plain_number() {
    // workbook.tsv: =N(DATE(2026,6,7)) → 46180 (number)
    assert_eq!(eval("=N(DATE(2026,6,7))"), Value::Number(46180.0));
}

#[test]
fn date_plus_one_day_stays_date_typed() {
    // workbook.tsv: =DATE(2026,6,7)+1 → 46181 (date)
    assert_eq!(eval("=DATE(2026,6,7)+1"), Value::Date(46181.0));
}

#[test]
fn date_arithmetic_across_month_boundary_stays_date_typed() {
    // workbook.tsv: =DATE(2026,6,30)+1 → 46204 (date)
    assert_eq!(eval("=DATE(2026,6,30)+1"), Value::Date(46204.0));
}

#[test]
fn date_minus_date_is_a_plain_number() {
    // workbook.tsv: =DATE(2026,6,7)-DATE(2026,6,1) → 6 (number)
    assert_eq!(eval("=DATE(2026,6,7)-DATE(2026,6,1)"), Value::Number(6.0));
}

#[test]
fn isdate_observations() {
    // workbook.tsv: =ISDATE(DATE(2026,6,7)) → TRUE,
    // =ISDATE(DATE(2026,6,7)+1) → TRUE, =ISDATE(N(DATE(2026,6,7))) → FALSE
    assert_eq!(eval("=ISDATE(DATE(2026,6,7))"), Value::Bool(true));
    assert_eq!(eval("=ISDATE(DATE(2026,6,7)+1)"), Value::Bool(true));
    assert_eq!(eval("=ISDATE(N(DATE(2026,6,7)))"), Value::Bool(false));
}

#[test]
fn plain_serial_literal_stays_number_typed() {
    // workbook.tsv: =45000 → 45000 (number)
    assert_eq!(eval("=45000"), Value::Number(45000.0));
}

#[test]
fn date_compares_equal_to_its_own_serial() {
    // workbook.tsv: =DATE(2026,6,7)=N(DATE(2026,6,7)) → TRUE (coercion)
    assert_eq!(eval("=DATE(2026,6,7)=N(DATE(2026,6,7))"), Value::Bool(true));
}

#[test]
fn datevalue_is_number_typed() {
    // workbook.tsv: =DATEVALUE("2026-06-07") → 46180 (number),
    // =ISDATE(DATEVALUE("2026-06-07")) → FALSE
    assert_eq!(eval("=DATEVALUE(\"2026-06-07\")"), Value::Number(46180.0));
    assert_eq!(eval("=ISDATE(DATEVALUE(\"2026-06-07\"))"), Value::Bool(false));
}

#[test]
fn eomonth_is_date_typed() {
    // workbook.tsv: =ISDATE(EOMONTH(DATE(2026,6,7),0)) → TRUE
    assert_eq!(eval("=ISDATE(EOMONTH(DATE(2026,6,7),0))"), Value::Bool(true));
}

// ── Sheets date serial system (epoch, divergence zone, +1900 rule) ──────────

#[test]
fn sheets_epoch_anchor_serials() {
    // workbook.tsv: =TEXT(0,"yyyy-mm-dd") → 1899-12-30,
    //               =TEXT(1,"yyyy-mm-dd") → 1899-12-31
    assert_eq!(eval("=TEXT(0,\"yyyy-mm-dd\")"), Value::Text("1899-12-30".into()));
    assert_eq!(eval("=TEXT(1,\"yyyy-mm-dd\")"), Value::Text("1899-12-31".into()));
}

#[test]
fn sheets_has_no_1900_leap_year_bug() {
    // workbook.tsv: =N(DATE(1900,1,1)) → 2, =N(DATE(1900,2,28)) → 60,
    // =N(DATE(1900,3,1)) → 61 — 60 and 61 are adjacent days; there is no
    // serial for the nonexistent 1900-02-29 in the Sheets system.
    assert_eq!(eval("=N(DATE(1900,1,1))"), Value::Number(2.0));
    assert_eq!(eval("=N(DATE(1900,2,28))"), Value::Number(60.0));
    assert_eq!(eval("=N(DATE(1900,3,1))"), Value::Number(61.0));
}

#[test]
fn nonexistent_1900_02_29_rolls_over_to_march_first() {
    // workbook.tsv: =N(DATE(1900,2,29)) → 61,
    //               =TEXT(DATE(1900,2,29),"yyyy-mm-dd") → 1900-03-01
    assert_eq!(eval("=N(DATE(1900,2,29))"), Value::Number(61.0));
    assert_eq!(
        eval("=TEXT(DATE(1900,2,29),\"yyyy-mm-dd\")"),
        Value::Text("1900-03-01".into())
    );
}

#[test]
fn plus_1900_year_rule_for_year_arguments_below_1900() {
    // workbook.tsv: =N(DATE(1899,12,31)) → 693962 (3799-12-31),
    // =N(DATE(1899,12,30)) → 693961, =N(DATE(1899,12,25)) → 693956,
    // =TEXT(DATE(1899,12,25),"yyyy-mm-dd") → 3799-12-25
    assert_eq!(eval("=N(DATE(1899,12,31))"), Value::Number(693962.0));
    assert_eq!(eval("=N(DATE(1899,12,30))"), Value::Number(693961.0));
    assert_eq!(eval("=N(DATE(1899,12,25))"), Value::Number(693956.0));
    assert_eq!(
        eval("=TEXT(DATE(1899,12,25),\"yyyy-mm-dd\")"),
        Value::Text("3799-12-25".into())
    );
}

// ── Volatile pinning via Engine::evaluate_at (RecalcContext hook) ────────────

#[test]
fn pinned_today_matches_fixture_row() {
    // workbook.tsv (volatile, pinned by meta.evaluatedAt): =TODAY() → 46180 (date)
    assert_eq!(eval_at("=TODAY()", PINNED_NOW), Value::Date(46180.0));
}

#[test]
fn pinned_today_is_date_typed() {
    // workbook.tsv: =ISDATE(TODAY()) → TRUE
    assert_eq!(eval_at("=ISDATE(TODAY())", PINNED_NOW), Value::Bool(true));
}

#[test]
fn today_minus_today_within_one_recalc_is_zero() {
    // workbook.tsv: =TODAY()-TODAY() → 0 (number)
    assert_eq!(eval_at("=TODAY()-TODAY()", PINNED_NOW), Value::Number(0.0));
}

#[test]
fn pinned_now_returns_the_pinned_serial() {
    assert_eq!(eval_at("=NOW()", PINNED_NOW), Value::Number(PINNED_NOW));
}

#[test]
fn evaluate_at_is_deterministic() {
    assert_eq!(
        eval_at("=TODAY()&\"|\"&NOW()", PINNED_NOW),
        eval_at("=TODAY()&\"|\"&NOW()", PINNED_NOW)
    );
}

#[test]
fn evaluate_at_rejects_non_finite_now() {
    assert_eq!(
        eval_at("=TODAY()", f64::NAN),
        Value::Error(ErrorKind::Num)
    );
    assert_eq!(
        eval_at("=TODAY()", f64::INFINITY),
        Value::Error(ErrorKind::Num)
    );
}

// ── Unspilled arrays ─────────────────────────────────────────────────────────

#[test]
fn array_literal_flows_through_unspilled() {
    // The engine returns the whole array; collapsing to the top-left element
    // (the old behavior) or spilling is the workbook/surface layer's job.
    assert_eq!(
        eval("={1,2,3}"),
        Value::Array(vec![
            Value::Number(1.0),
            Value::Number(2.0),
            Value::Number(3.0),
        ])
    );
}

#[test]
fn array_through_function_result_is_unspilled() {
    // An array produced inside a larger expression also survives whole.
    assert_eq!(
        eval("=IF(TRUE,{1,2},0)"),
        Value::Array(vec![Value::Number(1.0), Value::Number(2.0)])
    );
}

// ── Excel flavor stays stubbed behind the flavor enum (allowed by #526) ─────

#[test]
fn excel_evaluation_is_still_stubbed() {
    // The Excel 1900 date system (incl. leap-bug serial 60) is locked behind
    // the flavor enum in `eval::functions::date::serial`; Excel *evaluation*
    // remains a stub returning an error (exact kind tracked by #556).
    assert!(matches!(
        Engine::excel().evaluate("=DATE(2026,6,7)", &HashMap::new()),
        Value::Error(_)
    ));
}
