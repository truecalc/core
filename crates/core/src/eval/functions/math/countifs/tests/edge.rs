use super::super::countifs_fn;
use crate::types::Value;

fn nums(ns: &[f64]) -> Value {
    Value::Array(ns.iter().map(|&n| Value::Number(n)).collect())
}

#[test]
fn no_matches_returns_zero() {
    // COUNTIFS({1,2,3}, ">5", {1,2,3}, "<10") → 0
    let range = nums(&[1.0, 2.0, 3.0]);
    let result = countifs_fn(&[
        range.clone(),
        Value::Text(">5".to_string()),
        range.clone(),
        Value::Text("<10".to_string()),
    ]);
    assert_eq!(result, Value::Number(0.0));
}

#[test]
fn all_match_returns_full_count() {
    // COUNTIFS({1,2,3}, ">=1", {1,2,3}, "<=3") → 3
    let range = nums(&[1.0, 2.0, 3.0]);
    let result = countifs_fn(&[
        range.clone(),
        Value::Text(">=1".to_string()),
        range.clone(),
        Value::Text("<=3".to_string()),
    ]);
    assert_eq!(result, Value::Number(3.0));
}

#[test]
fn empty_array_returns_zero() {
    let result = countifs_fn(&[Value::Array(vec![]), Value::Number(1.0)]);
    assert_eq!(result, Value::Number(0.0));
}

#[test]
fn scalar_range_single_element() {
    // COUNTIFS(5, 5) → 1 (scalar treated as single-element range)
    let result = countifs_fn(&[Value::Number(5.0), Value::Number(5.0)]);
    assert_eq!(result, Value::Number(1.0));
}
