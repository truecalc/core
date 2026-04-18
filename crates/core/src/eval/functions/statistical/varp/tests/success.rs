use super::super::*;
use crate::types::Value;

#[test]
fn varp_matches_var_p_result() {
    // VARP delegates to VAR.P (population variance)
    // [2, 4, 6]: pop var=8/3
    let result = varp_fn(&[Value::Number(2.0), Value::Number(4.0), Value::Number(6.0)]);
    if let Value::Number(v) = result {
        assert!((v - 8.0 / 3.0).abs() < 1e-10);
    } else {
        panic!("Expected Number, got {:?}", result);
    }
}

#[test]
fn varp_single_value_returns_zero() {
    // Population variance of one value is 0
    assert_eq!(varp_fn(&[Value::Number(5.0)]), Value::Number(0.0));
}

#[test]
fn varp_two_values() {
    // [1, 3]: pop var=1
    let result = varp_fn(&[Value::Number(1.0), Value::Number(3.0)]);
    assert_eq!(result, Value::Number(1.0));
}
