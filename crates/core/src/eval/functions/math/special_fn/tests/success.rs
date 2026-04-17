use super::super::{erf_fn, erf_precise_fn, erfc_fn, erfc_precise_fn, gammaln_fn, gammaln_precise_fn};
use crate::types::Value;

fn approx_eq(a: f64, b: f64, tol: f64) -> bool {
    (a - b).abs() < tol
}

fn num(v: &Value) -> f64 {
    match v {
        Value::Number(n) => *n,
        other => panic!("expected Number, got {:?}", other),
    }
}

// ── ERF ──────────────────────────────────────────────────────────────────────

#[test]
fn erf_zero() {
    // A&S approximation has tiny error near 0; allow 1e-6 tolerance
    let result = num(&erf_fn(&[Value::Number(0.0)]));
    assert!(approx_eq(result, 0.0, 1e-6), "erf(0) = {}", result);
}

#[test]
fn erf_one() {
    let result = num(&erf_fn(&[Value::Number(1.0)]));
    assert!(approx_eq(result, 0.8427, 1e-4), "erf(1) = {}", result);
}

#[test]
fn erf_negative() {
    // ERF(-1) = -ERF(1)
    let result = num(&erf_fn(&[Value::Number(-1.0)]));
    assert!(approx_eq(result, -0.8427, 1e-4), "erf(-1) = {}", result);
}

#[test]
fn erf_two_args() {
    // ERF(0, 1) = erf(1) - erf(0) ≈ 0.8427
    let result = num(&erf_fn(&[Value::Number(0.0), Value::Number(1.0)]));
    assert!(approx_eq(result, 0.8427, 1e-4), "erf(0,1) = {}", result);
}

#[test]
fn erf_precise_one() {
    let result = num(&erf_precise_fn(&[Value::Number(1.0)]));
    assert!(approx_eq(result, 0.8427, 1e-4), "erf_precise(1) = {}", result);
}

// ── ERFC ─────────────────────────────────────────────────────────────────────

#[test]
fn erfc_zero() {
    // A&S approximation has tiny error; allow 1e-6 tolerance
    let result = num(&erfc_fn(&[Value::Number(0.0)]));
    assert!(approx_eq(result, 1.0, 1e-6), "erfc(0) = {}", result);
}

#[test]
fn erfc_one() {
    let result = num(&erfc_fn(&[Value::Number(1.0)]));
    assert!(approx_eq(result, 0.1573, 1e-4), "erfc(1) = {}", result);
}

#[test]
fn erfc_precise_one() {
    let result = num(&erfc_precise_fn(&[Value::Number(1.0)]));
    assert!(approx_eq(result, 0.1573, 1e-4), "erfc_precise(1) = {}", result);
}

// ── GAMMALN ───────────────────────────────────────────────────────────────────

#[test]
fn gammaln_one() {
    // GAMMALN(1) = ln(Gamma(1)) = ln(1) = 0; Lanczos has tiny error
    let result = num(&gammaln_fn(&[Value::Number(1.0)]));
    assert!(approx_eq(result, 0.0, 1e-10), "gammaln(1) = {}", result);
}

#[test]
fn gammaln_half() {
    // GAMMALN(0.5) = ln(sqrt(pi)) ≈ 0.5724
    let result = num(&gammaln_fn(&[Value::Number(0.5)]));
    assert!(approx_eq(result, 0.5724, 1e-4), "gammaln(0.5) = {}", result);
}

#[test]
fn gammaln_precise_one() {
    let result = num(&gammaln_precise_fn(&[Value::Number(1.0)]));
    assert!(approx_eq(result, 0.0, 1e-10), "gammaln_precise(1) = {}", result);
}
