use super::super::*;
use crate::types::Value;

#[test]
fn stdev_all_same_values_returns_zero() {
    assert_eq!(
        stdev_fn(&[Value::Number(4.0), Value::Number(4.0), Value::Number(4.0)]),
        Value::Number(0.0)
    );
}

#[test]
fn stdev_ignores_bool_and_text() {
    // Text/bool ignored; [2, 6] → mean=4, sample var=8, stdev=sqrt(8)
    let result = stdev_fn(&[
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
