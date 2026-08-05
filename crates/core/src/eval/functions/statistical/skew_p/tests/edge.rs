use super::super::*;
use crate::types::{ErrorKind, Value};

#[test]
fn skew_p_single_value_returns_div0() {
    // σ = 0 for a single value → #DIV/0!
    assert_eq!(
        skew_p_fn(&[Value::Number(5.0)]),
        Value::Error(ErrorKind::DivByZero)
    );
}

#[test]
fn skew_p_symmetric_data_near_zero() {
    // SKEW.P(1, 2, 3) — symmetric; population skewness should be ~0
    let result = skew_p_fn(&[
        Value::Number(1.0),
        Value::Number(2.0),
        Value::Number(3.0),
    ]);
    if let Value::Number(n) = result {
        assert!(n.abs() < 1e-10, "expected ~0 skewness, got {n}");
    } else {
        panic!("expected Number, got {:?}", result);
    }
}

#[test]
fn skew_p_array_arg() {
    // SKEW.P via Array argument
    let arr = Value::Array(vec![
        Value::Number(3.0),
        Value::Number(4.0),
        Value::Number(5.0),
    ]);
    let result = skew_p_fn(&[arr]);
    if let Value::Number(n) = result {
        assert!(n.is_finite(), "got {n}");
    } else {
        panic!("expected Number, got {:?}", result);
    }
}
