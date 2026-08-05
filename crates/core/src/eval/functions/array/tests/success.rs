use super::super::{
    array_constrain_fn, flatten_fn, rows_fn, columns_fn,
    sort_fn, sumproduct_fn, transpose_fn, unique_fn,
};
use crate::types::Value;

fn num(n: f64) -> Value { Value::Number(n) }

fn flat(ns: &[f64]) -> Value {
    Value::Array(ns.iter().map(|&n| num(n)).collect())
}

fn col(ns: &[f64]) -> Value {
    Value::Array(ns.iter().map(|&n| Value::Array(vec![num(n)])).collect())
}

fn make_2d(rows: &[&[f64]]) -> Value {
    Value::Array(
        rows.iter()
            .map(|row| Value::Array(row.iter().map(|&n| num(n)).collect()))
            .collect(),
    )
}

// ── ROWS / COLUMNS ────────────────────────────────────────────────────────────

#[test]
fn rows_2x3_array() {
    assert_eq!(rows_fn(&[make_2d(&[&[1.0, 2.0, 3.0], &[4.0, 5.0, 6.0]])]), num(2.0));
}

#[test]
fn columns_2x3_array() {
    assert_eq!(columns_fn(&[make_2d(&[&[1.0, 2.0, 3.0], &[4.0, 5.0, 6.0]])]), num(3.0));
}

// ── TRANSPOSE ─────────────────────────────────────────────────────────────────

#[test]
fn transpose_2x2() {
    let input = make_2d(&[&[1.0, 2.0], &[3.0, 4.0]]);
    assert_eq!(transpose_fn(&[input]), make_2d(&[&[1.0, 3.0], &[2.0, 4.0]]));
}

#[test]
fn transpose_2x3_to_3x2() {
    let input = make_2d(&[&[1.0, 2.0, 3.0], &[4.0, 5.0, 6.0]]);
    let expected = make_2d(&[&[1.0, 4.0], &[2.0, 5.0], &[3.0, 6.0]]);
    assert_eq!(transpose_fn(&[input]), expected);
}

// ── SORT ──────────────────────────────────────────────────────────────────────

#[test]
fn sort_ascending() {
    assert_eq!(sort_fn(&[col(&[3.0, 1.0, 2.0])]), col(&[1.0, 2.0, 3.0]));
}

#[test]
fn sort_descending() {
    assert_eq!(
        sort_fn(&[col(&[3.0, 1.0, 2.0]), num(1.0), num(-1.0)]),
        col(&[3.0, 2.0, 1.0])
    );
}

#[test]
fn sort_2d_by_first_col() {
    let input = Value::Array(vec![
        Value::Array(vec![num(3.0), Value::Text("a".to_string())]),
        Value::Array(vec![num(1.0), Value::Text("b".to_string())]),
        Value::Array(vec![num(2.0), Value::Text("c".to_string())]),
    ]);
    let expected = Value::Array(vec![
        Value::Array(vec![num(1.0), Value::Text("b".to_string())]),
        Value::Array(vec![num(2.0), Value::Text("c".to_string())]),
        Value::Array(vec![num(3.0), Value::Text("a".to_string())]),
    ]);
    assert_eq!(sort_fn(&[input, num(1.0), num(1.0)]), expected);
}

// ── UNIQUE ────────────────────────────────────────────────────────────────────

#[test]
fn unique_removes_duplicates() {
    assert_eq!(
        unique_fn(&[col(&[1.0, 2.0, 2.0, 3.0, 1.0])]),
        col(&[1.0, 2.0, 3.0])
    );
}

#[test]
fn unique_2d_deduplicates_rows() {
    let input = Value::Array(vec![
        Value::Array(vec![num(1.0), num(2.0)]),
        Value::Array(vec![num(3.0), num(4.0)]),
        Value::Array(vec![num(1.0), num(2.0)]),
    ]);
    let expected = Value::Array(vec![
        Value::Array(vec![num(1.0), num(2.0)]),
        Value::Array(vec![num(3.0), num(4.0)]),
    ]);
    assert_eq!(unique_fn(&[input]), expected);
}

#[test]
fn unique_exactly_once_flag() {
    // exactly_once=true → only rows appearing once
    assert_eq!(
        unique_fn(&[col(&[1.0, 2.0, 2.0, 3.0]), Value::Bool(false), Value::Bool(true)]),
        col(&[1.0, 3.0])
    );
}

// ── ARRAY_CONSTRAIN ───────────────────────────────────────────────────────────

#[test]
fn constrain_2x3_to_1x2() {
    let arr = make_2d(&[&[1.0, 2.0, 3.0], &[4.0, 5.0, 6.0]]);
    assert_eq!(
        array_constrain_fn(&[arr, num(1.0), num(2.0)]),
        Value::Array(vec![num(1.0), num(2.0)])
    );
}

#[test]
fn constrain_exact_size() {
    let arr = make_2d(&[&[1.0, 2.0], &[3.0, 4.0]]);
    assert_eq!(array_constrain_fn(&[arr.clone(), num(2.0), num(2.0)]), arr);
}

// ── SUMPRODUCT ────────────────────────────────────────────────────────────────

#[test]
fn sumproduct_two_arrays() {
    // 1*4 + 2*5 + 3*6 = 32
    assert_eq!(
        sumproduct_fn(&[flat(&[1.0, 2.0, 3.0]), flat(&[4.0, 5.0, 6.0])]),
        num(32.0)
    );
}

#[test]
fn sumproduct_single_array() {
    assert_eq!(sumproduct_fn(&[flat(&[2.0, 3.0, 4.0])]), num(9.0));
}

// ── FLATTEN ───────────────────────────────────────────────────────────────────

#[test]
fn flatten_2d_array() {
    let arr = make_2d(&[&[1.0, 2.0], &[3.0, 4.0]]);
    assert_eq!(flatten_fn(&[arr]), col(&[1.0, 2.0, 3.0, 4.0]));
}

#[test]
fn flatten_1d_array() {
    let arr = Value::Array(vec![num(1.0), num(2.0), num(3.0)]);
    assert_eq!(flatten_fn(&[arr]), col(&[1.0, 2.0, 3.0]));
}
