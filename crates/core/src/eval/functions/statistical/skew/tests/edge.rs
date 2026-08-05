use super::super::*;
use crate::types::Value;

#[test]
fn skew_exactly_three_values() {
    // SKEW(1, 2, 4) — minimum valid n; asymmetric, should return a finite number
    let result = skew_fn(&[
        Value::Number(1.0),
        Value::Number(2.0),
        Value::Number(4.0),
    ]);
    if let Value::Number(n) = result {
        assert!(n.is_finite(), "got {n}");
    } else {
        panic!("expected Number, got {:?}", result);
    }
}

#[test]
fn skew_symmetric_data_near_zero() {
    // SKEW(1, 2, 3) — perfectly symmetric; skewness should be ~0
    let result = skew_fn(&[
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
fn skew_array_arg() {
    // SKEW via Array argument
    let arr = Value::Array(vec![
        Value::Number(3.0),
        Value::Number(4.0),
        Value::Number(5.0),
        Value::Number(2.0),
        Value::Number(3.0),
        Value::Number(4.0),
        Value::Number(5.0),
        Value::Number(6.0),
        Value::Number(4.0),
        Value::Number(7.0),
    ]);
    let result = skew_fn(&[arr]);
    if let Value::Number(n) = result {
        assert!((n - 0.3595).abs() < 1e-3, "got {n}");
    } else {
        panic!("expected Number, got {:?}", result);
    }
}
