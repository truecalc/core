use super::super::countifs_fn;
use crate::types::Value;

fn nums(ns: &[f64]) -> Value {
    Value::Array(ns.iter().map(|&n| Value::Number(n)).collect())
}

fn texts(ss: &[&str]) -> Value {
    Value::Array(ss.iter().map(|s| Value::Text(s.to_string())).collect())
}

#[test]
fn single_criterion_gt() {
    // COUNTIFS({1,2,3,4,5}, ">2") → 3
    let result = countifs_fn(&[nums(&[1.0, 2.0, 3.0, 4.0, 5.0]), Value::Text(">2".to_string())]);
    assert_eq!(result, Value::Number(3.0));
}

#[test]
fn two_criteria_both_match() {
    // COUNTIFS({1,2,3,4,5}, ">1", {1,2,3,4,5}, "<5") → 3
    let range = nums(&[1.0, 2.0, 3.0, 4.0, 5.0]);
    let result = countifs_fn(&[
        range.clone(),
        Value::Text(">1".to_string()),
        range.clone(),
        Value::Text("<5".to_string()),
    ]);
    assert_eq!(result, Value::Number(3.0));
}

#[test]
fn two_criteria_different_ranges() {
    // COUNTIFS({1,2,3,4}, ">1", {10,20,30,40}, ">20") → 2 (rows 3,4: 3→30, 4→40)
    let result = countifs_fn(&[
        nums(&[1.0, 2.0, 3.0, 4.0]),
        Value::Text(">1".to_string()),
        nums(&[10.0, 20.0, 30.0, 40.0]),
        Value::Text(">20".to_string()),
    ]);
    assert_eq!(result, Value::Number(2.0));
}

#[test]
fn single_criterion_exact_number() {
    // COUNTIFS({1,2,3,2,1}, 2) → 2
    let result = countifs_fn(&[
        nums(&[1.0, 2.0, 3.0, 2.0, 1.0]),
        Value::Number(2.0),
    ]);
    assert_eq!(result, Value::Number(2.0));
}

#[test]
fn text_criterion() {
    // COUNTIFS({"a","b","a","c"}, "a") → 2
    let result = countifs_fn(&[
        texts(&["a", "b", "a", "c"]),
        Value::Text("a".to_string()),
    ]);
    assert_eq!(result, Value::Number(2.0));
}

#[test]
fn wildcard_criterion() {
    // COUNTIFS({"apple","apricot","banana"}, "a*") → 2
    let result = countifs_fn(&[
        texts(&["apple", "apricot", "banana"]),
        Value::Text("a*".to_string()),
    ]);
    assert_eq!(result, Value::Number(2.0));
}

#[test]
fn ne_criterion() {
    // COUNTIFS({1,2,3,2,1}, "<>2") → 3
    let result = countifs_fn(&[
        nums(&[1.0, 2.0, 3.0, 2.0, 1.0]),
        Value::Text("<>2".to_string()),
    ]);
    assert_eq!(result, Value::Number(3.0));
}
