use crate::evaluate;
use crate::eval::functions::lookup::{
    index_match::{index_fn, match_fn},
    lookup_fn::{lookup_fn, xlookup_fn, xmatch_fn},
    vlookup::{hlookup_fn, vlookup_fn},
};
use crate::types::{ErrorKind, Value};
use std::collections::HashMap;

fn make_2d(rows: Vec<Vec<Value>>) -> Value {
    Value::Array(rows.into_iter().map(Value::Array).collect())
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

fn run(formula: &str) -> Value {
    evaluate(formula, &HashMap::new())
}

// CHOOSE
#[test]
fn choose_index_zero_is_error() {
    assert_eq!(run(r#"CHOOSE(0, "a", "b")"#), Value::Error(ErrorKind::Num));
}

#[test]
fn choose_index_too_large_is_error() {
    assert_eq!(run(r#"CHOOSE(5, "a", "b")"#), Value::Error(ErrorKind::Num));
}

// VLOOKUP
#[test]
fn vlookup_exact_match_not_found() {
    let range = make_2d(vec![vec![n(1.0), t("a")], vec![n(2.0), t("b")]]);
    assert_eq!(vlookup_fn(&[n(99.0), range, n(2.0), Value::Bool(false)]), Value::Error(ErrorKind::NA));
}

#[test]
fn vlookup_wrong_arg_count() {
    assert_eq!(vlookup_fn(&[]), Value::Error(ErrorKind::NA));
    assert_eq!(vlookup_fn(&[n(1.0), n(2.0)]), Value::Error(ErrorKind::NA));
}

#[test]
fn vlookup_col_index_out_of_range() {
    let range = make_2d(vec![vec![n(1.0), t("a")]]);
    assert_eq!(vlookup_fn(&[n(1.0), range, n(5.0), Value::Bool(false)]), Value::Error(ErrorKind::Ref));
}

// HLOOKUP
#[test]
fn hlookup_exact_match_not_found() {
    let range = make_2d(vec![
        vec![n(1.0), n(2.0), n(3.0)],
        vec![t("a"), t("b"), t("c")],
    ]);
    assert_eq!(hlookup_fn(&[n(99.0), range, n(2.0), Value::Bool(false)]), Value::Error(ErrorKind::NA));
}

#[test]
fn hlookup_wrong_arg_count() {
    assert_eq!(hlookup_fn(&[]), Value::Error(ErrorKind::NA));
    assert_eq!(hlookup_fn(&[n(1.0), n(2.0)]), Value::Error(ErrorKind::NA));
}

#[test]
fn hlookup_row_index_out_of_range() {
    let range = make_2d(vec![vec![n(1.0), n(2.0)]]);
    assert_eq!(hlookup_fn(&[n(1.0), range, n(5.0), Value::Bool(false)]), Value::Error(ErrorKind::Ref));
}

// MATCH
#[test]
fn match_exact_not_found() {
    let arr = make_1d(vec![n(1.0), n(2.0), n(3.0)]);
    assert_eq!(match_fn(&[n(99.0), arr, n(0.0)]), Value::Error(ErrorKind::NA));
}

#[test]
fn match_approximate_ascending_not_found() {
    let arr = make_1d(vec![n(1.0), n(2.0), n(3.0)]);
    // nothing <= 0
    assert_eq!(match_fn(&[n(0.0), arr, n(1.0)]), Value::Error(ErrorKind::NA));
}

#[test]
fn match_wrong_arg_count() {
    assert_eq!(match_fn(&[]), Value::Error(ErrorKind::NA));
    assert_eq!(match_fn(&[n(1.0)]), Value::Error(ErrorKind::NA));
}

// INDEX
#[test]
fn index_1d_out_of_bounds() {
    let arr = make_1d(vec![t("a"), t("b")]);
    assert_eq!(index_fn(&[arr, n(5.0)]), Value::Error(ErrorKind::Ref));
}

#[test]
fn index_2d_row_out_of_bounds() {
    let arr = make_2d(vec![vec![n(1.0), n(2.0)]]);
    assert_eq!(index_fn(&[arr, n(5.0), n(1.0)]), Value::Error(ErrorKind::Ref));
}

#[test]
fn index_wrong_arg_count() {
    assert_eq!(index_fn(&[]), Value::Error(ErrorKind::NA));
    assert_eq!(index_fn(&[n(1.0)]), Value::Error(ErrorKind::NA));
}

// LOOKUP failures
#[test]
fn lookup_not_found_returns_na() {
    let search = make_1d(vec![n(5.0), n(6.0), n(7.0)]);
    assert_eq!(lookup_fn(&[n(1.0), search]), Value::Error(ErrorKind::NA));
}

#[test]
fn xlookup_not_found_no_fallback_returns_na() {
    let lookup = make_1d(vec![n(1.0), n(2.0)]);
    let result = make_1d(vec![t("a"), t("b")]);
    assert_eq!(xlookup_fn(&[n(9.0), lookup, result]), Value::Error(ErrorKind::NA));
}

#[test]
fn xmatch_not_found_returns_na() {
    let lookup = make_1d(vec![n(1.0), n(2.0)]);
    assert_eq!(xmatch_fn(&[n(9.0), lookup]), Value::Error(ErrorKind::NA));
}

#[test]
fn xmatch_invalid_mode_returns_value_error() {
    let lookup = make_1d(vec![n(1.0)]);
    assert_eq!(xmatch_fn(&[n(1.0), lookup, n(99.0)]), Value::Error(ErrorKind::Value));
}

#[test]
fn xlookup_invalid_match_mode_returns_not_found_fallback() {
    let lookup = make_1d(vec![n(1.0)]);
    let result = make_1d(vec![t("a")]);
    // mode=99 → falls through to exact match, key not found → uses if_not_found fallback
    assert_eq!(
        xlookup_fn(&[n(9.0), lookup, result, t("n/a"), n(99.0)]),
        t("n/a"),
    );
}
