use super::super::seriessum_fn;
use crate::types::Value;

fn arr(ns: &[f64]) -> Value {
    Value::Array(ns.iter().map(|&n| Value::Number(n)).collect())
}

#[test]
fn single_coeff_identity() {
    // SERIESSUM(1, 0, 1, {1}) = 1^0 * 1 = 1
    assert_eq!(
        seriessum_fn(&[Value::Number(1.0), Value::Number(0.0), Value::Number(1.0), arr(&[1.0])]),
        Value::Number(1.0)
    );
}

#[test]
fn two_coeffs() {
    // SERIESSUM(2, 1, 1, {1, 1}) = 1*2^1 + 1*2^2 = 2 + 4 = 6
    assert_eq!(
        seriessum_fn(&[Value::Number(2.0), Value::Number(1.0), Value::Number(1.0), arr(&[1.0, 1.0])]),
        Value::Number(6.0)
    );
}

#[test]
fn step_zero() {
    // SERIESSUM(3, 0, 0, {5}) = 5 * 3^0 = 5
    assert_eq!(
        seriessum_fn(&[Value::Number(3.0), Value::Number(0.0), Value::Number(0.0), arr(&[5.0])]),
        Value::Number(5.0)
    );
}

#[test]
fn x_zero() {
    // SERIESSUM(0, 1, 1, {5, 3}) = 5*0^1 + 3*0^2 = 0
    assert_eq!(
        seriessum_fn(&[Value::Number(0.0), Value::Number(1.0), Value::Number(1.0), arr(&[5.0, 3.0])]),
        Value::Number(0.0)
    );
}

#[test]
fn step_two() {
    // SERIESSUM(2, 0, 2, {1, 1}) = 1*2^0 + 1*2^2 = 1 + 4 = 5
    assert_eq!(
        seriessum_fn(&[Value::Number(2.0), Value::Number(0.0), Value::Number(2.0), arr(&[1.0, 1.0])]),
        Value::Number(5.0)
    );
}
