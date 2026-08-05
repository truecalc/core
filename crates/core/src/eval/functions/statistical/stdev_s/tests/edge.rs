use super::super::*;
use crate::types::Value;

#[test]
fn stdev_s_all_same_values_returns_zero() {
    assert_eq!(
        stdev_s_fn(&[Value::Number(4.0), Value::Number(4.0), Value::Number(4.0)]),
        Value::Number(0.0)
    );
}

#[test]
fn stdev_s_negative_numbers() {
    // [-3, -1]: sample var=2, stdev=sqrt(2)
    let result = stdev_s_fn(&[Value::Number(-3.0), Value::Number(-1.0)]);
    if let Value::Number(v) = result {
        assert!((v - 2.0_f64.sqrt()).abs() < 1e-10);
    } else {
        panic!("Expected Number, got {:?}", result);
    }
}

#[test]
fn stdev_s_large_spread() {
    // [0, 10]: sample var=50, stdev=sqrt(50)
    let result = stdev_s_fn(&[Value::Number(0.0), Value::Number(10.0)]);
    if let Value::Number(v) = result {
        assert!((v - 50.0_f64.sqrt()).abs() < 1e-10);
    } else {
        panic!("Expected Number, got {:?}", result);
    }
}
