use super::super::*;
use crate::types::{ErrorKind, Value};

#[test]
fn median_ignores_text() {
    // MEDIAN(1, 2, 3, "text", 4) → 2.5 (numeric set [1,2,3,4])
    assert_eq!(
        median_fn(&[
            Value::Number(1.0),
            Value::Number(2.0),
            Value::Number(3.0),
            Value::Text("text".to_string()),
            Value::Number(4.0)
        ]),
        Value::Number(2.5)
    );
}

#[test]
fn median_ignores_bool_and_empty() {
    // Only Numbers are counted; Bool and Empty ignored
    // Numbers: 2, 4 → median 3.0
    assert_eq!(
        median_fn(&[
            Value::Bool(true),
            Value::Number(2.0),
            Value::Empty,
            Value::Number(4.0)
        ]),
        Value::Number(3.0)
    );
}

#[test]
fn median_all_non_numeric_returns_num_error() {
    assert_eq!(
        median_fn(&[Value::Text("a".to_string()), Value::Bool(true), Value::Empty]),
        Value::Error(ErrorKind::Num)
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
