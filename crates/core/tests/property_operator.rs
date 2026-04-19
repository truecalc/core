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

// 1. Addition is commutative
#[test]
fn add_commutative() {
    proptest!(ProptestConfig::with_cases(CASES), |(a in small_f64(), b in small_f64())| {
        let ab = run_vars("=x + y", vec![("x", a), ("y", b)]);
        let ba = run_vars("=x + y", vec![("x", b), ("y", a)]);
        prop_assert_eq!(ab, ba);
    });
    eprintln!("proptest: {CASES} cases (a ∈ [-1e6, 1e6], b ∈ [-1e6, 1e6])");
}

// 2. Multiplication is commutative
#[test]
fn multiply_commutative() {
    proptest!(ProptestConfig::with_cases(CASES), |(a in small_f64(), b in small_f64())| {
        let ab = run_vars("=x * y", vec![("x", a), ("y", b)]);
        let ba = run_vars("=x * y", vec![("x", b), ("y", a)]);
        prop_assert_eq!(ab, ba);
    });
    eprintln!("proptest: {CASES} cases (a ∈ [-1e6, 1e6], b ∈ [-1e6, 1e6])");
}

// 3. Additive identity: x + 0 = x
#[test]
fn add_identity_zero() {
    proptest!(ProptestConfig::with_cases(CASES), |(x in small_f64())| {
        let result = run_vars("=x + 0", vec![("x", x)]);
        prop_assert_eq!(result, Value::Number(x));
    });
    eprintln!("proptest: {CASES} cases (x ∈ [-1e6, 1e6])");
}

// 4. Multiplicative identity: x * 1 = x
#[test]
fn multiply_identity_one() {
    proptest!(ProptestConfig::with_cases(CASES), |(x in small_f64())| {
        let result = run_vars("=x * 1", vec![("x", x)]);
        prop_assert_eq!(result, Value::Number(x));
    });
    eprintln!("proptest: {CASES} cases (x ∈ [-1e6, 1e6])");
}

// 5. Multiplicative absorbing: x * 0 = 0
#[test]
fn multiply_absorbing_zero() {
    proptest!(ProptestConfig::with_cases(CASES), |(x in small_f64())| {
        let result = run_vars("=x * 0", vec![("x", x)]);
        prop_assert_eq!(result, Value::Number(0.0));
    });
    eprintln!("proptest: {CASES} cases (x ∈ [-1e6, 1e6])");
}

// 6. Double negation: -(-x) = x
#[test]
fn double_negation() {
    proptest!(ProptestConfig::with_cases(CASES), |(x in small_f64())| {
        let result = run_vars("=-(-x)", vec![("x", x)]);
        prop_assert_eq!(result, Value::Number(x));
    });
    eprintln!("proptest: {CASES} cases (x ∈ [-1e6, 1e6])");
}

// 7. EQ is reflexive: x = x is always TRUE
#[test]
fn eq_reflexive() {
    proptest!(ProptestConfig::with_cases(CASES), |(x in small_f64())| {
        let result = run_vars("=x = x", vec![("x", x)]);
        prop_assert_eq!(result, Value::Bool(true));
    });
    eprintln!("proptest: {CASES} cases (x ∈ [-1e6, 1e6])");
}

// 8. NE is anti-reflexive: x <> x is always FALSE
#[test]
fn ne_anti_reflexive() {
    proptest!(ProptestConfig::with_cases(CASES), |(x in small_f64())| {
        let result = run_vars("=x <> x", vec![("x", x)]);
        prop_assert_eq!(result, Value::Bool(false));
    });
    eprintln!("proptest: {CASES} cases (x ∈ [-1e6, 1e6])");
}

// 9. LTE reflexive: x <= x is always TRUE
#[test]
fn lte_reflexive() {
    proptest!(ProptestConfig::with_cases(CASES), |(x in small_f64())| {
        let result = run_vars("=x <= x", vec![("x", x)]);
        prop_assert_eq!(result, Value::Bool(true));
    });
    eprintln!("proptest: {CASES} cases (x ∈ [-1e6, 1e6])");
}

// 10. GTE reflexive: x >= x is always TRUE
#[test]
fn gte_reflexive() {
    proptest!(ProptestConfig::with_cases(CASES), |(x in small_f64())| {
        let result = run_vars("=x >= x", vec![("x", x)]);
        prop_assert_eq!(result, Value::Bool(true));
    });
    eprintln!("proptest: {CASES} cases (x ∈ [-1e6, 1e6])");
}

// 11. Percent operator: x% = x / 100
#[test]
fn percent_divides_by_100() {
    proptest!(ProptestConfig::with_cases(CASES), |(x in small_f64())| {
        let pct = run_vars("=x%", vec![("x", x)]);
        let div = run_vars("=x / 100", vec![("x", x)]);
        prop_assert_eq!(pct, div);
    });
    eprintln!("proptest: {CASES} cases (x ∈ [-1e6, 1e6])");
}

// 12. ISBETWEEN: x is always between x and x (inclusive on both ends)
#[test]
fn isbetween_reflexive() {
    proptest!(ProptestConfig::with_cases(CASES), |(x in small_f64())| {
        let result = run_vars("=ISBETWEEN(x, x, x)", vec![("x", x)]);
        prop_assert_eq!(result, Value::Bool(true));
    });
    eprintln!("proptest: {CASES} cases (x ∈ [-1e6, 1e6])");
}

// Sanity checks
#[test]
fn sanity_add() {
    assert_eq!(run("=3 + 4"), Value::Number(7.0));
}

#[test]
fn sanity_multiply() {
    assert_eq!(run("=3 * 4"), Value::Number(12.0));
}

#[test]
fn sanity_eq() {
    assert_eq!(run("=5 = 5"), Value::Bool(true));
    assert_eq!(run("=5 = 6"), Value::Bool(false));
}
