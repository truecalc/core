use proptest::prelude::*;
use truecalc_core::{evaluate, Value};
use std::collections::HashMap;

const CASES: u32 = 500;

fn run(formula: &str) -> Value {
    evaluate(formula, &HashMap::new())
}

fn run_vars(formula: &str, vars: Vec<(&str, f64)>) -> Value {
    let map = vars.into_iter().map(|(k, v)| (k.to_string(), Value::Number(v))).collect();
    evaluate(formula, &map)
}

fn small_positive_f64() -> impl Strategy<Value = f64> {
    1e-3f64..1e6f64
}

// 1. CONVERT identity: same unit returns the value unchanged
#[test]
fn convert_same_unit_identity() {
    proptest!(ProptestConfig::with_cases(CASES), |(x in small_positive_f64())| {
        let result = run_vars("=CONVERT(x, \"m\", \"m\")", vec![("x", x)]);
        if let Value::Number(v) = result {
            prop_assert!((v - x).abs() < 1e-9, "CONVERT(x, m→m) = {} ≠ {}", v, x);
        }
    });
    eprintln!("proptest: {CASES} cases (x ∈ [1e-3, 1e6])");
}

// 2. CONVERT m→km roundtrip: CONVERT(CONVERT(x, m, km), km, m) ≈ x
#[test]
fn convert_roundtrip_m_km() {
    proptest!(ProptestConfig::with_cases(CASES), |(x in small_positive_f64())| {
        let result = run_vars("=CONVERT(CONVERT(x, \"m\", \"km\"), \"km\", \"m\")", vec![("x", x)]);
        if let Value::Number(v) = result {
            prop_assert!((v - x).abs() / x.max(1.0) < 1e-9,
                "CONVERT m→km→m({}) = {} (delta {})", x, v, (v - x).abs());
        }
    });
    eprintln!("proptest: {CASES} cases (x ∈ [1e-3, 1e6])");
}

// 3. CONVERT kg→lbm roundtrip
#[test]
fn convert_roundtrip_kg_lbm() {
    proptest!(ProptestConfig::with_cases(CASES), |(x in small_positive_f64())| {
        let result = run_vars("=CONVERT(CONVERT(x, \"kg\", \"lbm\"), \"lbm\", \"kg\")", vec![("x", x)]);
        if let Value::Number(v) = result {
            prop_assert!((v - x).abs() / x.max(1.0) < 1e-9,
                "CONVERT kg→lbm→kg({}) = {} (delta {})", x, v, (v - x).abs());
        }
    });
    eprintln!("proptest: {CASES} cases (x ∈ [1e-3, 1e6])");
}

// 4. TO_TEXT of an integer followed by TO_PURE_NUMBER recovers the value
#[test]
fn to_text_then_pure_number_roundtrip() {
    proptest!(ProptestConfig::with_cases(CASES), |(n in 1i32..=100000i32)| {
        let result = run_vars("=TO_PURE_NUMBER(TO_TEXT(n))", vec![("n", n as f64)]);
        if let Value::Number(v) = result {
            prop_assert!((v - n as f64).abs() < 1e-9,
                "TO_PURE_NUMBER(TO_TEXT({})) = {}", n, v);
        }
    });
    eprintln!("proptest: {CASES} cases (n ∈ [1, 100000])");
}

// 5. CONVERT m→ft→in is the same as CONVERT m→in directly
#[test]
fn convert_chain_equals_direct() {
    proptest!(ProptestConfig::with_cases(CASES), |(x in small_positive_f64())| {
        let chained = run_vars("=CONVERT(CONVERT(x, \"m\", \"ft\"), \"ft\", \"in\")", vec![("x", x)]);
        let direct = run_vars("=CONVERT(x, \"m\", \"in\")", vec![("x", x)]);
        if let (Value::Number(c), Value::Number(d)) = (chained, direct) {
            prop_assert!((c - d).abs() / d.abs().max(1.0) < 1e-9,
                "chained={} direct={}", c, d);
        }
    });
    eprintln!("proptest: {CASES} cases (x ∈ [1e-3, 1e6])");
}

// 6. CONVERT is linear: CONVERT(2*x) = 2 * CONVERT(x) for unit conversions
#[test]
fn convert_is_linear() {
    proptest!(ProptestConfig::with_cases(CASES), |(x in 0.001f64..1000.0f64)| {
        let single = run_vars("=CONVERT(x, \"m\", \"ft\")", vec![("x", x)]);
        let double = run_vars("=CONVERT(2*x, \"m\", \"ft\")", vec![("x", x)]);
        if let (Value::Number(s), Value::Number(d)) = (single, double) {
            prop_assert!((d - 2.0 * s).abs() / s.abs().max(1.0) < 1e-9,
                "CONVERT(2x) = {} ≠ 2 * CONVERT(x) = {}", d, 2.0 * s);
        }
    });
    eprintln!("proptest: {CASES} cases (x ∈ [0.001, 1000])");
}

// Sanity checks
#[test]
fn sanity_convert_km() {
    // 1000 m = 1 km
    let result = run("=CONVERT(1000, \"m\", \"km\")");
    if let Value::Number(v) = result {
        assert!((v - 1.0).abs() < 1e-9, "1000m = {}km", v);
    }
}

#[test]
fn sanity_to_text() {
    assert_eq!(run("=TO_TEXT(42)"), Value::Text("42".to_string()));
}
