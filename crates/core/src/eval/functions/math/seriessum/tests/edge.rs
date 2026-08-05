use super::super::seriessum_fn;
use crate::types::Value;

fn arr(ns: &[f64]) -> Value {
    Value::Array(ns.iter().map(|&n| Value::Number(n)).collect())
}

#[test]
fn three_coeffs_all_ones() {
    // SERIESSUM(1, 1, 1, {1,1,1}) = 1*1^1 + 1*1^2 + 1*1^3 = 3
    assert_eq!(
        seriessum_fn(&[Value::Number(1.0), Value::Number(1.0), Value::Number(1.0), arr(&[1.0, 1.0, 1.0])]),
        Value::Number(3.0)
    );
}

#[test]
fn scalar_coeff() {
    // SERIESSUM(10, 0, 1, 1) = 1 * 10^0 = 1 (scalar treated as single element)
    assert_eq!(
        seriessum_fn(&[Value::Number(10.0), Value::Number(0.0), Value::Number(1.0), Value::Number(1.0)]),
        Value::Number(1.0)
    );
}

#[test]
fn approx_series() {
    // SERIESSUM(1, 0, 1, {1, 1, 0.5}) = 1 + 1 + 0.5 = 2.5
    assert_eq!(
        seriessum_fn(&[
            Value::Number(1.0),
            Value::Number(0.0),
            Value::Number(1.0),
            arr(&[1.0, 1.0, 0.5])
        ]),
        Value::Number(2.5)
    );
}
