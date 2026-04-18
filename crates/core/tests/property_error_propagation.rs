// crates/core/tests/property_error_propagation.rs
//
// Verifies that non-error-handling functions propagate error values rather
// than silently resolving them. This is a correctness invariant derived from
// Google Sheets behavior: errors are contagious.

use ganit_core::{evaluate, ErrorKind, Value};
use proptest::prelude::*;
use std::collections::HashMap;

fn error_value() -> impl Strategy<Value = Value> {
    prop_oneof![
        Just(Value::Error(ErrorKind::Value)),
        Just(Value::Error(ErrorKind::NA)),
        Just(Value::Error(ErrorKind::DivByZero)),
        Just(Value::Error(ErrorKind::Num)),
    ]
}

fn run_with_error(formula: &str, var: &str, err: Value) -> Value {
    let mut vars = HashMap::new();
    vars.insert(var.to_string(), err);
    evaluate(formula, &vars)
}

fn is_error(v: &Value) -> bool {
    matches!(v, Value::Error(_))
}

proptest! {
    // Math functions propagate errors
    #[test]
    fn abs_propagates_error(e in error_value()) {
        let result = run_with_error("=ABS(x)", "x", e);
        prop_assert!(is_error(&result), "ABS did not propagate error, got {:?}", result);
    }

    #[test]
    fn sqrt_propagates_error(e in error_value()) {
        let result = run_with_error("=SQRT(x)", "x", e);
        prop_assert!(is_error(&result), "SQRT did not propagate error, got {:?}", result);
    }

    #[test]
    fn ln_propagates_error(e in error_value()) {
        let result = run_with_error("=LN(x)", "x", e);
        prop_assert!(is_error(&result), "LN did not propagate error, got {:?}", result);
    }

    #[test]
    fn exp_propagates_error(e in error_value()) {
        let result = run_with_error("=EXP(x)", "x", e);
        prop_assert!(is_error(&result), "EXP did not propagate error, got {:?}", result);
    }

    #[test]
    fn round_propagates_error(e in error_value()) {
        let result = run_with_error("=ROUND(x, 2)", "x", e);
        prop_assert!(is_error(&result), "ROUND did not propagate error, got {:?}", result);
    }

    // Text functions propagate errors
    #[test]
    fn len_propagates_error(e in error_value()) {
        let result = run_with_error("=LEN(x)", "x", e);
        prop_assert!(is_error(&result), "LEN did not propagate error, got {:?}", result);
    }

    #[test]
    fn upper_propagates_error(e in error_value()) {
        let result = run_with_error("=UPPER(x)", "x", e);
        prop_assert!(is_error(&result), "UPPER did not propagate error, got {:?}", result);
    }

    #[test]
    fn lower_propagates_error(e in error_value()) {
        let result = run_with_error("=LOWER(x)", "x", e);
        prop_assert!(is_error(&result), "LOWER did not propagate error, got {:?}", result);
    }

    #[test]
    fn trim_propagates_error(e in error_value()) {
        let result = run_with_error("=TRIM(x)", "x", e);
        prop_assert!(is_error(&result), "TRIM did not propagate error, got {:?}", result);
    }
}
