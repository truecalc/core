use super::super::countunique_fn;
use crate::types::Value;

#[test]
fn three_unique_from_six() {
    // COUNTUNIQUE(1,2,2,3,3,3) → 3
    let result = countunique_fn(&[
        Value::Number(1.0),
        Value::Number(2.0),
        Value::Number(2.0),
        Value::Number(3.0),
        Value::Number(3.0),
        Value::Number(3.0),
    ]);
    assert_eq!(result, Value::Number(3.0));
}

#[test]
fn two_unique_text() {
    // COUNTUNIQUE("a","b","a") → 2
    let result = countunique_fn(&[
        Value::Text("a".into()),
        Value::Text("b".into()),
        Value::Text("a".into()),
    ]);
    assert_eq!(result, Value::Number(2.0));
}

#[test]
fn single_value() {
    // COUNTUNIQUE(1) → 1
    let result = countunique_fn(&[Value::Number(1.0)]);
    assert_eq!(result, Value::Number(1.0));
}

#[test]
fn array_with_duplicates() {
    // COUNTUNIQUE({1,2,2,3}) → 3
    let arr = Value::Array(vec![
        Value::Number(1.0),
        Value::Number(2.0),
        Value::Number(2.0),
        Value::Number(3.0),
    ]);
    let result = countunique_fn(&[arr]);
    assert_eq!(result, Value::Number(3.0));
}

#[test]
fn all_same_returns_one() {
    // COUNTUNIQUE(5,5,5) → 1
    let result = countunique_fn(&[
        Value::Number(5.0),
        Value::Number(5.0),
        Value::Number(5.0),
    ]);
    assert_eq!(result, Value::Number(1.0));
}

#[test]
fn five_unique_values() {
    // COUNTUNIQUE(1,2,3,4,5) → 5
    let result = countunique_fn(&[
        Value::Number(1.0),
        Value::Number(2.0),
        Value::Number(3.0),
        Value::Number(4.0),
        Value::Number(5.0),
    ]);
    assert_eq!(result, Value::Number(5.0));
}

#[test]
fn mixed_types_number_text_bool() {
    // COUNTUNIQUE(1, "1", TRUE) → 3 (all different types)
    let result = countunique_fn(&[
        Value::Number(1.0),
        Value::Text("1".into()),
        Value::Bool(true),
    ]);
    assert_eq!(result, Value::Number(3.0));
}

#[test]
fn text_case_insensitive() {
    // COUNTUNIQUE("a","b","A") → 2 (case-insensitive)
    let result = countunique_fn(&[
        Value::Text("a".into()),
        Value::Text("b".into()),
        Value::Text("A".into()),
    ]);
    assert_eq!(result, Value::Number(2.0));
}
