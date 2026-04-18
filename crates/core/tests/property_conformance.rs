// crates/core/tests/property_conformance.rs
//
// Conformance-driven property tests: for each function confirmed by Google Sheets
// oracle fixtures, verify that the mathematical property holds across generated inputs.
//
// These tests complement the fixed-row conformance tests in conformance.rs.
// They catch regressions for inputs that don't appear in the fixture files.

use ganit_core::{evaluate, Value};
use proptest::prelude::*;
use std::collections::HashMap;

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

proptest! {
    // ABS: confirmed by m1/Math.xlsx — result is always >= 0
    #[test]
    fn m1_abs_non_negative(x in small_f64()) {
        let r = run_vars("=ABS(x)", vec![("x", x)]);
        if let Value::Number(n) = r {
            prop_assert!(n >= 0.0, "ABS({}) = {} is negative", x, n);
        }
    }

    // ROUND(x, 0): result is an integer (confirmed by m1/Math.xlsx)
    #[test]
    fn m1_round_zero_is_integer(x in small_f64()) {
        let r = run_vars("=ROUND(x, 0)", vec![("x", x)]);
        if let Value::Number(n) = r {
            prop_assert!(
                (n - n.round()).abs() < 1e-10,
                "ROUND({}, 0) = {} is not an integer", x, n
            );
        }
    }

    // ROUND(x, 2): result differs from x by less than 0.005
    #[test]
    fn m1_round_two_within_half_ulp(x in small_f64()) {
        let r = run_vars("=ROUND(x, 2)", vec![("x", x)]);
        if let Value::Number(n) = r {
            prop_assert!(
                (n - x).abs() <= 0.005 + 1e-9,
                "ROUND({}, 2) = {} differs by more than 0.005", x, n
            );
        }
    }

    // SQRT(x): x >= 0 → result >= 0 and result^2 ≈ x
    #[test]
    fn m1_sqrt_non_negative(x in pos_f64()) {
        let r = run_vars("=SQRT(x)", vec![("x", x)]);
        if let Value::Number(n) = r {
            prop_assert!(n >= 0.0, "SQRT({}) = {} is negative", x, n);
        }
    }

    // SQRT(x)^2 ≈ x for x > 0 (confirmed by m1/Math.xlsx)
    #[test]
    fn m1_sqrt_inverse_square(x in pos_f64()) {
        let r = run_vars("=SQRT(x)", vec![("x", x)]);
        if let Value::Number(n) = r {
            prop_assert!(
                (n * n - x).abs() / x.max(1.0) < 1e-9,
                "SQRT({})^2 = {} (expected {})", x, n * n, x
            );
        }
    }

    // MOD(a, b): b > 0 → 0 <= result < b (confirmed by m1/Math.xlsx)
    #[test]
    fn m1_mod_remainder_bounds(a in small_f64(), b in pos_f64()) {
        let r = run_vars("=MOD(a, b)", vec![("a", a), ("b", b)]);
        if let Value::Number(n) = r {
            prop_assert!(n >= 0.0, "MOD({},{}) = {} is negative", a, b, n);
            prop_assert!(n < b + 1e-9, "MOD({},{}) = {} >= b", a, b, n);
        }
    }

    // POWER(x, 2) == x * x for x >= 0 (confirmed by m1/Math.xlsx)
    #[test]
    fn m1_power_two_equals_square(x in 0.0f64..1000.0f64) {
        let r = run_vars("=POWER(x, 2)", vec![("x", x)]);
        if let Value::Number(n) = r {
            prop_assert!(
                (n - x * x).abs() < 1e-6,
                "POWER({}, 2) = {} but x*x = {}", x, n, x * x
            );
        }
    }
}

// ─── M1: Text ────────────────────────────────────────────────────────────────

