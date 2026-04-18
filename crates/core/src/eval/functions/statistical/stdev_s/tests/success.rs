use super::super::*;
use crate::types::Value;

#[test]
fn stdev_s_basic() {
    // [2, 4, 6]: sample var=4, stdev=2.0
    let result = stdev_s_fn(&[Value::Number(2.0), Value::Number(4.0), Value::Number(6.0)]);
    assert_eq!(result, Value::Number(2.0));
}

#[test]
fn stdev_s_two_values() {
    // [1, 3]: sample var=2, stdev=sqrt(2)
    let result = stdev_s_fn(&[Value::Number(1.0), Value::Number(3.0)]);
    if let Value::Number(v) = result {
        assert!((v - 2.0_f64.sqrt()).abs() < 1e-10);
    } else {
        panic!("Expected Number, got {:?}", result);
    }
}

#[test]
fn stdev_s_ignores_bool_and_text() {
    // Only numbers counted: [2, 6] → mean=4, sample var=8, stdev=sqrt(8)
    let result = stdev_s_fn(&[
        Value::Number(2.0),
        Value::Number(6.0),
        Value::Bool(true),
        Value::Text("x".to_string()),
    ]);
    if let Value::Number(v) = result {
        assert!((v - 8.0_f64.sqrt()).abs() < 1e-10);
    } else {
        panic!("Expected Number, got {:?}", result);
    }
}

#[test]
fn stdev_s_known_dataset() {
    // [2,4,4,4,5,5,7,9]: sample var≈4.5714, stdev≈2.1381
    let result = stdev_s_fn(&[
        Value::Number(2.0),
        Value::Number(4.0),
        Value::Number(4.0),
        Value::Number(4.0),
        Value::Number(5.0),
        Value::Number(5.0),
        Value::Number(7.0),
        Value::Number(9.0),
    ]);
    if let Value::Number(v) = result {
        assert!((v - (32.0_f64 / 7.0).sqrt()).abs() < 1e-10);
    } else {
        panic!("Expected Number, got {:?}", result);
    }
}
