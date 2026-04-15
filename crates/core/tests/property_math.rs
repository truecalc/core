use proptest::prelude::*;
use ganit_core::{evaluate, Value};
use std::collections::HashMap;

fn run(formula: &str) -> Value {
    evaluate(formula, HashMap::new())
}

fn run_vars(formula: &str, vars: Vec<(&str, f64)>) -> Value {
    let map = vars.into_iter().map(|(k, v)| (k.to_string(), Value::Number(v))).collect();
    evaluate(formula, map)
}

fn small_f64() -> impl Strategy<Value = f64> {
    -1e6f64..1e6f64
}

fn finite_f64() -> impl Strategy<Value = f64> {
    prop::num::f64::NORMAL | prop::num::f64::ZERO | prop::num::f64::SUBNORMAL
}

proptest! {
    // 1. SUM is commutative
    #[test]
    fn sum_commutative(a in small_f64(), b in small_f64()) {
        let ab = run_vars("=SUM(x, y)", vec![("x", a), ("y", b)]);
        let ba = run_vars("=SUM(x, y)", vec![("x", b), ("y", a)]);
        prop_assert_eq!(ab, ba);
    }

    // 2. ABS always non-negative
    #[test]
    fn abs_non_negative(x in finite_f64()) {
        let result = run_vars("=ABS(x)", vec![("x", x)]);
        prop_assert!(matches!(result, Value::Number(_)), "expected Number, got {:?}", result);
        if let Value::Number(n) = result {
            prop_assert!(n >= 0.0);
        }
    }

    // 3. SQRT(x^2) ≈ ABS(x) for non-negative x
    #[test]
    fn sqrt_of_square(x in 0.0f64..1e6f64) {
        let sqrt_sq = run_vars("=SQRT(x*x)", vec![("x", x)]);
        let abs_x = run_vars("=ABS(x)", vec![("x", x)]);
        prop_assert!(matches!(sqrt_sq, Value::Number(_)), "expected Number for sqrt_sq, got {:?}", sqrt_sq);
        prop_assert!(matches!(abs_x, Value::Number(_)), "expected Number for abs_x, got {:?}", abs_x);
        if let (Value::Number(a), Value::Number(b)) = (sqrt_sq, abs_x) {
            prop_assert!((a - b).abs() < 1e-6, "SQRT(x^2)={} ABS(x)={}", a, b);
        }
    }

    // 4. LN(EXP(x)) ≈ x for small x (LN is natural log, EXP is natural exponential)
    #[test]
    fn ln_exp_identity(x in -10.0f64..10.0f64) {
        let result = run_vars("=LN(EXP(x))", vec![("x", x)]);
        prop_assert!(matches!(result, Value::Number(_)), "expected Number, got {:?}", result);
        if let Value::Number(n) = result {
            prop_assert!((n - x).abs() < 1e-9, "LN(EXP({}))={}", x, n);
        }
    }

    // 5. SIN^2 + COS^2 ≈ 1
    #[test]
    fn sin_cos_pythagorean(x in -1e4f64..1e4f64) {
        let sin_sq = run_vars("=SIN(x)*SIN(x)", vec![("x", x)]);
        let cos_sq = run_vars("=COS(x)*COS(x)", vec![("x", x)]);
        prop_assert!(matches!(sin_sq, Value::Number(_)), "expected Number for sin_sq, got {:?}", sin_sq);
        prop_assert!(matches!(cos_sq, Value::Number(_)), "expected Number for cos_sq, got {:?}", cos_sq);
        if let (Value::Number(s), Value::Number(c)) = (sin_sq, cos_sq) {
            prop_assert!((s + c - 1.0).abs() < 1e-9, "sin^2+cos^2={}", s + c);
        }
    }

    // 6. ROUND(x, 0) result is an integer (fractional part < 1e-10)
    #[test]
    fn round_zero_decimals_is_integer(x in small_f64()) {
        let result = run_vars("=ROUND(x, 0)", vec![("x", x)]);
        prop_assert!(matches!(result, Value::Number(_)), "expected Number, got {:?}", result);
        if let Value::Number(n) = result {
            prop_assert!((n - n.floor()).abs() < 1e-10, "ROUND(x,0)={} is not integer", n);
        }
    }

    // 7. INT(x) <= x for all finite x
    #[test]
    fn int_lte_x(x in finite_f64()) {
        let result = run_vars("=INT(x)", vec![("x", x)]);
        prop_assert!(matches!(result, Value::Number(_)), "expected Number, got {:?}", result);
        if let Value::Number(n) = result {
            prop_assert!(n <= x + 1e-10, "INT({})={} > x", x, n);
        }
    }

    // 8. SUM with zero: SUM(x, 0) == x
    #[test]
    fn sum_identity_zero(x in small_f64()) {
        let result = run_vars("=SUM(x, 0)", vec![("x", x)]);
        prop_assert_eq!(result, Value::Number(x));
    }

    // 9. ABS(ABS(x)) == ABS(x) — idempotent
    #[test]
    fn abs_idempotent(x in finite_f64()) {
        let abs1 = run_vars("=ABS(x)", vec![("x", x)]);
        prop_assert!(matches!(abs1, Value::Number(_)), "expected Number, got {:?}", abs1);
        if let Value::Number(a) = abs1 {
            let abs2 = run_vars("=ABS(x)", vec![("x", a)]);
            prop_assert_eq!(abs2, Value::Number(a));
        }
    }

    // 10. PRODUCT(x, 1) == x
    #[test]
    fn product_identity_one(x in small_f64()) {
        let result = run_vars("=PRODUCT(x, 1)", vec![("x", x)]);
        prop_assert_eq!(result, Value::Number(x));
    }
}

#[test]
fn abs_sanity() {
    assert_eq!(run("=ABS(-5)"), Value::Number(5.0));
}
