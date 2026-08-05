use super::super::{erfc_fn, erf_fn};
use crate::types::Value;

fn approx_eq(a: f64, b: f64, tol: f64) -> bool {
    (a - b).abs() < tol
}

fn unwrap_number(v: Value) -> f64 {
    match v {
        Value::Number(n) => n,
        other => panic!("expected Number, got {:?}", other),
    }
}

// ---------------------------------------------------------------------------
// ERF
// ---------------------------------------------------------------------------

#[test]
fn erf_zero() {
    // ERF(0) ≈ 0 (within 1e-6)
    let result = unwrap_number(erf_fn(&[Value::Number(0.0)]));
    assert!(
        approx_eq(result, 0.0, 1e-6),
        "ERF(0) = {result}, expected ≈ 0"
    );
}

#[test]
fn erf_one() {
    // ERF(1) ≈ 0.8427 (within 1e-4)
    let result = unwrap_number(erf_fn(&[Value::Number(1.0)]));
    assert!(
        approx_eq(result, 0.8427007929497149, 1e-4),
        "ERF(1) = {result}, expected ≈ 0.8427"
    );
}

#[test]
fn erf_two_args_range() {
    // ERF(0, 1) = ERF(1) - ERF(0) ≈ 0.8427
    let result = unwrap_number(erf_fn(&[Value::Number(0.0), Value::Number(1.0)]));
    assert!(
        approx_eq(result, 0.8427007929497149, 1e-4),
        "ERF(0,1) = {result}, expected ≈ 0.8427"
    );
}

// ---------------------------------------------------------------------------
// ERFC
// ---------------------------------------------------------------------------

#[test]
fn erfc_zero() {
    // ERFC(0) = 1 - ERF(0) ≈ 1 (within 1e-6)
    let result = unwrap_number(erfc_fn(&[Value::Number(0.0)]));
    assert!(
        approx_eq(result, 1.0, 1e-6),
        "ERFC(0) = {result}, expected ≈ 1"
    );
}

#[test]
fn erfc_one() {
    // ERFC(1) ≈ 1 - 0.8427 ≈ 0.1573
    let result = unwrap_number(erfc_fn(&[Value::Number(1.0)]));
    assert!(
        approx_eq(result, 0.15729920705028513, 1e-4),
        "ERFC(1) = {result}, expected ≈ 0.1573"
    );
}
