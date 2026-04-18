use super::super::*;
use crate::types::Value;

#[test]
fn stdevpa_basic_numbers() {
    // [2, 4, 6]: pop var=8/3, stdev=sqrt(8/3)
    let result = stdevpa_fn(&[Value::Number(2.0), Value::Number(4.0), Value::Number(6.0)]);
    if let Value::Number(v) = result {
        assert!((v - (8.0_f64 / 3.0).sqrt()).abs() < 1e-10);
    } else {
        panic!("Expected Number, got {:?}", result);
    }
}

#[test]
fn stdevpa_single_value_returns_zero() {
    // Pop stdev of one value is 0
    assert_eq!(stdevpa_fn(&[Value::Number(7.0)]), Value::Number(0.0));
}

#[test]
fn stdevpa_two_values() {
    // [1, 3]: pop var=1, stdev=1.0
    let result = stdevpa_fn(&[Value::Number(1.0), Value::Number(3.0)]);
    assert_eq!(result, Value::Number(1.0));
}
