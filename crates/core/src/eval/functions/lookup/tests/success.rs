use crate::evaluate;
use crate::eval::functions::lookup::{
    index_match::{index_fn, match_fn},
    lookup_fn::{lookup_fn, xlookup_fn, xmatch_fn},
    vlookup::{hlookup_fn, vlookup_fn},
};
use crate::types::Value;
use std::collections::HashMap;

fn make_2d(rows: Vec<Vec<Value>>) -> Value {
    Value::Array(rows.into_iter().map(Value::Array).collect())
}

fn make_1d(vals: Vec<Value>) -> Value {
    Value::Array(vals)
}

fn t(s: &str) -> Value {
    Value::Text(s.to_string())
}

fn n(v: f64) -> Value {
    Value::Number(v)
}

fn run(formula: &str) -> Value {
    evaluate(formula, &HashMap::new())
}

// CHOOSE
#[test]
fn choose_first() {
    assert_eq!(run(r#"CHOOSE(1, "a", "b", "c")"#), t("a"));
}

#[test]
fn choose_last() {
    assert_eq!(run(r#"CHOOSE(3, "x", "y", "z")"#), t("z"));
}

#[test]
fn choose_middle_number() {
    assert_eq!(run("CHOOSE(2, 10, 20, 30)"), n(20.0));
}

// VLOOKUP
#[test]
fn vlookup_exact_match_found() {
    let range = make_2d(vec![
        vec![n(1.0), t("a")],
        vec![n(2.0), t("b")],
        vec![n(3.0), t("c")],
    ]);
    assert_eq!(vlookup_fn(&[n(2.0), range, n(2.0), Value::Bool(false)]), t("b"));
}

#[test]
fn vlookup_exact_match_col_1() {
    let range = make_2d(vec![vec![n(5.0), t("x")], vec![n(10.0), t("y")]]);
    assert_eq!(vlookup_fn(&[n(10.0), range, n(1.0), Value::Bool(false)]), n(10.0));
}

#[test]
fn vlookup_approximate_match() {
    let range = make_2d(vec![
        vec![n(1.0), t("a")],
        vec![n(3.0), t("b")],
        vec![n(5.0), t("c")],
    ]);
    // largest <= 4 is 3 → "b"
    assert_eq!(vlookup_fn(&[n(4.0), range, n(2.0), Value::Bool(true)]), t("b"));
}

// HLOOKUP
#[test]
fn hlookup_exact_match_found() {
    let range = make_2d(vec![
        vec![n(1.0), n(2.0), n(3.0)],
        vec![t("a"), t("b"), t("c")],
    ]);
    assert_eq!(hlookup_fn(&[n(2.0), range, n(2.0), Value::Bool(false)]), t("b"));
}

// MATCH
#[test]
fn match_exact_found() {
    let arr = make_1d(vec![n(1.0), n(2.0), n(3.0)]);
    assert_eq!(match_fn(&[n(2.0), arr, n(0.0)]), n(2.0));
}

#[test]
fn match_approximate_ascending() {
    let arr = make_1d(vec![n(1.0), n(2.0), n(3.0)]);
    assert_eq!(match_fn(&[n(2.0), arr, n(1.0)]), n(2.0));
}

// INDEX
#[test]
fn index_1d_first_element() {
    let arr = make_1d(vec![t("a"), t("b"), t("c")]);
    assert_eq!(index_fn(&[arr, n(1.0)]), t("a"));
}

#[test]
fn index_1d_second_element() {
    let arr = make_1d(vec![t("a"), t("b"), t("c")]);
    assert_eq!(index_fn(&[arr, n(2.0)]), t("b"));
}

#[test]
fn index_2d_row2_col1() {
    let arr = make_2d(vec![vec![n(1.0), n(2.0)], vec![n(3.0), n(4.0)]]);
    assert_eq!(index_fn(&[arr, n(2.0), n(1.0)]), n(3.0));
}

#[test]
fn index_2d_row1_col2() {
    let arr = make_2d(vec![vec![n(1.0), n(2.0)], vec![n(3.0), n(4.0)]]);
    assert_eq!(index_fn(&[arr, n(1.0), n(2.0)]), n(2.0));
}

// LOOKUP
#[test]
fn lookup_finds_exact_match() {
    let search = make_1d(vec![n(1.0), n(2.0), n(3.0), n(4.0)]);
    assert_eq!(lookup_fn(&[n(3.0), search]), n(3.0));
}

#[test]
fn lookup_returns_from_result_range() {
    let keys = make_1d(vec![n(1.0), n(2.0), n(3.0)]);
    let vals = make_1d(vec![t("a"), t("b"), t("c")]);
    assert_eq!(lookup_fn(&[n(2.0), keys, vals]), t("b"));
}

#[test]
fn lookup_approximate_finds_largest_lte() {
    let search = make_1d(vec![n(1.0), n(2.0), n(3.0)]);
    assert_eq!(lookup_fn(&[n(2.5), search]), n(2.0));
}

// XLOOKUP
#[test]
fn xlookup_exact_match_found() {
    let lookup = make_1d(vec![n(1.0), n(2.0), n(3.0)]);
    let result = make_1d(vec![t("a"), t("b"), t("c")]);
    assert_eq!(xlookup_fn(&[n(2.0), lookup, result]), t("b"));
}

#[test]
fn xlookup_not_found_returns_if_not_found() {
    let lookup = make_1d(vec![n(1.0), n(2.0)]);
    let result = make_1d(vec![t("a"), t("b")]);
    assert_eq!(xlookup_fn(&[n(9.0), lookup, result, t("missing")]), t("missing"));
}

#[test]
fn xlookup_match_mode_1_next_larger() {
    let lookup = make_1d(vec![n(1.0), n(2.0), n(3.0)]);
    let result = make_1d(vec![t("a"), t("b"), t("c")]);
    assert_eq!(xlookup_fn(&[n(2.5), lookup, result, t("n/a"), n(1.0)]), t("c"));
}

#[test]
fn xlookup_match_mode_neg1_next_smaller() {
    let lookup = make_1d(vec![n(1.0), n(2.0), n(3.0)]);
    let result = make_1d(vec![t("a"), t("b"), t("c")]);
    assert_eq!(xlookup_fn(&[n(2.5), lookup, result, t("n/a"), n(-1.0)]), t("b"));
}

// XMATCH
#[test]
fn xmatch_exact_returns_1based_position() {
    let lookup = make_1d(vec![t("a"), t("b"), t("c")]);
    assert_eq!(xmatch_fn(&[t("b"), lookup]), n(2.0));
}

#[test]
fn xmatch_mode_1_returns_position_of_lte() {
    let lookup = make_1d(vec![n(1.0), n(2.0), n(3.0)]);
    assert_eq!(xmatch_fn(&[n(2.5), lookup, n(1.0)]), n(2.0));
}

#[test]
fn xmatch_mode_neg1_returns_position_of_gte() {
    let lookup = make_1d(vec![n(3.0), n(2.0), n(1.0)]);
    assert_eq!(xmatch_fn(&[n(2.5), lookup, n(-1.0)]), n(1.0));
}

// ROW / COLUMN
#[test]
fn row_no_args_returns_1() {
    assert_eq!(run("=ROW()"), n(1.0));
}

#[test]
fn column_no_args_returns_1() {
    assert_eq!(run("=COLUMN()"), n(1.0));
}

#[test]
fn row_with_cell_ref_returns_row_number() {
    assert_eq!(run("=ROW(A5)"), n(5.0));
    assert_eq!(run("=ROW(B10)"), n(10.0));
}

#[test]
fn column_with_cell_ref_returns_col_number() {
    assert_eq!(run("=COLUMN(A1)"), n(1.0));
    assert_eq!(run("=COLUMN(B1)"), n(2.0));
    assert_eq!(run("=COLUMN(D1)"), n(4.0));
}

