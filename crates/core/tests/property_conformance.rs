// crates/core/tests/property_conformance.rs
//
// Conformance-driven property tests: for each function confirmed by Google Sheets
// oracle fixtures, verify that the mathematical property holds across generated inputs.
//
// These tests complement the fixed-row conformance tests in conformance.rs.
// They catch regressions for inputs that don't appear in the fixture files.

use truecalc_core::{evaluate, Value};
use proptest::prelude::*;
use std::collections::HashMap;

const CASES: u32 = 500;

fn run(formula: &str) -> Value {
    evaluate(formula, &HashMap::new())
}

fn run_vars(formula: &str, vars: Vec<(&str, f64)>) -> Value {
    let map = vars
        .into_iter()
        .map(|(k, v)| (k.to_string(), Value::Number(v)))
        .collect();
    evaluate(formula, &map)
}

fn run_text(formula: &str, var: &str, s: &str) -> Value {
    let mut m = HashMap::new();
    m.insert(var.to_string(), Value::Text(s.to_string()));
    evaluate(formula, &m)
}

fn small_f64() -> impl Strategy<Value = f64> {
    -1e6f64..1e6f64
}

fn pos_f64() -> impl Strategy<Value = f64> {
    1e-3f64..1e4f64
}

fn ascii_str() -> impl Strategy<Value = String> {
    "[a-z]{0,20}"
}

// ─── M1: Math ────────────────────────────────────────────────────────────────

// ABS: confirmed by m1/Math.xlsx — result is always >= 0
#[test]
fn m1_abs_non_negative() {
    proptest!(proptest::prelude::ProptestConfig::with_cases(CASES), |(x in small_f64())| {
        let r = run_vars("=ABS(x)", vec![("x", x)]);
        if let Value::Number(n) = r {
            prop_assert!(n >= 0.0, "ABS({}) = {} is negative", x, n);
        }
    });
    eprintln!("proptest: {CASES} cases (x ∈ [-1e6, 1e6])");
}

// ROUND(x, 0): result is an integer (confirmed by m1/Math.xlsx)
#[test]
fn m1_round_zero_is_integer() {
    proptest!(proptest::prelude::ProptestConfig::with_cases(CASES), |(x in small_f64())| {
        let r = run_vars("=ROUND(x, 0)", vec![("x", x)]);
        if let Value::Number(n) = r {
            prop_assert!(
                (n - n.round()).abs() < 1e-10,
                "ROUND({}, 0) = {} is not an integer", x, n
            );
        }
    });
    eprintln!("proptest: {CASES} cases (x ∈ [-1e6, 1e6])");
}

// ROUND(x, 2): result differs from x by less than 0.005
#[test]
fn m1_round_two_within_half_ulp() {
    proptest!(proptest::prelude::ProptestConfig::with_cases(CASES), |(x in small_f64())| {
        let r = run_vars("=ROUND(x, 2)", vec![("x", x)]);
        if let Value::Number(n) = r {
            prop_assert!(
                (n - x).abs() <= 0.005 + 1e-9,
                "ROUND({}, 2) = {} differs by more than 0.005", x, n
            );
        }
    });
    eprintln!("proptest: {CASES} cases (x ∈ [-1e6, 1e6])");
}

// SQRT(x): x >= 0 → result >= 0 and result^2 ≈ x
#[test]
fn m1_sqrt_non_negative() {
    proptest!(proptest::prelude::ProptestConfig::with_cases(CASES), |(x in pos_f64())| {
        let r = run_vars("=SQRT(x)", vec![("x", x)]);
        if let Value::Number(n) = r {
            prop_assert!(n >= 0.0, "SQRT({}) = {} is negative", x, n);
        }
    });
    eprintln!("proptest: {CASES} cases (x ∈ [1e-3, 1e4])");
}

// SQRT(x)^2 ≈ x for x > 0 (confirmed by m1/Math.xlsx)
#[test]
fn m1_sqrt_inverse_square() {
    proptest!(proptest::prelude::ProptestConfig::with_cases(CASES), |(x in pos_f64())| {
        let r = run_vars("=SQRT(x)", vec![("x", x)]);
        if let Value::Number(n) = r {
            prop_assert!(
                (n * n - x).abs() / x.max(1.0) < 1e-9,
                "SQRT({})^2 = {} (expected {})", x, n * n, x
            );
        }
    });
    eprintln!("proptest: {CASES} cases (x ∈ [1e-3, 1e4])");
}

