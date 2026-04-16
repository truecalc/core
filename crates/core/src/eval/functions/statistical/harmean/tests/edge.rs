use super::super::*;
use crate::types::Value;

#[test]
fn harmean_value_of_one() {
    // HARMEAN(1, 1, 1) = 1
    let result = harmean_fn(&[Value::Number(1.0), Value::Number(1.0), Value::Number(1.0)]);
    if let Value::Number(n) = result {
        assert!((n - 1.0).abs() < 1e-10, "got {n}");
    } else {
        panic!("expected Number, got {:?}", result);
    }
}

#[test]
fn harmean_array_arg() {
    // HARMEAN([1, 4]) = 2 / (1 + 0.25) = 1.6
    let arr = Value::Array(vec![Value::Number(1.0), Value::Number(4.0)]);
    let result = harmean_fn(&[arr]);
    if let Value::Number(n) = result {
        assert!((n - 1.6).abs() < 1e-10, "got {n}");
    } else {
        panic!("expected Number, got {:?}", result);
    }
}

#[test]
fn harmean_ignores_non_numeric() {
    // Text and Bool args are ignored; only the Number counts
    let result = harmean_fn(&[
        Value::Number(4.0),
        Value::Text("ignored".to_string()),
        Value::Bool(true),
    ]);
    assert_eq!(result, Value::Number(4.0));
}
