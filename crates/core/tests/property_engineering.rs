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

fn small_f64() -> impl Strategy<Value = f64> {
    -1e6f64..1e6f64
}

// 1. DELTA(x, x) = 1 for all x (reflexive equality)
#[test]
fn delta_reflexive() {
    proptest!(ProptestConfig::with_cases(CASES), |(x in small_f64())| {
        let result = run_vars("=DELTA(x, x)", vec![("x", x)]);
        prop_assert_eq!(result, Value::Number(1.0));
    });
    eprintln!("proptest: {CASES} cases (x ∈ [-1e6, 1e6])");
}

// 2. GESTEP(x, x) = 1 for all x (x >= x is always true)
#[test]
fn gestep_reflexive() {
    proptest!(ProptestConfig::with_cases(CASES), |(x in small_f64())| {
        let result = run_vars("=GESTEP(x, x)", vec![("x", x)]);
        prop_assert_eq!(result, Value::Number(1.0));
    });
    eprintln!("proptest: {CASES} cases (x ∈ [-1e6, 1e6])");
}

// 3. BITAND(n, n) = n (idempotent)
#[test]
fn bitand_idempotent() {
    proptest!(ProptestConfig::with_cases(CASES), |(n in 0u32..=65535u32)| {
        let result = run_vars("=BITAND(n, n)", vec![("n", n as f64)]);
        prop_assert_eq!(result, Value::Number(n as f64));
    });
    eprintln!("proptest: {CASES} cases (n ∈ [0, 65535])");
}

// 4. BITOR(n, n) = n (idempotent)
#[test]
fn bitor_idempotent() {
    proptest!(ProptestConfig::with_cases(CASES), |(n in 0u32..=65535u32)| {
        let result = run_vars("=BITOR(n, n)", vec![("n", n as f64)]);
        prop_assert_eq!(result, Value::Number(n as f64));
    });
    eprintln!("proptest: {CASES} cases (n ∈ [0, 65535])");
}

// 5. BITXOR(n, n) = 0 (self-inverse)
#[test]
fn bitxor_self_inverse() {
    proptest!(ProptestConfig::with_cases(CASES), |(n in 0u32..=65535u32)| {
        let result = run_vars("=BITXOR(n, n)", vec![("n", n as f64)]);
        prop_assert_eq!(result, Value::Number(0.0));
    });
    eprintln!("proptest: {CASES} cases (n ∈ [0, 65535])");
}

// 6. BIN2DEC(DEC2BIN(n)) roundtrip for n ∈ [0, 255]
#[test]
fn bin2dec_dec2bin_roundtrip() {
    proptest!(ProptestConfig::with_cases(CASES), |(n in 0u32..=255u32)| {
        let result = run_vars("=BIN2DEC(DEC2BIN(n))", vec![("n", n as f64)]);
        prop_assert_eq!(result, Value::Number(n as f64));
    });
    eprintln!("proptest: {CASES} cases (n ∈ [0, 255])");
}

// 7. HEX2DEC(DEC2HEX(n)) roundtrip for n ∈ [0, 65535]
#[test]
fn hex2dec_dec2hex_roundtrip() {
    proptest!(ProptestConfig::with_cases(CASES), |(n in 0u32..=65535u32)| {
        let result = run_vars("=HEX2DEC(DEC2HEX(n))", vec![("n", n as f64)]);
        prop_assert_eq!(result, Value::Number(n as f64));
    });
    eprintln!("proptest: {CASES} cases (n ∈ [0, 65535])");
}

// 8. IMABS(COMPLEX(a, b)) = SQRT(a^2 + b^2)
#[test]
fn imabs_equals_euclidean_norm() {
    proptest!(ProptestConfig::with_cases(CASES), |(a in -1000.0f64..1000.0f64, b in -1000.0f64..1000.0f64)| {
        let imabs = run_vars("=IMABS(COMPLEX(a, b))", vec![("a", a), ("b", b)]);
        let norm = run_vars("=SQRT(a*a + b*b)", vec![("a", a), ("b", b)]);
        if let (Value::Number(im), Value::Number(eu)) = (imabs, norm) {
            prop_assert!((im - eu).abs() < 1e-9,
                "IMABS({},{}) = {} ≠ SQRT(a²+b²) = {}", a, b, im, eu);
        }
    });
    eprintln!("proptest: {CASES} cases (a,b ∈ [-1000, 1000])");
}

// 9. GESTEP(x, 0) = 1 iff x >= 0
#[test]
fn gestep_zero_matches_sign() {
    proptest!(ProptestConfig::with_cases(CASES), |(x in small_f64())| {
        let result = run_vars("=GESTEP(x, 0)", vec![("x", x)]);
        let expected = Value::Number(if x >= 0.0 { 1.0 } else { 0.0 });
        prop_assert_eq!(result, expected);
    });
    eprintln!("proptest: {CASES} cases (x ∈ [-1e6, 1e6])");
}

// Sanity checks
#[test]
fn sanity_delta() {
    assert_eq!(run("=DELTA(5, 5)"), Value::Number(1.0));
    assert_eq!(run("=DELTA(5, 6)"), Value::Number(0.0));
}

#[test]
fn sanity_bitwise() {
    assert_eq!(run("=BITAND(12, 10)"), Value::Number(8.0));
    assert_eq!(run("=BITOR(12, 10)"), Value::Number(14.0));
    assert_eq!(run("=BITXOR(12, 10)"), Value::Number(6.0));
}
