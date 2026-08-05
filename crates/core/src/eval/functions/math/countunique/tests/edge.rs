use super::super::countunique_fn;
use crate::types::Value;

#[test]
fn empty_strings_ignored() {
    // COUNTUNIQUE("", "", "a") → 1 (empty strings are blank and ignored)
    let result = countunique_fn(&[
        Value::Text("".into()),
        Value::Text("".into()),
        Value::Text("a".into()),
    ]);
    assert_eq!(result, Value::Number(1.0));
}

#[test]
fn empty_value_ignored() {
    // Value::Empty is treated as blank and ignored
    let result = countunique_fn(&[
        Value::Empty,
        Value::Number(1.0),
        Value::Number(2.0),
    ]);
    assert_eq!(result, Value::Number(2.0));
}

#[test]
fn all_empty_returns_zero() {
    // COUNTUNIQUE("", "") → 0
    let result = countunique_fn(&[Value::Text("".into()), Value::Text("".into())]);
    assert_eq!(result, Value::Number(0.0));
}

#[test]
fn mixed_scalar_and_array() {
    // COUNTUNIQUE(1, {2,2,3}) → 3
    let result = countunique_fn(&[
        Value::Number(1.0),
        Value::Array(vec![
            Value::Number(2.0),
            Value::Number(2.0),
            Value::Number(3.0),
        ]),
    ]);
    assert_eq!(result, Value::Number(3.0));
}
