use super::super::*;
use crate::types::Value;

#[test]
fn stdeva_basic_numbers() {
    // [2, 4, 6]: sample var=4, stdev=2.0
    let result = stdeva_fn(&[Value::Number(2.0), Value::Number(4.0), Value::Number(6.0)]);
    assert_eq!(result, Value::Number(2.0));
}

#[test]
fn stdeva_two_values() {
    // [1, 3]: sample var=2, stdev=sqrt(2)
    let result = stdeva_fn(&[Value::Number(1.0), Value::Number(3.0)]);
    if let Value::Number(v) = result {
        assert!((v - 2.0_f64.sqrt()).abs() < 1e-10);
    } else {
        panic!("Expected Number, got {:?}", result);
    }
}

#[test]
fn stdeva_numbers_only() {
    // Same as STDEV.S when only numbers present
    let result = stdeva_fn(&[Value::Number(0.0), Value::Number(10.0)]);
    if let Value::Number(v) = result {
        assert!((v - 50.0_f64.sqrt()).abs() < 1e-10);
    } else {
        panic!("Expected Number, got {:?}", result);
    }
}