// MOD(a, b): b > 0 → 0 <= result < b (confirmed by m1/Math.xlsx)
#[test]
fn m1_mod_remainder_bounds() {
    proptest!(proptest::prelude::ProptestConfig::with_cases(CASES), |(a in small_f64(), b in pos_f64())| {
        let r = run_vars("=MOD(a, b)", vec![("a", a), ("b", b)]);
        if let Value::Number(n) = r {
            prop_assert!(n >= 0.0, "MOD({},{}) = {} is negative", a, b, n);
            prop_assert!(n < b + 1e-9, "MOD({},{}) = {} >= b", a, b, n);
        }
    });
    eprintln!("proptest: {CASES} cases (a ∈ [-1e6, 1e6], b ∈ [1e-3, 1e4])");
}

// POWER(x, 2) == x * x for x >= 0 (confirmed by m1/Math.xlsx)
#[test]
fn m1_power_two_equals_square() {
    proptest!(proptest::prelude::ProptestConfig::with_cases(CASES), |(x in 0.0f64..1000.0f64)| {
        let r = run_vars("=POWER(x, 2)", vec![("x", x)]);
        if let Value::Number(n) = r {
            prop_assert!(
                (n - x * x).abs() < 1e-6,
                "POWER({}, 2) = {} but x*x = {}", x, n, x * x
            );
        }
    });
    eprintln!("proptest: {CASES} cases (x ∈ [0, 1000])");
}

// ─── M1: Text ────────────────────────────────────────────────────────────────

// LEN: confirmed by m1/Text.xlsx — always >= 0
#[test]
fn m1_len_non_negative() {
    proptest!(proptest::prelude::ProptestConfig::with_cases(CASES), |(s in ascii_str())| {
        let r = run_text("=LEN(x)", "x", &s);
        if let Value::Number(n) = r {
            prop_assert!(n >= 0.0, "LEN({:?}) = {} is negative", s, n);
        }
    });
    eprintln!("proptest: {CASES} cases (s ∈ [a-z]{{0,20}})");
}

// LEN(s) == number of characters in s (confirmed by m1/Text.xlsx)
#[test]
fn m1_len_equals_char_count() {
    proptest!(proptest::prelude::ProptestConfig::with_cases(CASES), |(s in ascii_str())| {
        let r = run_text("=LEN(x)", "x", &s);
        if let Value::Number(n) = r {
            prop_assert_eq!(
                n as usize,
                s.chars().count(),
                "LEN({:?}) = {} but char count = {}", s, n, s.chars().count()
            );
        }
    });
    eprintln!("proptest: {CASES} cases (s ∈ [a-z]{{0,20}})");
}

// LOWER: result is always lowercase (confirmed by m1/Text.xlsx)
#[test]
fn m1_lower_result_is_lowercase() {
    proptest!(proptest::prelude::ProptestConfig::with_cases(CASES), |(s in ascii_str())| {
        let r = run_text("=LOWER(x)", "x", &s);
        if let Value::Text(t) = r {
            prop_assert_eq!(
                t.clone(),
                t.to_lowercase(),
                "LOWER({:?}) = {:?} is not lowercase", s, t
            );
        }
    });
    eprintln!("proptest: {CASES} cases (s ∈ [a-z]{{0,20}})");
}

// UPPER: result is always uppercase (confirmed by m1/Text.xlsx)
#[test]
fn m1_upper_result_is_uppercase() {
    proptest!(proptest::prelude::ProptestConfig::with_cases(CASES), |(s in ascii_str())| {
        let r = run_text("=UPPER(x)", "x", &s);
        if let Value::Text(t) = r {
            prop_assert_eq!(
                t.clone(),
                t.to_uppercase(),
                "UPPER({:?}) = {:?} is not uppercase", s, t
            );
        }
    });
    eprintln!("proptest: {CASES} cases (s ∈ [a-z]{{0,20}})");
}

// ─── M1: Logical ─────────────────────────────────────────────────────────────

// IF(TRUE, a, b) == a (confirmed by m1/Logical.xlsx)
#[test]
fn m1_if_true_picks_first() {
    proptest!(proptest::prelude::ProptestConfig::with_cases(CASES), |(a in small_f64(), b in small_f64())| {
        let r = run_vars("=IF(TRUE, x, y)", vec![("x", a), ("y", b)]);
        prop_assert_eq!(r, Value::Number(a));
    });
    eprintln!("proptest: {CASES} cases (a ∈ [-1e6, 1e6], b ∈ [-1e6, 1e6])");
}

