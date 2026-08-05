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

// ── ROWS / COLUMNS: scalar and flat-array inputs ──────────────────────────────

#[test]
fn rows_scalar() {
    assert_eq!(rows_fn(&[num(42.0)]), num(1.0));
}

#[test]
fn rows_1d_flat_array() {
    let arr = Value::Array(vec![num(1.0), num(2.0), num(3.0)]);
    assert_eq!(rows_fn(&[arr]), num(1.0));
}

#[test]
fn columns_scalar() {
    assert_eq!(columns_fn(&[num(42.0)]), num(1.0));
}

#[test]
fn columns_1d_flat_array() {
    let arr = Value::Array(vec![num(1.0), num(2.0), num(3.0)]);
    assert_eq!(columns_fn(&[arr]), num(3.0));
}

// ── TRANSPOSE: scalar and 1D inputs ──────────────────────────────────────────

#[test]
fn transpose_1d_row_to_column() {
    let input = Value::Array(vec![num(1.0), num(2.0), num(3.0)]);
    let expected = Value::Array(vec![
        Value::Array(vec![num(1.0)]),
        Value::Array(vec![num(2.0)]),
        Value::Array(vec![num(3.0)]),
    ]);
    assert_eq!(transpose_fn(&[input]), expected);
}

#[test]
fn transpose_scalar() {
    // to_2d(5) → [[5]], transpose → [[5]], from_2d → Array([5])
    assert_eq!(transpose_fn(&[num(5.0)]), Value::Array(vec![num(5.0)]));
}

// ── SORT: already-sorted and single-element ───────────────────────────────────

#[test]
fn sort_already_sorted() {
    assert_eq!(sort_fn(&[col(&[1.0, 2.0, 3.0])]), col(&[1.0, 2.0, 3.0]));
}

#[test]
fn sort_single_element() {
    // Single-row 2D → from_2d returns flat Array
    assert_eq!(sort_fn(&[make_2d(&[&[7.0]])]), Value::Array(vec![num(7.0)]));
}

// ── UNIQUE: no duplicates and all-same ────────────────────────────────────────

#[test]
fn unique_already_unique() {
    assert_eq!(unique_fn(&[col(&[4.0, 5.0, 6.0])]), col(&[4.0, 5.0, 6.0]));
}

#[test]
fn unique_all_same() {
    // from_2d with 1 row → flat Array
    assert_eq!(
        unique_fn(&[col(&[7.0, 7.0, 7.0])]),
        Value::Array(vec![num(7.0)])
    );
}

// ── ARRAY_CONSTRAIN: request larger than array ────────────────────────────────

#[test]
fn constrain_larger_than_array() {
    let arr = make_2d(&[&[1.0, 2.0], &[3.0, 4.0]]);
    assert_eq!(array_constrain_fn(&[arr.clone(), num(10.0), num(10.0)]), arr);
}

// ── FLATTEN: scalar input ─────────────────────────────────────────────────────

#[test]
fn flatten_scalar() {
    // 1-element → from_2d returns flat single-element Array
    assert_eq!(flatten_fn(&[num(5.0)]), Value::Array(vec![num(5.0)]));
}

// ── SUMPRODUCT: additional cases ──────────────────────────────────────────────

#[test]
fn sumproduct_three_arrays() {
    // 1*4*7 + 2*5*8 + 3*6*9 = 28 + 80 + 162 = 270
    assert_eq!(
        sumproduct_fn(&[
            flat(&[1.0, 2.0, 3.0]),
            flat(&[4.0, 5.0, 6.0]),
            flat(&[7.0, 8.0, 9.0]),
        ]),
        num(270.0)
    );
}

#[test]
fn sumproduct_single_element_arrays() {
    // SUMPRODUCT([5], [3]) = 15
    assert_eq!(
        sumproduct_fn(&[flat(&[5.0]), flat(&[3.0])]),
        num(15.0)
    );
}

#[test]
fn sumproduct_with_zeros() {
    // Any element zero → entire term is zero
    assert_eq!(
        sumproduct_fn(&[flat(&[0.0, 2.0, 3.0]), flat(&[1.0, 0.0, 5.0])]),
        num(15.0) // 0*1 + 2*0 + 3*5
    );
}

#[test]
fn sumproduct_negative_values() {
    // (-1)*1 + (-2)*(-2) + 3*3 = -1 + 4 + 9 = 12
    assert_eq!(
        sumproduct_fn(&[flat(&[-1.0, -2.0, 3.0]), flat(&[1.0, -2.0, 3.0])]),
        num(12.0)
    );
}
