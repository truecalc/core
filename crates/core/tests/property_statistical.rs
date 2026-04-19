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

// 1. AVERAGE of two values equals their midpoint
#[test]
fn average_of_two_is_midpoint() {
    proptest!(ProptestConfig::with_cases(CASES), |(a in small_f64(), b in small_f64())| {
        let avg = run_vars("=AVERAGE(x, y)", vec![("x", a), ("y", b)]);
        if let Value::Number(v) = avg {
            let expected = (a + b) / 2.0;
            prop_assert!((v - expected).abs() < 1e-9,
                "AVERAGE({}, {}) = {} ≠ {}", a, b, v, expected);
        }
    });
    eprintln!("proptest: {CASES} cases (a ∈ [-1e6, 1e6], b ∈ [-1e6, 1e6])");
}

// 2. MIN(a, b) <= MAX(a, b)
#[test]
fn min_lte_max() {
    proptest!(ProptestConfig::with_cases(CASES), |(a in small_f64(), b in small_f64())| {
        let mn = run_vars("=MIN(x, y)", vec![("x", a), ("y", b)]);
        let mx = run_vars("=MAX(x, y)", vec![("x", a), ("y", b)]);
        if let (Value::Number(lo), Value::Number(hi)) = (mn, mx) {
            prop_assert!(lo <= hi + 1e-12, "MIN={} > MAX={}", lo, hi);
        }
    });
    eprintln!("proptest: {CASES} cases (a ∈ [-1e6, 1e6], b ∈ [-1e6, 1e6])");
}

// 3. AVERAGE(a, b) is between MIN(a, b) and MAX(a, b)
#[test]
fn average_between_min_and_max() {
    proptest!(ProptestConfig::with_cases(CASES), |(a in small_f64(), b in small_f64())| {
        let mn = run_vars("=MIN(x, y)", vec![("x", a), ("y", b)]);
        let mx = run_vars("=MAX(x, y)", vec![("x", a), ("y", b)]);
        let avg = run_vars("=AVERAGE(x, y)", vec![("x", a), ("y", b)]);
        if let (Value::Number(lo), Value::Number(hi), Value::Number(av)) = (mn, mx, avg) {
            prop_assert!(av >= lo - 1e-9, "AVERAGE={} < MIN={}", av, lo);
            prop_assert!(av <= hi + 1e-9, "AVERAGE={} > MAX={}", av, hi);
        }
    });
    eprintln!("proptest: {CASES} cases (a ∈ [-1e6, 1e6], b ∈ [-1e6, 1e6])");
}

// 4. STDEV of two values is non-negative
#[test]
fn stdev_non_negative() {
    proptest!(ProptestConfig::with_cases(CASES), |(a in small_f64(), b in small_f64())| {
        let result = run_vars("=STDEV(x, y)", vec![("x", a), ("y", b)]);
        if let Value::Number(v) = result {
            prop_assert!(v >= 0.0, "STDEV({}, {}) = {} < 0", a, b, v);
        }
    });
    eprintln!("proptest: {CASES} cases (a ∈ [-1e6, 1e6], b ∈ [-1e6, 1e6])");
}

// 5. COUNT of numbers is non-negative
#[test]
fn count_non_negative() {
    proptest!(ProptestConfig::with_cases(CASES), |(a in small_f64(), b in small_f64())| {
        let result = run_vars("=COUNT(x, y)", vec![("x", a), ("y", b)]);
        if let Value::Number(v) = result {
            prop_assert!(v >= 0.0, "COUNT returned {}", v);
        }
    });
    eprintln!("proptest: {CASES} cases (a ∈ [-1e6, 1e6], b ∈ [-1e6, 1e6])");
}

// 6. MEDIAN of two values equals their average
#[test]
fn median_of_two_equals_average() {
    proptest!(ProptestConfig::with_cases(CASES), |(a in small_f64(), b in small_f64())| {
        let med = run_vars("=MEDIAN(x, y)", vec![("x", a), ("y", b)]);
        let avg = run_vars("=AVERAGE(x, y)", vec![("x", a), ("y", b)]);
        if let (Value::Number(m), Value::Number(av)) = (med, avg) {
            prop_assert!((m - av).abs() < 1e-9, "MEDIAN={} AVERAGE={}", m, av);
        }
    });
    eprintln!("proptest: {CASES} cases (a ∈ [-1e6, 1e6], b ∈ [-1e6, 1e6])");
}

// 7. LARGE(a, b, 1) >= LARGE(a, b, 2): first-largest >= second-largest
#[test]
fn large_rank1_gte_rank2() {
    proptest!(ProptestConfig::with_cases(CASES), |(a in small_f64(), b in small_f64())| {
        let l1 = run_vars("=LARGE({x, y}, 1)", vec![("x", a), ("y", b)]);
        let l2 = run_vars("=LARGE({x, y}, 2)", vec![("x", a), ("y", b)]);
        if let (Value::Number(v1), Value::Number(v2)) = (l1, l2) {
            prop_assert!(v1 >= v2 - 1e-12, "LARGE rank1={} < rank2={}", v1, v2);
        }
    });
    eprintln!("proptest: {CASES} cases (a ∈ [-1e6, 1e6], b ∈ [-1e6, 1e6])");
}

// 8. SMALL(a, b, 1) <= SMALL(a, b, 2): first-smallest <= second-smallest
#[test]
fn small_rank1_lte_rank2() {
    proptest!(ProptestConfig::with_cases(CASES), |(a in small_f64(), b in small_f64())| {
        let s1 = run_vars("=SMALL({x, y}, 1)", vec![("x", a), ("y", b)]);
        let s2 = run_vars("=SMALL({x, y}, 2)", vec![("x", a), ("y", b)]);
        if let (Value::Number(v1), Value::Number(v2)) = (s1, s2) {
            prop_assert!(v1 <= v2 + 1e-12, "SMALL rank1={} > rank2={}", v1, v2);
        }
    });
    eprintln!("proptest: {CASES} cases (a ∈ [-1e6, 1e6], b ∈ [-1e6, 1e6])");
}

// Sanity checks
#[test]
fn sanity_average() {
    assert_eq!(run("=AVERAGE(2, 4)"), Value::Number(3.0));
}

#[test]
fn sanity_min_max() {
    assert_eq!(run("=MIN(3, 7)"), Value::Number(3.0));
    assert_eq!(run("=MAX(3, 7)"), Value::Number(7.0));
}
