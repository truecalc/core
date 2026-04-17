use super::super::{
    complex_fn, imabs_fn, imaginary_fn, imargument_fn, imconjugate_fn, imdiv_fn, imexp_fn,
    imln_fn, impower_fn, improduct_fn, imreal_fn, imsqrt_fn, imsub_fn, imsum_fn,
};
use crate::types::Value;

fn approx_eq(a: f64, b: f64, tol: f64) -> bool {
    (a - b).abs() < tol
}

fn text(v: &Value) -> &str {
    match v {
        Value::Text(s) => s.as_str(),
        other => panic!("expected Text, got {:?}", other),
    }
}

fn num(v: &Value) -> f64 {
    match v {
        Value::Number(n) => *n,
        other => panic!("expected Number, got {:?}", other),
    }
}

fn t(s: &str) -> Value {
    Value::Text(s.to_string())
}

// ── COMPLEX ───────────────────────────────────────────────────────────────────

#[test]
fn complex_3_4i() {
    assert_eq!(
        complex_fn(&[Value::Number(3.0), Value::Number(4.0)]),
        Value::Text("3+4i".to_string())
    );
}

#[test]
fn complex_3_neg4i() {
    assert_eq!(
        complex_fn(&[Value::Number(3.0), Value::Number(-4.0)]),
        Value::Text("3-4i".to_string())
    );
}

#[test]
fn complex_0_1() {
    // COMPLEX(0, 1) → "i"
    assert_eq!(
        complex_fn(&[Value::Number(0.0), Value::Number(1.0)]),
        Value::Text("i".to_string())
    );
}

#[test]
fn complex_with_j_suffix() {
    assert_eq!(
        complex_fn(&[Value::Number(3.0), Value::Number(4.0), t("j")]),
        Value::Text("3+4j".to_string())
    );
}

// ── IMABS ─────────────────────────────────────────────────────────────────────

#[test]
fn imabs_3_4i() {
    // |3+4i| = 5
    assert_eq!(imabs_fn(&[t("3+4i")]), Value::Number(5.0));
}

#[test]
fn imabs_pure_real() {
    assert_eq!(imabs_fn(&[Value::Number(5.0)]), Value::Number(5.0));
}

// ── IMREAL / IMAGINARY ────────────────────────────────────────────────────────

#[test]
fn imreal_3_4i() {
    assert_eq!(imreal_fn(&[t("3+4i")]), Value::Number(3.0));
}

#[test]
fn imaginary_3_4i() {
    assert_eq!(imaginary_fn(&[t("3+4i")]), Value::Number(4.0));
}

#[test]
fn imreal_pure_real() {
    assert_eq!(imreal_fn(&[Value::Number(5.0)]), Value::Number(5.0));
}

#[test]
fn imaginary_pure_real() {
    assert_eq!(imaginary_fn(&[Value::Number(5.0)]), Value::Number(0.0));
}

// ── IMARGUMENT ────────────────────────────────────────────────────────────────

#[test]
fn imargument_1_i() {
    // arg(1+i) = pi/4
    let result = num(&imargument_fn(&[t("1+i")]));
    assert!(
        approx_eq(result, std::f64::consts::FRAC_PI_4, 1e-10),
        "imargument(1+i) = {}",
        result
    );
}

// ── IMCONJUGATE ───────────────────────────────────────────────────────────────

#[test]
fn imconjugate_3_4i() {
    assert_eq!(
        imconjugate_fn(&[t("3+4i")]),
        Value::Text("3-4i".to_string())
    );
}

// ── IMDIV ────────────────────────────────────────────────────────────────────

#[test]
fn imdiv_basic() {
    // (4+2i)/(1+i) = (4+2i)(1-i)/2 = (4-4i+2i-2i^2)/2 = (4+2+(-4+2)i)/2 = (6-2i)/2 = 3-i
    let result = imdiv_fn(&[t("4+2i"), t("1+i")]);
    assert_eq!(result, Value::Text("3-i".to_string()));
}

// ── IMEXP ────────────────────────────────────────────────────────────────────

#[test]
fn imexp_zero() {
    // exp(0) = 1
    assert_eq!(imexp_fn(&[Value::Number(0.0)]), Value::Number(1.0));
}

// ── IMLN ─────────────────────────────────────────────────────────────────────

#[test]
fn imln_one() {
    // ln(1) = 0
    assert_eq!(imln_fn(&[Value::Number(1.0)]), Value::Number(0.0));
}

// ── IMPOWER ──────────────────────────────────────────────────────────────────

#[test]
fn impower_square() {
    // (1+i)^2 = 1 + 2i - 1 = 2i
    let result = impower_fn(&[t("1+i"), Value::Number(2.0)]);
    assert_eq!(result, Value::Text("2i".to_string()));
}

// ── IMPRODUCT ─────────────────────────────────────────────────────────────────

#[test]
fn improduct_1_2i_times_3_4i() {
    // (1+2i)(3+4i) = 3+4i+6i+8i^2 = 3+10i-8 = -5+10i
    let result = improduct_fn(&[t("1+2i"), t("3+4i")]);
    assert_eq!(result, Value::Text("-5+10i".to_string()));
}

// ── IMSQRT ───────────────────────────────────────────────────────────────────

#[test]
fn imsqrt_neg1() {
    // sqrt(-1) = i
    let result = imsqrt_fn(&[Value::Number(-1.0)]);
    assert_eq!(result, Value::Text("i".to_string()));
}

// ── IMSUB ────────────────────────────────────────────────────────────────────

#[test]
fn imsub_basic() {
    // (3+4i) - (1+2i) = 2+2i
    let result = imsub_fn(&[t("3+4i"), t("1+2i")]);
    assert_eq!(result, Value::Text("2+2i".to_string()));
}

// ── IMSUM ────────────────────────────────────────────────────────────────────

#[test]
fn imsum_basic() {
    // (1+2i) + (3+4i) = 4+6i
    let result = imsum_fn(&[t("1+2i"), t("3+4i")]);
    assert_eq!(result, Value::Text("4+6i".to_string()));
}

#[test]
fn imsum_single() {
    let result = imsum_fn(&[t("3+4i")]);
    assert_eq!(result, Value::Text("3+4i".to_string()));
}

// ── String formatting edge cases ──────────────────────────────────────────────

#[test]
fn text_function_returns_text() {
    let result = complex_fn(&[Value::Number(3.0), Value::Number(4.0)]);
    assert!(matches!(result, Value::Text(_)));
    assert_eq!(text(&result), "3+4i");
}
