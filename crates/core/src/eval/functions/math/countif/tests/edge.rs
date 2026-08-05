use super::super::countif_fn;
use crate::types::Value;

fn nums(ns: &[f64]) -> Value {
    Value::Array(ns.iter().map(|&n| Value::Number(n)).collect())
}

fn texts(ss: &[&str]) -> Value {
    Value::Array(ss.iter().map(|s| Value::Text(s.to_string())).collect())
}

#[test]
fn no_matches_returns_zero() {
    // COUNTIF({1,2,3}, ">10") → 0
    let result = countif_fn(&[nums(&[1.0, 2.0, 3.0]), Value::Text(">10".to_string())]);
    assert_eq!(result, Value::Number(0.0));
}

#[test]
fn empty_array_returns_zero() {
    let result = countif_fn(&[Value::Array(vec![]), Value::Number(1.0)]);
    assert_eq!(result, Value::Number(0.0));
}

#[test]
fn wildcard_star_does_not_match_numbers() {
    // "*" is a text wildcard — numbers should not match it.
    let result = countif_fn(&[nums(&[1.0, 2.0, 3.0]), Value::Text("*".to_string())]);
    assert_eq!(result, Value::Number(0.0));
}

#[test]
fn eq_prefix_on_number_string() {
    // "=2" is the same as 2 for exact match.
    let result = countif_fn(&[nums(&[1.0, 2.0, 3.0, 2.0]), Value::Text("=2".to_string())]);
    assert_eq!(result, Value::Number(2.0));
}

#[test]
fn criterion_text_ne_text() {
    // COUNTIF({"a","b","c"}, "<>b") → 2
    let result = countif_fn(&[
        texts(&["a", "b", "c"]),
        Value::Text("<>b".to_string()),
    ]);
    assert_eq!(result, Value::Number(2.0));
}

#[test]
fn bool_criterion_true() {
    // COUNTIF({TRUE,FALSE,TRUE}, TRUE) → 2
    let arr = Value::Array(vec![Value::Bool(true), Value::Bool(false), Value::Bool(true)]);
    let result = countif_fn(&[arr, Value::Bool(true)]);
    assert_eq!(result, Value::Number(2.0));
}
