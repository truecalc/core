use super::super::{columns_fn, index_fn, rows_fn};
use crate::types::Value;

fn row(vals: Vec<Value>) -> Value {
    Value::Array(vals)
}
fn arr2d(rows: Vec<Vec<Value>>) -> Value {
    Value::Array(rows.into_iter().map(|r| row(r)).collect())
}
fn n(v: f64) -> Value {
    Value::Number(v)
}

// ── ROWS ─────────────────────────────────────────────────────────────────────

#[test]
fn rows_2d_array_returns_row_count() {
    let arr = arr2d(vec![vec![n(1.0), n(2.0)], vec![n(3.0), n(4.0)]]);
    assert_eq!(rows_fn(&[arr]), n(2.0));
}

#[test]
fn rows_flat_1d_array_returns_1() {
    let arr = row(vec![n(1.0), n(2.0), n(3.0)]);
    assert_eq!(rows_fn(&[arr]), n(1.0));
}

#[test]
fn rows_scalar_returns_1() {
    assert_eq!(rows_fn(&[n(42.0)]), n(1.0));
}

#[test]
fn rows_text_scalar_returns_1() {
    assert_eq!(rows_fn(&[Value::Text("hello".into())]), n(1.0));
}

// ── COLUMNS ──────────────────────────────────────────────────────────────────

#[test]
fn columns_2d_array_returns_col_count() {
    let arr = arr2d(vec![vec![n(1.0), n(2.0), n(3.0)], vec![n(4.0), n(5.0), n(6.0)]]);
    assert_eq!(columns_fn(&[arr]), n(3.0));
}

#[test]
fn columns_flat_1d_array_returns_element_count() {
    let arr = row(vec![n(1.0), n(2.0), n(3.0)]);
    assert_eq!(columns_fn(&[arr]), n(3.0));
}

#[test]
fn columns_scalar_returns_1() {
    assert_eq!(columns_fn(&[n(42.0)]), n(1.0));
}

// ── INDEX ─────────────────────────────────────────────────────────────────────

#[test]
fn index_2d_first_element() {
    let arr = arr2d(vec![vec![n(10.0), n(20.0)], vec![n(30.0), n(40.0)]]);
    assert_eq!(index_fn(&[arr, n(1.0), n(1.0)]), n(10.0));
}

#[test]
fn index_2d_last_element() {
    let arr = arr2d(vec![vec![n(10.0), n(20.0)], vec![n(30.0), n(40.0)]]);
    assert_eq!(index_fn(&[arr, n(2.0), n(2.0)]), n(40.0));
}

#[test]
fn index_2d_no_col_defaults_to_first_col() {
    let arr = arr2d(vec![vec![n(10.0), n(20.0)], vec![n(30.0), n(40.0)]]);
    assert_eq!(index_fn(&[arr, n(2.0)]), n(30.0));
}

#[test]
fn index_1d_column_vector() {
    let arr = row(vec![n(100.0), n(200.0), n(300.0)]);
    assert_eq!(index_fn(&[arr, n(2.0)]), n(200.0));
}

#[test]
fn index_1d_row_vector_with_col() {
    let arr = row(vec![n(100.0), n(200.0), n(300.0)]);
    assert_eq!(index_fn(&[arr, n(1.0), n(3.0)]), n(300.0));
}

#[test]
fn index_scalar_one_one() {
    assert_eq!(index_fn(&[n(42.0), n(1.0), n(1.0)]), n(42.0));
}

#[test]
fn index_scalar_two_args() {
    assert_eq!(index_fn(&[n(42.0), n(1.0)]), n(42.0));
}
