use super::super::*;
use crate::types::Value;

#[test]
fn first_element() {
    // INDEX({"a","b","c"}, 1) → "a"
    let arr = Value::Array(vec![
        Value::Text("a".into()),
        Value::Text("b".into()),
        Value::Text("c".into()),
    ]);
    assert_eq!(index_fn(&[arr, Value::Number(1.0)]), Value::Text("a".into()));
}

#[test]
fn middle_element() {
    // INDEX({10, 20, 30}, 2) → 20
    let arr = Value::Array(vec![
        Value::Number(10.0),
        Value::Number(20.0),
        Value::Number(30.0),
    ]);
    assert_eq!(index_fn(&[arr, Value::Number(2.0)]), Value::Number(20.0));
}

#[test]
fn last_element() {
    // INDEX({10, 20, 30}, 3) → 30
    let arr = Value::Array(vec![
        Value::Number(10.0),
        Value::Number(20.0),
        Value::Number(30.0),
    ]);
    assert_eq!(index_fn(&[arr, Value::Number(3.0)]), Value::Number(30.0));
}

#[test]
fn scalar_at_index_one() {
    // INDEX(42, 1) → 42  (scalar is valid at position 1)
    assert_eq!(
        index_fn(&[Value::Number(42.0), Value::Number(1.0)]),
        Value::Number(42.0)
    );
}
