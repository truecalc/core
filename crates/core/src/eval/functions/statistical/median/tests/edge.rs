use super::super::*;
use crate::types::{ErrorKind, Value};

#[test]
fn median_direct_text_returns_value_error() {
    // Direct non-parseable text → #VALUE!
    assert_eq!(
        median_fn(&[
            Value::Number(1.0),
            Value::Number(2.0),
            Value::Number(3.0),
            Value::Text("text".to_string()),
            Value::Number(4.0)
        ]),
        Value::Error(ErrorKind::Value)
    );
}

#[test]
fn median_bool_coerced_empty_skipped() {
    // Bool coerced to 1.0/0.0; Empty skipped → [1.0, 2.0, 4.0] → median=2.0
    assert_eq!(
        median_fn(&[
            Value::Bool(true),
            Value::Number(2.0),
            Value::Empty,
            Value::Number(4.0)
        ]),
        Value::Number(2.0)
    );
}

#[test]
fn median_all_non_numeric_with_text_returns_value_error() {
    // Non-parseable text → #VALUE! (not #NUM!)
    assert_eq!(
        median_fn(&[Value::Text("a".to_string()), Value::Bool(true), Value::Empty]),
        Value::Error(ErrorKind::Value)
    );
}

#[test]
fn median_unsorted_input() {
    // MEDIAN(5, 1, 3) → 3 after sorting [1, 3, 5]
    assert_eq!(
        median_fn(&[Value::Number(5.0), Value::Number(1.0), Value::Number(3.0)]),
        Value::Number(3.0)
    );
}
