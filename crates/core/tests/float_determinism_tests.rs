/// Float determinism tests (ADR 0001).
///
/// Each test verifies that the engine produces a result that is bit-identical to the
/// libm reference value, confirming the migration from native f64 methods to libm.
use std::collections::HashMap;
use truecalc_core::{Engine, Value};

fn sheets() -> Engine {
    Engine::sheets()
}

fn eval_f64(engine: &Engine, formula: &str) -> f64 {
    match engine.evaluate(formula, &HashMap::new()) {
        Value::Number(n) => n,
        other => panic!("expected Number, got {:?} for formula: {}", other, formula),
    }
}

// ── SIN ───────────────────────────────────────────────────────────────────────

#[test]
fn sin_bit_identical_to_libm() {
    let engine = sheets();
    let got = eval_f64(&engine, "=SIN(1)");
    assert_eq!(got.to_bits(), libm::sin(1.0_f64).to_bits());
}

#[test]
fn sin_pi_over_6_bit_identical() {
    let engine = sheets();
    let got = eval_f64(&engine, "=SIN(PI()/6)");
    let expected = libm::sin(std::f64::consts::PI / 6.0);
    assert_eq!(got.to_bits(), expected.to_bits());
}

// ── COS ───────────────────────────────────────────────────────────────────────

#[test]
fn cos_bit_identical_to_libm() {
    let engine = sheets();
    let got = eval_f64(&engine, "=COS(1)");
    assert_eq!(got.to_bits(), libm::cos(1.0_f64).to_bits());
}

// ── TAN ───────────────────────────────────────────────────────────────────────

#[test]
fn tan_bit_identical_to_libm() {
    let engine = sheets();
    let got = eval_f64(&engine, "=TAN(1)");
    assert_eq!(got.to_bits(), libm::tan(1.0_f64).to_bits());
}

// ── SQRT ─────────────────────────────────────────────────────────────────────

#[test]
fn sqrt_bit_identical_to_libm() {
    let engine = sheets();
    let got = eval_f64(&engine, "=SQRT(2)");
    assert_eq!(got.to_bits(), libm::sqrt(2.0_f64).to_bits());
}

#[test]
fn sqrt_9_exact() {
    let engine = sheets();
    let got = eval_f64(&engine, "=SQRT(9)");
    assert_eq!(got.to_bits(), libm::sqrt(9.0_f64).to_bits());
}

// ── EXP ──────────────────────────────────────────────────────────────────────

#[test]
fn exp_bit_identical_to_libm() {
    let engine = sheets();
    let got = eval_f64(&engine, "=EXP(1)");
    assert_eq!(got.to_bits(), libm::exp(1.0_f64).to_bits());
}

#[test]
fn exp_2_bit_identical() {
    let engine = sheets();
    let got = eval_f64(&engine, "=EXP(2)");
    assert_eq!(got.to_bits(), libm::exp(2.0_f64).to_bits());
}

// ── LN ───────────────────────────────────────────────────────────────────────

#[test]
fn ln_bit_identical_to_libm() {
    let engine = sheets();
    let got = eval_f64(&engine, "=LN(10)");
    assert_eq!(got.to_bits(), libm::log(10.0_f64).to_bits());
}

// ── LOG / LOG10 ───────────────────────────────────────────────────────────────

#[test]
fn log10_bit_identical_to_libm() {
    let engine = sheets();
    let got = eval_f64(&engine, "=LOG10(100)");
    assert_eq!(got.to_bits(), libm::log10(100.0_f64).to_bits());
}

#[test]
fn log_base2_bit_identical() {
    let engine = sheets();
    let got = eval_f64(&engine, "=LOG(8,2)");
    // LOG(8,2) = ln(8)/ln(2)
    let expected = libm::log(8.0_f64) / libm::log(2.0_f64);
    assert_eq!(got.to_bits(), expected.to_bits());
}

// ── POWER ────────────────────────────────────────────────────────────────────

#[test]
fn power_bit_identical_to_libm() {
    let engine = sheets();
    let got = eval_f64(&engine, "=POWER(2,0.5)");
    assert_eq!(got.to_bits(), libm::pow(2.0_f64, 0.5_f64).to_bits());
}

#[test]
fn power_caret_operator_bit_identical() {
    let engine = sheets();
    let got = eval_f64(&engine, "=2^0.5");
    assert_eq!(got.to_bits(), libm::pow(2.0_f64, 0.5_f64).to_bits());
}
