use crate::Engine;
use crate::eval::functions::lookup::{
    index_match::match_fn,
    lookup_fn::{lookup_fn, xmatch_fn},
    vlookup::vlookup_fn,
};
use crate::types::{ErrorKind, Value};
use std::collections::HashMap;

fn run(formula: &str) -> Value {
    Engine::sheets().evaluate(formula, &HashMap::new())
}

fn make_1d(vals: Vec<Value>) -> Value {
    Value::Array(vals)
}

fn n(v: f64) -> Value {
    Value::Number(v)
}

fn t(s: &str) -> Value {
    Value::Text(s.to_string())
}

fn make_2d(rows: Vec<Vec<Value>>) -> Value {
    Value::Array(rows.into_iter().map(Value::Array).collect())
}

// ---------------------------------------------------------------------------
// VLOOKUP approximate match edge cases
// ---------------------------------------------------------------------------

#[test]
fn vlookup_approx_below_minimum_returns_na() {
    // Nothing in the first column is <= 0.5, so result is #N/A
    let range = make_2d(vec![
        vec![n(1.0), t("a")],
        vec![n(3.0), t("b")],
        vec![n(5.0), t("c")],
    ]);
    assert_eq!(vlookup_fn(&[n(0.5), range, n(2.0), Value::Bool(true)]), Value::Error(ErrorKind::NA));
}

#[test]
fn vlookup_approx_above_maximum_returns_last_row() {
    // 100 > all keys; largest <= 100 is 5 → "c"
    let range = make_2d(vec![
        vec![n(1.0), t("a")],
        vec![n(3.0), t("b")],
        vec![n(5.0), t("c")],
    ]);
    assert_eq!(vlookup_fn(&[n(100.0), range, n(2.0), Value::Bool(true)]), t("c"));
}

#[test]
fn vlookup_approx_exact_key_match() {
    // Key 3 exists exactly → returns "b", not "a" or "c"
    let range = make_2d(vec![
        vec![n(1.0), t("a")],
        vec![n(3.0), t("b")],
        vec![n(5.0), t("c")],
    ]);
    assert_eq!(vlookup_fn(&[n(3.0), range, n(2.0), Value::Bool(true)]), t("b"));
}

#[test]
fn vlookup_default_is_sorted_true() {
    // When 4th arg is omitted, is_sorted defaults to true (approximate match)
    let range = make_2d(vec![
        vec![n(1.0), t("a")],
        vec![n(3.0), t("b")],
        vec![n(5.0), t("c")],
    ]);
    assert_eq!(vlookup_fn(&[n(2.0), range, n(2.0)]), t("a"));
}

#[test]
fn vlookup_approx_col_index_1_returns_key() {
    // col_index=1 returns the matched key value itself
    let range = make_2d(vec![
        vec![n(10.0), n(100.0)],
        vec![n(20.0), n(200.0)],
    ]);
    assert_eq!(vlookup_fn(&[n(15.0), range, n(1.0), Value::Bool(true)]), n(10.0));
}

// MATCH boundary cases
#[test]
fn match_approx_between_values() {
    // 2.5 → largest <= 2.5 is 2, at position 2
    let arr = make_1d(vec![n(1.0), n(2.0), n(3.0)]);
    assert_eq!(match_fn(&[n(2.5), arr, n(1.0)]), n(2.0));
}

#[test]
fn match_first_element() {
    let arr = make_1d(vec![n(1.0), n(2.0), n(3.0)]);
    assert_eq!(match_fn(&[n(1.0), arr, n(0.0)]), n(1.0));
}

#[test]
fn match_last_element() {
    let arr = make_1d(vec![n(1.0), n(2.0), n(3.0)]);
    assert_eq!(match_fn(&[n(3.0), arr, n(0.0)]), n(3.0));
}

#[test]
fn lookup_with_single_element_range_found() {
    let search = make_1d(vec![n(42.0)]);
    assert_eq!(lookup_fn(&[n(42.0), search]), n(42.0));
}

#[test]
fn xmatch_finds_first_occurrence() {
    let lookup = make_1d(vec![n(1.0), n(2.0), n(2.0), n(3.0)]);
    assert_eq!(xmatch_fn(&[n(2.0), lookup]), n(2.0));
}

#[test]
fn row_with_range_ref_returns_start_row() {
    assert_eq!(run("=ROW(B3:D5)"), n(3.0));
}

#[test]
fn column_with_range_ref_returns_start_col() {
    assert_eq!(run("=COLUMN(C1:E5)"), n(3.0));
}

#[test]
fn lookup_result_range_shorter_than_search_range_returns_na() {
    let keys = make_1d(vec![n(1.0), n(2.0), n(3.0)]);
    let vals = make_1d(vec![t("a")]);
    assert_eq!(lookup_fn(&[n(3.0), keys, vals]), Value::Error(ErrorKind::NA));
}
