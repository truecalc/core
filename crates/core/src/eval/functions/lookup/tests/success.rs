use crate::evaluate;
use crate::eval::functions::lookup::{
    index_match::{index_fn, match_fn},
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
