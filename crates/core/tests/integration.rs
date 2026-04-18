mod helpers;
use truecalc_core::{Value, ErrorKind};
use helpers::{eval, eval_with};

// 1. Basic arithmetic
#[test]
fn arithmetic_add() {
    assert_eq!(eval("=1+2"), Value::Number(3.0));
}

// 2. Variable resolution
#[test]
fn variable_resolution() {
    assert_eq!(eval_with("=A1+B1", [("A1", Value::Number(5.0)), ("B1", Value::Number(3.0))]),
               Value::Number(8.0));
}

// 3. SUM function
#[test]
fn sum_function() {
    assert_eq!(eval("=SUM(1,2,3)"), Value::Number(6.0));
}

// 4. IF function — lazy evaluation
#[test]
fn if_function_true_branch() {
    assert_eq!(eval("=IF(TRUE,\"yes\",\"no\")"), Value::Text("yes".to_string()));
}

// 5. IF function — false branch
#[test]
fn if_function_false_branch() {
    assert_eq!(eval("=IF(FALSE,\"yes\",\"no\")"), Value::Text("no".to_string()));
}

// 6. Division by zero
#[test]
fn division_by_zero() {
    assert_eq!(eval("=1/0"), Value::Error(ErrorKind::DivByZero));
}

// 7. Unknown variable returns Empty
#[test]
fn unknown_variable_returns_empty() {
    assert_eq!(eval("=Z99"), Value::Empty);
}

// 8. Unknown function returns #NAME?
#[test]
fn unknown_function_returns_name_error() {
    assert_eq!(eval("=NONEXISTENT(1)"), Value::Error(ErrorKind::Name));
}

// 9. String concatenation
#[test]
fn string_concatenation() {
    assert_eq!(eval("=\"hello\"&\" world\""), Value::Text("hello world".to_string()));
}

// 10. Comparison — equal
#[test]
fn comparison_equal() {
    assert_eq!(eval("=1=1"), Value::Bool(true));
    assert_eq!(eval("=1=2"), Value::Bool(false));
}

// 11. Percent operator
#[test]
fn percent_operator() {
    assert_eq!(eval("=50%"), Value::Number(0.5));
}

// 12. Nested function call: IF(SUM(0,0)=0, "empty", "not")
#[test]
fn nested_if_with_sum() {
    assert_eq!(eval("=IF(SUM(0,0)=0,\"empty\",\"not\")"), Value::Text("empty".to_string()));
}

// 13. Parse error returns #VALUE!
#[test]
fn invalid_formula_returns_value_error() {
    assert_eq!(eval("=("), Value::Error(ErrorKind::Value));
}

use proptest::prelude::*;

proptest! {
    #[test]
    fn display_number_parses_back(n in -1e9f64..=1e9f64) {
        // display_number should produce a string that, when parsed as f64,
        // round-trips with relative tolerance 1e-9 and absolute floor 1e-12
        // (consistent with display_number's 14 significant digits of precision).
        let s = truecalc_core::display_number(n);
        // If it is a valid display string (not "#NUM!"), it should parse back.
        if !s.starts_with('#') {
            let parsed: f64 = s.parse().expect("display_number output should be parseable");
            prop_assert!((parsed - n).abs() <= n.abs() * 1e-9 + 1e-12,
                "round-trip failed for n={n}, displayed as {s:?}, parsed back as {parsed}");
        }
    }
}

#[test]
fn display_number_nan_returns_num_error() {
    assert_eq!(truecalc_core::display_number(f64::NAN), "#NUM!");
}

#[test]
fn display_number_infinity_returns_num_error() {
    assert_eq!(truecalc_core::display_number(f64::INFINITY), "#NUM!");
}

// Temporary debug test for UNIQUE
#[test]
fn debug_unique_counta() {
    let result = helpers::eval("=UNIQUE({1,2,2,3,3,3},FALSE,FALSE)");
    println!("UNIQUE result: {:?}", result);
    
    let result2 = helpers::eval("=COUNTA(UNIQUE({1,2,2,3,3,3},FALSE,FALSE))");
    println!("COUNTA(UNIQUE) result: {:?}", result2);
}

#[test]
fn debug_counta_array() {
    // Test COUNTA with a direct array
    let result = helpers::eval("=COUNTA({1,2,2,3,3,3})");
    println!("COUNTA direct array result: {:?}", result);
}