// IF(FALSE, a, b) == b (confirmed by m1/Logical.xlsx)
#[test]
fn m1_if_false_picks_second() {
    proptest!(proptest::prelude::ProptestConfig::with_cases(CASES), |(a in small_f64(), b in small_f64())| {
        let r = run_vars("=IF(FALSE, x, y)", vec![("x", a), ("y", b)]);
        prop_assert_eq!(r, Value::Number(b));
    });
    eprintln!("proptest: {CASES} cases (a ∈ [-1e6, 1e6], b ∈ [-1e6, 1e6])");
}

// AND(x, y) matches x && y (confirmed by m1/Logical.xlsx)
#[test]
fn m1_and_truth_table() {
    proptest!(proptest::prelude::ProptestConfig::with_cases(CASES), |(x in proptest::bool::ANY, y in proptest::bool::ANY)| {
        let tx = if x { "TRUE" } else { "FALSE" };
        let ty = if y { "TRUE" } else { "FALSE" };
        let formula = format!("=AND({},{})", tx, ty);
        let r = run(&formula);
        if let Value::Bool(b) = r {
            prop_assert_eq!(b, x && y, "AND({},{}) = {} (expected {})", x, y, b, x && y);
        }
    });
    eprintln!("proptest: {CASES} cases (x ∈ {{true, false}}, y ∈ {{true, false}})");
}

// OR(x, y) matches x || y (confirmed by m1/Logical.xlsx)
#[test]
fn m1_or_truth_table() {
    proptest!(proptest::prelude::ProptestConfig::with_cases(CASES), |(x in proptest::bool::ANY, y in proptest::bool::ANY)| {
        let tx = if x { "TRUE" } else { "FALSE" };
        let ty = if y { "TRUE" } else { "FALSE" };
        let formula = format!("=OR({},{})", tx, ty);
        let r = run(&formula);
        if let Value::Bool(b) = r {
            prop_assert_eq!(b, x || y, "OR({},{}) = {} (expected {})", x, y, b, x || y);
        }
    });
    eprintln!("proptest: {CASES} cases (x ∈ {{true, false}}, y ∈ {{true, false}})");
}

// ─── M2: Math ────────────────────────────────────────────────────────────────

// Multiplication commutativity: a*b == b*a (confirmed by m2/Math.xlsx)
#[test]
fn m2_multiplication_commutative() {
    proptest!(proptest::prelude::ProptestConfig::with_cases(CASES), |(a in small_f64(), b in small_f64())| {
        let ab = run_vars("=x*y", vec![("x", a), ("y", b)]);
        let ba = run_vars("=x*y", vec![("x", b), ("y", a)]);
        prop_assert_eq!(ab, ba, "{}*{} != {}*{}", a, b, b, a);
    });
    eprintln!("proptest: {CASES} cases (a ∈ [-1e6, 1e6], b ∈ [-1e6, 1e6])");
}

// Addition commutativity: a+b == b+a (confirmed by m2/Math.xlsx)
#[test]
fn m2_addition_commutative() {
    proptest!(proptest::prelude::ProptestConfig::with_cases(CASES), |(a in small_f64(), b in small_f64())| {
        let ab = run_vars("=x+y", vec![("x", a), ("y", b)]);
        let ba = run_vars("=x+y", vec![("x", b), ("y", a)]);
        prop_assert_eq!(ab, ba, "{}+{} != {}+{}", a, b, b, a);
    });
    eprintln!("proptest: {CASES} cases (a ∈ [-1e6, 1e6], b ∈ [-1e6, 1e6])");
}

// ABS(-x) == ABS(x) — symmetry (confirmed by m2/Math.xlsx)
#[test]
fn m2_abs_symmetry() {
    proptest!(proptest::prelude::ProptestConfig::with_cases(CASES), |(x in small_f64())| {
        let pos = run_vars("=ABS(x)", vec![("x", x)]);
        let neg = run_vars("=ABS(x)", vec![("x", -x)]);
        prop_assert_eq!(pos, neg, "ABS({}) != ABS({})", x, -x);
    });
    eprintln!("proptest: {CASES} cases (x ∈ [-1e6, 1e6])");
}
