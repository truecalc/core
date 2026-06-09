use super::super::*;
use crate::types::Value;

#[test]
fn stdev_s_basic() {
    let result = stdev_s_fn(&[Value::Number(2.0), Value::Number(4.0), Value::Number(6.0)]);
    assert_eq!(result, Value::Number(2.0));
}

#[test]
fn stdev_s_two_values() {
    let result = stdev_s_fn(&[Value::Number(1.0), Value::Number(3.0)]);
    if let Value::Number(v) = result {
        assert!((v - 2.0_f64.sqrt()).abs() < 1e-10);
    } else { panic!("Expected Number, got {:?}", result); }
}

#[test]
fn stdev_s_bool_coerces_to_number() {
    // TRUE->1, FALSE->0; [2,6,1,0]: sample var=20.75/3
    let result = stdev_s_fn(&[
        Value::Number(2.0), Value::Number(6.0), Value::Bool(true), Value::Bool(false),
    ]);
    if let Value::Number(v) = result {
        assert!((v - (20.75_f64/3.0).sqrt()).abs() < 1e-10);
    } else { panic!("Expected Number, got {:?}", result); }
}

#[test]
fn stdev_s_known_dataset() {
    let result = stdev_s_fn(&[
        Value::Number(2.0), Value::Number(4.0), Value::Number(4.0), Value::Number(4.0),
        Value::Number(5.0), Value::Number(5.0), Value::Number(7.0), Value::Number(9.0),
    ]);
    if let Value::Number(v) = result {
        assert!((v - (32.0_f64/7.0).sqrt()).abs() < 1e-10);
    } else { panic!("Expected Number, got {:?}", result); }
}
