use super::super::*;
use crate::types::{ErrorKind, Value};

#[test]
fn min_no_args_returns_na() {
    assert_eq!(min_fn(&[]), Value::Error(ErrorKind::NA));
}

#[test]
fn min_text_in_args_returns_value_error() {
    assert_eq!(
        min_fn(&[Value::Text("a".to_string()), Value::Bool(true), Value::Empty]),
        Value::Error(ErrorKind::Value)
    );
}

#[test]
fn min_empty_array_is_ref_error() {
    // Matches MAX and Google Sheets: an empty array argument is #REF!,
    // not a silent 0.
    assert_eq!(
        min_fn(&[Value::Array(vec![])]),
        Value::Error(ErrorKind::Ref)
    );
}

#[test]
fn min_empty_array_beside_a_number_is_ref_error() {
    assert_eq!(
        min_fn(&[Value::Number(1.0), Value::Array(vec![])]),
        Value::Error(ErrorKind::Ref)
    );
    assert_eq!(
        min_fn(&[Value::Array(vec![]), Value::Number(1.0)]),
        Value::Error(ErrorKind::Ref)
    );
}

#[test]
fn min_non_empty_array_without_numbers_still_returns_zero() {
    // Distinct from the absent-argument rule: the fixtures pin
    // `=MIN({"a","b"})` and `=IFERROR(MIN({"a","b","c"}),"no numbers")` to
    // the number 0, so a populated-but-numberless array must not become #REF!.
    assert_eq!(
        min_fn(&[Value::Array(vec![
            Value::Text("a".to_string()),
            Value::Text("b".to_string()),
        ])]),
        Value::Number(0.0)
    );
    // Booleans are skipped in array context but still make the array present.
    assert_eq!(
        min_fn(&[Value::Array(vec![Value::Bool(true), Value::Bool(false)])]),
        Value::Number(0.0)
    );
}

#[test]
fn min_array_of_only_blanks_is_unchanged_at_zero() {
    // No captured row covers an all-blank array, so MIN keeps the 0 it has
    // always given. Pinned here so a future change to it has to be deliberate
    // rather than a side effect of the empty-array rule above.
    assert_eq!(
        min_fn(&[Value::Array(vec![Value::Empty, Value::Empty, Value::Empty])]),
        Value::Number(0.0)
    );
    assert_eq!(
        min_fn(&[Value::Array(vec![
            Value::Array(vec![Value::Empty]),
            Value::Array(vec![Value::Empty]),
        ])]),
        Value::Number(0.0)
    );
}

#[test]
fn min_negative_numbers() {
    assert_eq!(
        min_fn(&[Value::Number(-3.0), Value::Number(-1.0), Value::Number(-5.0)]),
        Value::Number(-5.0)
    );
}
