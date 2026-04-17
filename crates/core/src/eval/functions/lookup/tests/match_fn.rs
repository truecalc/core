use crate::eval::functions::lookup::index_match::match_fn;
use crate::types::{ErrorKind, Value};

fn arr(ns: &[f64]) -> Value {
    Value::Array(ns.iter().map(|&v| Value::Number(v)).collect())
}

fn n(v: f64) -> Value {
    Value::Number(v)
}

// MATCH(2, [1,2,3], 0) → 2 (exact, 1-based)
#[test]
fn exact_match_found() {
    let result = match_fn(&[n(2.0), arr(&[1.0, 2.0, 3.0]), n(0.0)]);
    assert_eq!(result, n(2.0));
}

// MATCH(99, [1,2,3], 0) → #N/A
#[test]
fn exact_match_not_found() {
    let result = match_fn(&[n(99.0), arr(&[1.0, 2.0, 3.0]), n(0.0)]);
    assert_eq!(result, Value::Error(ErrorKind::NA));
}

// MATCH(2, [1,2,3], 1) → 2 (approximate ascending: largest <= 2 is at position 2)
#[test]
fn approximate_ascending_match() {
    let result = match_fn(&[n(2.0), arr(&[1.0, 2.0, 3.0]), n(1.0)]);
    assert_eq!(result, n(2.0));
}

// MATCH(2.5, [1,2,3], 1) → 2 (approximate ascending: largest <= 2.5 is 2, position 2)
#[test]
fn approximate_ascending_between_values() {
    let result = match_fn(&[n(2.5), arr(&[1.0, 2.0, 3.0]), n(1.0)]);
    assert_eq!(result, n(2.0));
}

// MATCH(0, [1,2,3], 1) → #N/A (nothing <= 0)
#[test]
fn approximate_ascending_not_found() {
    let result = match_fn(&[n(0.0), arr(&[1.0, 2.0, 3.0]), n(1.0)]);
    assert_eq!(result, Value::Error(ErrorKind::NA));
}

// MATCH wrong arg count → #N/A
#[test]
fn wrong_arg_count() {
    assert_eq!(match_fn(&[]), Value::Error(ErrorKind::NA));
    assert_eq!(match_fn(&[n(1.0)]), Value::Error(ErrorKind::NA));
}

// MATCH first element → 1
#[test]
fn exact_match_first_element() {
    let result = match_fn(&[n(1.0), arr(&[1.0, 2.0, 3.0]), n(0.0)]);
    assert_eq!(result, n(1.0));
}

// MATCH last element → 3
#[test]
fn exact_match_last_element() {
    let result = match_fn(&[n(3.0), arr(&[1.0, 2.0, 3.0]), n(0.0)]);
    assert_eq!(result, n(3.0));
}
