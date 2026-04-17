use super::super::{filter_fn, index_fn, sort_fn};
use crate::types::Value;

fn nums(ns: &[f64]) -> Value {
    Value::Array(ns.iter().map(|&n| Value::Number(n)).collect())
}

// ---------------------------------------------------------------------------
// FILTER edge cases
// ---------------------------------------------------------------------------

#[test]
fn filter_scalar_array_truthy_include() {
    // FILTER(scalar, true) → scalar
    let result = filter_fn(&[Value::Number(42.0), Value::Bool(true)]);
    assert_eq!(result, Value::Number(42.0));
}

#[test]
fn filter_scalar_array_falsy_include() {
    // FILTER(scalar, false) → #N/A
    use crate::types::ErrorKind;
    let result = filter_fn(&[Value::Number(42.0), Value::Bool(false)]);
    assert_eq!(result, Value::Error(ErrorKind::NA));
}

#[test]
fn filter_scalar_include_true_for_all() {
    // FILTER({1,2,3}, true) → {1,2,3}  (scalar include = keep all)
    let array = nums(&[1.0, 2.0, 3.0]);
    let result = filter_fn(&[array.clone(), Value::Bool(true)]);
    assert_eq!(result, array);
}

#[test]
fn filter_numeric_mask_nonzero_is_truthy() {
    // FILTER({1,2,3}, {1,0,1}) → {1,3}
    let array = nums(&[1.0, 2.0, 3.0]);
    let mask = Value::Array(vec![
        Value::Number(1.0),
        Value::Number(0.0),
        Value::Number(1.0),
    ]);
    let result = filter_fn(&[array, mask]);
    assert_eq!(result, nums(&[1.0, 3.0]));
}

// ---------------------------------------------------------------------------
// SORT edge cases
// ---------------------------------------------------------------------------

#[test]
fn sort_single_element_2d() {
    // SORT({{5}}) → {{5}}
    let array = Value::Array(vec![Value::Array(vec![Value::Number(5.0)])]);
    let result = sort_fn(&[array.clone()]);
    assert_eq!(result, array);
}

#[test]
fn sort_scalar_returns_scalar() {
    // SORT(42) → 42
    let result = sort_fn(&[Value::Number(42.0)]);
    assert_eq!(result, Value::Number(42.0));
}

// ---------------------------------------------------------------------------
// INDEX edge cases
// ---------------------------------------------------------------------------

#[test]
fn index_row_vector_access_via_col() {
    // INDEX({10,20,30}, 1, 2) → 20  (row=1, col=2 treats array as row vector)
    let result = index_fn(&[
        nums(&[10.0, 20.0, 30.0]),
        Value::Number(1.0),
        Value::Number(2.0),
    ]);
    assert_eq!(result, Value::Number(20.0));
}

#[test]
fn index_scalar_returns_itself() {
    // INDEX(7, 1) → 7
    let result = index_fn(&[Value::Number(7.0), Value::Number(1.0)]);
    assert_eq!(result, Value::Number(7.0));
}
