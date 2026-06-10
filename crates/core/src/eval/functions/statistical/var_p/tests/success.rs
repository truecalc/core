use super::super::*;
use crate::types::Value;

#[test]
fn var_p_basic() {
    // [2, 4, 6]: pop var=8/3
    let result = var_p_fn(&[Value::Number(2.0), Value::Number(4.0), Value::Number(6.0)]);
    if let Value::Number(v) = result {
        assert!((v - 8.0/3.0).abs() < 1e-10);
    } else { panic!("Expected Number, got {:?}", result); }
}

#[test]
fn var_p_two_values() {
    let result = var_p_fn(&[Value::Number(1.0), Value::Number(3.0)]);
    assert_eq!(result, Value::Number(1.0));
}

#[test]
fn var_p_single_value_returns_zero() {
    assert_eq!(var_p_fn(&[Value::Number(5.0)]), Value::Number(0.0));
}

#[test]
fn var_p_bool_coerces_to_number() {
    // TRUE->1, FALSE->0; [2,6,1,0]: pop var=20.75/4
    let result = var_p_fn(&[
        Value::Number(2.0), Value::Number(6.0), Value::Bool(true), Value::Bool(false),
    ]);
    if let Value::Number(v) = result {
        assert!((v - 20.75_f64/4.0).abs() < 1e-10);
    } else { panic!("Expected Number, got {:?}", result); }
}
