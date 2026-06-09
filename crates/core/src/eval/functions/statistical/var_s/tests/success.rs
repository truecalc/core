use super::super::*;
use crate::types::Value;

#[test]
fn var_s_basic() {
    // [2, 4, 6]: mean=4, sample var=4
    let result = var_s_fn(&[Value::Number(2.0), Value::Number(4.0), Value::Number(6.0)]);
    assert_eq!(result, Value::Number(4.0));
}

#[test]
fn var_s_two_values() {
    let result = var_s_fn(&[Value::Number(1.0), Value::Number(3.0)]);
    assert_eq!(result, Value::Number(2.0));
}

#[test]
fn var_s_bool_coerces_to_number() {
    // TRUE->1, FALSE->0; [2,6,1,0]: sample var=20.75/3
    let result = var_s_fn(&[
        Value::Number(2.0), Value::Number(6.0), Value::Bool(true), Value::Bool(false),
    ]);
    if let Value::Number(v) = result {
        assert!((v - 20.75_f64/3.0).abs() < 1e-10);
    } else { panic!("Expected Number, got {:?}", result); }
}
