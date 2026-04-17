use crate::eval::functions::lookup::index_match::index_fn;
use crate::types::{ErrorKind, Value};

fn make_1d(vals: Vec<Value>) -> Value {
    Value::Array(vals)
}

fn make_2d(rows: Vec<Vec<Value>>) -> Value {
    Value::Array(rows.into_iter().map(|r| Value::Array(r)).collect())
}

fn t(s: &str) -> Value {
    Value::Text(s.to_string())
}

fn n(v: f64) -> Value {
    Value::Number(v)
}

// INDEX(["a","b","c"], 1) → "a" (1-based)
#[test]
fn one_d_first_element() {
    let arr = make_1d(vec![t("a"), t("b"), t("c")]);
    let result = index_fn(&[arr, n(1.0)]);
    assert_eq!(result, t("a"));
}

// INDEX(["a","b","c"], 2) → "b"
#[test]
fn one_d_second_element() {
    let arr = make_1d(vec![t("a"), t("b"), t("c")]);
    let result = index_fn(&[arr, n(2.0)]);
    assert_eq!(result, t("b"));
}

// INDEX([[1,2],[3,4]], 2, 1) → 3 (2D: row 2, col 1)
#[test]
fn two_d_row2_col1() {
    let arr = make_2d(vec![vec![n(1.0), n(2.0)], vec![n(3.0), n(4.0)]]);
    let result = index_fn(&[arr, n(2.0), n(1.0)]);
    assert_eq!(result, n(3.0));
}

// INDEX([[1,2],[3,4]], 1, 2) → 2 (2D: row 1, col 2)
#[test]
fn two_d_row1_col2() {
    let arr = make_2d(vec![vec![n(1.0), n(2.0)], vec![n(3.0), n(4.0)]]);
    let result = index_fn(&[arr, n(1.0), n(2.0)]);
    assert_eq!(result, n(2.0));
}

// INDEX out of bounds → #REF!
#[test]
fn one_d_out_of_bounds() {
    let arr = make_1d(vec![t("a"), t("b")]);
    let result = index_fn(&[arr, n(5.0)]);
    assert_eq!(result, Value::Error(ErrorKind::Ref));
}

// INDEX 2D out of bounds row → #REF!
#[test]
fn two_d_row_out_of_bounds() {
    let arr = make_2d(vec![vec![n(1.0), n(2.0)]]);
    let result = index_fn(&[arr, n(5.0), n(1.0)]);
    assert_eq!(result, Value::Error(ErrorKind::Ref));
}

// INDEX wrong arg count → #N/A
#[test]
fn wrong_arg_count() {
    assert_eq!(index_fn(&[]), Value::Error(ErrorKind::NA));
    assert_eq!(index_fn(&[n(1.0)]), Value::Error(ErrorKind::NA));
}
