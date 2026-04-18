use super::super::countif_fn;
use crate::types::Value;

fn nums(ns: &[f64]) -> Value {
    Value::Array(ns.iter().map(|&n| Value::Number(n)).collect())
}

fn texts(ss: &[&str]) -> Value {
    Value::Array(ss.iter().map(|s| Value::Text(s.to_string())).collect())
}

#[test]
fn count_exact_number() {
    // COUNTIF({1,2,3,2,1}, 2) → 2
    let result = countif_fn(&[nums(&[1.0, 2.0, 3.0, 2.0, 1.0]), Value::Number(2.0)]);
    assert_eq!(result, Value::Number(2.0));
}

#[test]
fn count_gt_criterion() {
    // COUNTIF({1,2,3,4,5}, ">2") → 3
    let result = countif_fn(&[nums(&[1.0, 2.0, 3.0, 4.0, 5.0]), Value::Text(">2".to_string())]);
    assert_eq!(result, Value::Number(3.0));
}

#[test]
fn count_lt_criterion() {
    // COUNTIF({1,2,3,4,5}, "<3") → 2
    let result = countif_fn(&[nums(&[1.0, 2.0, 3.0, 4.0, 5.0]), Value::Text("<3".to_string())]);
    assert_eq!(result, Value::Number(2.0));
}

#[test]
fn count_lte_criterion() {
    // COUNTIF({1,2,3,4,5}, "<=3") → 3
    let result = countif_fn(&[nums(&[1.0, 2.0, 3.0, 4.0, 5.0]), Value::Text("<=3".to_string())]);
    assert_eq!(result, Value::Number(3.0));
}

#[test]
fn count_gte_criterion() {
    // COUNTIF({1,2,3,4,5}, ">=2") → 4
    let result = countif_fn(&[nums(&[1.0, 2.0, 3.0, 4.0, 5.0]), Value::Text(">=2".to_string())]);
    assert_eq!(result, Value::Number(4.0));
}

#[test]
fn count_ne_criterion() {
    // COUNTIF({1,2,3,2,1}, "<>2") → 3
    let result = countif_fn(&[nums(&[1.0, 2.0, 3.0, 2.0, 1.0]), Value::Text("<>2".to_string())]);
    assert_eq!(result, Value::Number(3.0));
}

#[test]
fn count_exact_text() {
    // COUNTIF({"a","b","a","c"}, "a") → 2
    let result = countif_fn(&[texts(&["a", "b", "a", "c"]), Value::Text("a".to_string())]);
    assert_eq!(result, Value::Number(2.0));
}

#[test]
fn count_text_case_insensitive() {
    // COUNTIF({"Apple","apple","APPLE","banana"}, "apple") → 3
    let result = countif_fn(&[
        texts(&["Apple", "apple", "APPLE", "banana"]),
        Value::Text("apple".to_string()),
    ]);
    assert_eq!(result, Value::Number(3.0));
}

#[test]
fn count_wildcard_star_all_text() {
    // COUNTIF({"a","b","c"}, "*") → 3
    let result = countif_fn(&[texts(&["a", "b", "c"]), Value::Text("*".to_string())]);
    assert_eq!(result, Value::Number(3.0));
}

#[test]
fn count_wildcard_prefix_star() {
    // COUNTIF({"apt","ape","app","bat"}, "ap*") → 3
    let result = countif_fn(&[
        texts(&["apt", "ape", "app", "bat"]),
        Value::Text("ap*".to_string()),
    ]);
    assert_eq!(result, Value::Number(3.0));
}

#[test]
fn count_wildcard_question() {
    // COUNTIF({"apt","ape","app","bat"}, "ap?") → 3
    let result = countif_fn(&[
        texts(&["apt", "ape", "app", "bat"]),
        Value::Text("ap?".to_string()),
    ]);
    assert_eq!(result, Value::Number(3.0));
}

#[test]
fn count_scalar_range() {
    // COUNTIF(5, 5) → 1 (scalar treated as single-element range)
    let result = countif_fn(&[Value::Number(5.0), Value::Number(5.0)]);
    assert_eq!(result, Value::Number(1.0));
}