proptest! {
    // LEN: confirmed by m1/Text.xlsx — always >= 0
    #[test]
    fn m1_len_non_negative(s in ascii_str()) {
        let r = run_text("=LEN(x)", "x", &s);
        if let Value::Number(n) = r {
            prop_assert!(n >= 0.0, "LEN({:?}) = {} is negative", s, n);
        }
    }

    // LEN(s) == number of characters in s (confirmed by m1/Text.xlsx)
    #[test]
    fn m1_len_equals_char_count(s in ascii_str()) {
        let r = run_text("=LEN(x)", "x", &s);
        if let Value::Number(n) = r {
            prop_assert_eq!(
                n as usize,
                s.chars().count(),
                "LEN({:?}) = {} but char count = {}", s, n, s.chars().count()
            );
        }
    }

    // LOWER: result is always lowercase (confirmed by m1/Text.xlsx)
    #[test]
    fn m1_lower_result_is_lowercase(s in ascii_str()) {
        let r = run_text("=LOWER(x)", "x", &s);
        if let Value::Text(t) = r {
            prop_assert_eq!(
                t.clone(),
                t.to_lowercase(),
                "LOWER({:?}) = {:?} is not lowercase", s, t
            );
        }
    }

    // UPPER: result is always uppercase (confirmed by m1/Text.xlsx)
    #[test]
    fn m1_upper_result_is_uppercase(s in ascii_str()) {
        let r = run_text("=UPPER(x)", "x", &s);
        if let Value::Text(t) = r {
            prop_assert_eq!(
                t.clone(),
                t.to_uppercase(),
                "UPPER({:?}) = {:?} is not uppercase", s, t
            );
        }
    }
}

// ─── M1: Logical ─────────────────────────────────────────────────────────────

proptest! {
    // IF(TRUE, a, b) == a (confirmed by m1/Logical.xlsx)
    #[test]
    fn m1_if_true_picks_first(a in small_f64(), b in small_f64()) {
        let r = run_vars("=IF(TRUE, x, y)", vec![("x", a), ("y", b)]);
        prop_assert_eq!(r, Value::Number(a));
    }

    // IF(FALSE, a, b) == b (confirmed by m1/Logical.xlsx)
    #[test]
    fn m1_if_false_picks_second(a in small_f64(), b in small_f64()) {
        let r = run_vars("=IF(FALSE, x, y)", vec![("x", a), ("y", b)]);
        prop_assert_eq!(r, Value::Number(b));
    }

    // AND(x, y) matches x && y (confirmed by m1/Logical.xlsx)
    #[test]
    fn m1_and_truth_table(x in proptest::bool::ANY, y in proptest::bool::ANY) {
        let tx = if x { "TRUE" } else { "FALSE" };
        let ty = if y { "TRUE" } else { "FALSE" };
        let formula = format!("=AND({},{})", tx, ty);
        let r = run(&formula);
        if let Value::Bool(b) = r {
            prop_assert_eq!(b, x && y, "AND({},{}) = {} (expected {})", x, y, b, x && y);
        }
    }

    // OR(x, y) matches x || y (confirmed by m1/Logical.xlsx)
    #[test]
    fn m1_or_truth_table(x in proptest::bool::ANY, y in proptest::bool::ANY) {
        let tx = if x { "TRUE" } else { "FALSE" };
        let ty = if y { "TRUE" } else { "FALSE" };
        let formula = format!("=OR({},{})", tx, ty);
        let r = run(&formula);
        if let Value::Bool(b) = r {
            prop_assert_eq!(b, x || y, "OR({},{}) = {} (expected {})", x, y, b, x || y);
        }
    }
}

// ─── M2: Math ────────────────────────────────────────────────────────────────

proptest! {
    // Multiplication commutativity: a*b == b*a (confirmed by m2/Math.xlsx)
    #[test]
    fn m2_multiplication_commutative(a in small_f64(), b in small_f64()) {
        let ab = run_vars("=x*y", vec![("x", a), ("y", b)]);
        let ba = run_vars("=x*y", vec![("x", b), ("y", a)]);
        prop_assert_eq!(ab, ba, "{}*{} != {}*{}", a, b, b, a);
    }

    // Addition commutativity: a+b == b+a (confirmed by m2/Math.xlsx)
    #[test]
    fn m2_addition_commutative(a in small_f64(), b in small_f64()) {
        let ab = run_vars("=x+y", vec![("x", a), ("y", b)]);
        let ba = run_vars("=x+y", vec![("x", b), ("y", a)]);
        prop_assert_eq!(ab, ba, "{}+{} != {}+{}", a, b, b, a);
    }

    // ABS(-x) == ABS(x) — symmetry (confirmed by m2/Math.xlsx)
    #[test]
    fn m2_abs_symmetry(x in small_f64()) {
        let pos = run_vars("=ABS(x)", vec![("x", x)]);
        let neg = run_vars("=ABS(x)", vec![("x", -x)]);
        prop_assert_eq!(pos, neg, "ABS({}) != ABS({})", x, -x);
    }
}
