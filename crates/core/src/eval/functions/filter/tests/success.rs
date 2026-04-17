use super::super::{columns_fn, filter_fn, index_fn, rows_fn, sort_fn};
use crate::types::Value;

fn nums(ns: &[f64]) -> Value {
    Value::Array(ns.iter().map(|&n| Value::Number(n)).collect())
}

fn bools(bs: &[bool]) -> Value {
    Value::Array(bs.iter().map(|&b| Value::Bool(b)).collect())
}

// ---------------------------------------------------------------------------
// FILTER
// ---------------------------------------------------------------------------

#[test]
fn filter_1d_boolean_mask() {
    // FILTER({1,2,3,4,5}, {true,false,true,false,true}) → {1,3,5}
    let result = filter_fn(&[
        nums(&[1.0, 2.0, 3.0, 4.0, 5.0]),
        bools(&[true, false, true, false, true]),
    ]);
    assert_eq!(result, nums(&[1.0, 3.0, 5.0]));
}

#[test]
fn filter_all_true_returns_original() {
    // FILTER({10,20,30}, {true,true,true}) → {10,20,30}
    let result = filter_fn(&[
        nums(&[10.0, 20.0, 30.0]),
        bools(&[true, true, true]),
    ]);
    assert_eq!(result, nums(&[10.0, 20.0, 30.0]));
}

#[test]
fn filter_with_if_empty_on_no_match() {
    // FILTER({1,2,3}, {false,false,false}, -1) → -1
    let result = filter_fn(&[
        nums(&[1.0, 2.0, 3.0]),
        bools(&[false, false, false]),
        Value::Number(-1.0),
    ]);
    assert_eq!(result, Value::Number(-1.0));
}

// ---------------------------------------------------------------------------
// SORT (1-D arrays are returned unchanged; 2-D arrays are sorted by column)
// ---------------------------------------------------------------------------

#[test]
fn sort_2d_ascending() {
    // SORT({{3},{1},{2}}) by col 1 ascending → {{1},{2},{3}}
    let array = Value::Array(vec![
        Value::Array(vec![Value::Number(3.0)]),
        Value::Array(vec![Value::Number(1.0)]),
        Value::Array(vec![Value::Number(2.0)]),
    ]);
    let result = sort_fn(&[array]);
    let expected = Value::Array(vec![
        Value::Array(vec![Value::Number(1.0)]),
        Value::Array(vec![Value::Number(2.0)]),
        Value::Array(vec![Value::Number(3.0)]),
    ]);
    assert_eq!(result, expected);
}

#[test]
fn sort_2d_descending() {
    // SORT({{1},{3},{2}}, 1, false) → {{3},{2},{1}}
    let array = Value::Array(vec![
        Value::Array(vec![Value::Number(1.0)]),
        Value::Array(vec![Value::Number(3.0)]),
        Value::Array(vec![Value::Number(2.0)]),
    ]);
    let result = sort_fn(&[array, Value::Number(1.0), Value::Bool(false)]);
    let expected = Value::Array(vec![
        Value::Array(vec![Value::Number(3.0)]),
        Value::Array(vec![Value::Number(2.0)]),
        Value::Array(vec![Value::Number(1.0)]),
    ]);
    assert_eq!(result, expected);
}

#[test]
fn sort_1d_returns_unchanged() {
    // For a flat 1-D array, SORT returns it unchanged
    let array = nums(&[3.0, 1.0, 2.0]);
    let result = sort_fn(&[array.clone()]);
    assert_eq!(result, array);
}

// ---------------------------------------------------------------------------
// ROWS
// ---------------------------------------------------------------------------

#[test]
fn rows_of_array_returns_one() {
    // ROWS({1,2,3}) → 1 (flat 1-D array)
    let result = rows_fn(&[nums(&[1.0, 2.0, 3.0])]);
    assert_eq!(result, Value::Number(1.0));
}

#[test]
fn rows_of_scalar_returns_one() {
    // ROWS(42) → 1
    let result = rows_fn(&[Value::Number(42.0)]);
    assert_eq!(result, Value::Number(1.0));
}

// ---------------------------------------------------------------------------
// COLUMNS
// ---------------------------------------------------------------------------

#[test]
fn columns_of_array_returns_length() {
    // COLUMNS({1,2,3,4}) → 4
    let result = columns_fn(&[nums(&[1.0, 2.0, 3.0, 4.0])]);
    assert_eq!(result, Value::Number(4.0));
}

#[test]
fn columns_of_scalar_returns_one() {
    // COLUMNS(42) → 1
    let result = columns_fn(&[Value::Number(42.0)]);
    assert_eq!(result, Value::Number(1.0));
}

// ---------------------------------------------------------------------------
// INDEX
// ---------------------------------------------------------------------------

#[test]
fn index_first_element() {
    // INDEX({10,20,30}, 1) → 10
    let result = index_fn(&[nums(&[10.0, 20.0, 30.0]), Value::Number(1.0)]);
    assert_eq!(result, Value::Number(10.0));
}

#[test]
fn index_last_element() {
    // INDEX({10,20,30}, 3) → 30
    let result = index_fn(&[nums(&[10.0, 20.0, 30.0]), Value::Number(3.0)]);
    assert_eq!(result, Value::Number(30.0));
}

#[test]
fn index_2d_row_and_col() {
    // INDEX({{1,2},{3,4}}, 2, 1) → 3
    let array = Value::Array(vec![
        Value::Array(vec![Value::Number(1.0), Value::Number(2.0)]),
        Value::Array(vec![Value::Number(3.0), Value::Number(4.0)]),
    ]);
    let result = index_fn(&[array, Value::Number(2.0), Value::Number(1.0)]);
    assert_eq!(result, Value::Number(3.0));
}
