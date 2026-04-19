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

// 1. ISNUMBER on any number is always TRUE
#[test]
fn isnumber_on_number_is_true() {
    proptest!(ProptestConfig::with_cases(CASES), |(x in small_f64())| {
        let result = run_vars("=ISNUMBER(x)", vec![("x", x)]);
        prop_assert_eq!(result, Value::Bool(true));
    });
    eprintln!("proptest: {CASES} cases (x ∈ [-1e6, 1e6])");
}

// 2. ISTEXT on a number is always FALSE
#[test]
fn istext_on_number_is_false() {
    proptest!(ProptestConfig::with_cases(CASES), |(x in small_f64())| {
        let result = run_vars("=ISTEXT(x)", vec![("x", x)]);
        prop_assert_eq!(result, Value::Bool(false));
    });
    eprintln!("proptest: {CASES} cases (x ∈ [-1e6, 1e6])");
}

// 3. ISODD and ISEVEN are complementary for integers
#[test]
fn isodd_iseven_complementary() {
    proptest!(ProptestConfig::with_cases(CASES), |(n in 1i32..=500000i32)| {
        let odd = run_vars("=ISODD(n)", vec![("n", n as f64)]);
        let even = run_vars("=ISEVEN(n)", vec![("n", n as f64)]);
        // Exactly one of ISODD or ISEVEN must be true for any integer
        prop_assert!(odd != even,
            "ISODD({}) = {:?}, ISEVEN({}) = {:?} — should differ", n, odd, n, even);
    });
    eprintln!("proptest: {CASES} cases (n ∈ [1, 500000])");
}

// 4. ISODD(2*n) = FALSE for all integers (even numbers are not odd)
#[test]
fn isodd_of_even_number_is_false() {
    proptest!(ProptestConfig::with_cases(CASES), |(n in 0i32..=500000i32)| {
        let result = run_vars("=ISODD(n * 2)", vec![("n", n as f64)]);
        prop_assert_eq!(result, Value::Bool(false));
    });
    eprintln!("proptest: {CASES} cases (n ∈ [0, 500000])");
}

// 5. ISEVEN(2*n) = TRUE for all integers (2n is always even)
#[test]
fn iseven_of_doubled_integer_is_true() {
    proptest!(ProptestConfig::with_cases(CASES), |(n in 0i32..=500000i32)| {
        let result = run_vars("=ISEVEN(n * 2)", vec![("n", n as f64)]);
        prop_assert_eq!(result, Value::Bool(true));
    });
    eprintln!("proptest: {CASES} cases (n ∈ [0, 500000])");
}

// 6. ISBLANK on a number is FALSE
#[test]
fn isblank_on_number_is_false() {
    proptest!(ProptestConfig::with_cases(CASES), |(x in small_f64())| {
        let result = run_vars("=ISBLANK(x)", vec![("x", x)]);
        prop_assert_eq!(result, Value::Bool(false));
    });
    eprintln!("proptest: {CASES} cases (x ∈ [-1e6, 1e6])");
}

// 7. ISLOGICAL(TRUE) = TRUE and ISLOGICAL(FALSE) = TRUE
#[test]
fn islogical_on_booleans() {
    assert_eq!(run("=ISLOGICAL(TRUE)"), Value::Bool(true));
    assert_eq!(run("=ISLOGICAL(FALSE)"), Value::Bool(true));
}

// 8. ISLOGICAL on a number is FALSE
#[test]
fn islogical_on_number_is_false() {
    proptest!(ProptestConfig::with_cases(CASES), |(x in small_f64())| {
        let result = run_vars("=ISLOGICAL(x)", vec![("x", x)]);
        prop_assert_eq!(result, Value::Bool(false));
    });
    eprintln!("proptest: {CASES} cases (x ∈ [-1e6, 1e6])");
}

// Sanity checks
#[test]
fn sanity_isodd_iseven() {
    assert_eq!(run("=ISODD(3)"), Value::Bool(true));
    assert_eq!(run("=ISODD(4)"), Value::Bool(false));
    assert_eq!(run("=ISEVEN(4)"), Value::Bool(true));
    assert_eq!(run("=ISEVEN(3)"), Value::Bool(false));
}
