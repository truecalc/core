// crates/core/tests/property_lookup.rs
//
// Property-based tests for lookup functions.
// Verifies invariants: out-of-range inputs produce errors, in-range inputs
// produce values within the searched set.

use ganit_core::{evaluate, Value};
use proptest::prelude::*;
use std::collections::HashMap;

fn run(formula: &str) -> Value {
    evaluate(formula, &HashMap::new())
}

fn is_error(v: &Value) -> bool {
    matches!(v, Value::Error(_))
}

proptest! {
    // CHOOSE(idx, ...) with idx out of range [1, n] returns an error
    #[test]
    fn choose_out_of_range_errors(
        n_choices in 1usize..=5,
        idx_offset in 1usize..=10,
    ) {
        let n = n_choices;
        let bad_idx = n + idx_offset; // always > n, always out of range
        let choices = (1..=n).map(|i| i.to_string()).collect::<Vec<_>>().join(", ");
        let formula = format!("=CHOOSE({}, {})", bad_idx, choices);
        let result = run(&formula);
        prop_assert!(is_error(&result),
            "CHOOSE({}, {} choices) should error but got {:?}", bad_idx, n, result);
    }

    // CHOOSE(idx, ...) with idx in [1, n] returns one of the choices (a Number)
    #[test]
    fn choose_in_range_returns_value(
        n_choices in 1usize..=5,
        idx_minus_one in 0usize..5,
    ) {
        let n = n_choices;
        let idx = (idx_minus_one % n) + 1; // always in [1, n]
        let choices = (1..=n).map(|i| i.to_string()).collect::<Vec<_>>().join(", ");
        let formula = format!("=CHOOSE({}, {})", idx, choices);
        let result = run(&formula);
        prop_assert!(matches!(result, Value::Number(_)),
            "CHOOSE({}, {} choices) should return Number but got {:?}", idx, n, result);
        if let Value::Number(v) = result {
            prop_assert_eq!(v, idx as f64,
                "CHOOSE({}) returned {} instead of {}", idx, v, idx);
        }
    }
}
