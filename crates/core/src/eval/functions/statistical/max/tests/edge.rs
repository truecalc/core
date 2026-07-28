use super::super::*;
use crate::types::{ErrorKind, Value};

#[test]
fn max_no_args_returns_na() {
    assert_eq!(max_fn(&[]), Value::Error(ErrorKind::NA));
}

#[test]
fn max_text_in_args_returns_value_error() {
    assert_eq!(
        max_fn(&[Value::Text("a".to_string()), Value::Bool(true), Value::Empty]),
        Value::Error(ErrorKind::Value)
    );
}

#[test]
fn max_empty_array_is_ref_error() {
    assert_eq!(
        max_fn(&[Value::Array(vec![])]),
        Value::Error(ErrorKind::Ref)
    );
}

#[test]
fn max_non_empty_array_without_numbers_returns_zero() {
    // `=MAX({"a","b"})` and `=MAX({TRUE,FALSE})` are both 0: neither text nor
    // booleans contribute a number in array context, but both are enough to
    // lift the array out of the #REF! rule. These two variants are the whole
    // of the carve-out — nothing else sets `had_content`.
    assert_eq!(
        max_fn(&[Value::Array(vec![
            Value::Text("a".to_string()),
            Value::Text("b".to_string()),
        ])]),
        Value::Number(0.0)
    );
    assert_eq!(
        max_fn(&[Value::Array(vec![Value::Bool(true), Value::Bool(false)])]),
        Value::Number(0.0)
    );
}

#[test]
fn max_array_of_only_blanks_is_unchanged_at_ref_error() {
    // No captured row covers an all-blank array, so MAX keeps the #REF! it
    // has always given. Pinned here because narrowing the rule for text and
    // booleans must not disturb this case.
    assert_eq!(
        max_fn(&[Value::Array(vec![Value::Empty, Value::Empty, Value::Empty])]),
        Value::Error(ErrorKind::Ref)
    );
    assert_eq!(
        max_fn(&[Value::Array(vec![
            Value::Array(vec![Value::Empty]),
            Value::Array(vec![Value::Empty]),
        ])]),
        Value::Error(ErrorKind::Ref)
    );
}

#[test]
fn max_array_of_only_dates_is_unchanged_at_ref_error() {
    // `max_array_into` folds only `Value::Number`, so a date-only array has
    // never produced a result and has always answered #REF!. That is almost
    // certainly wrong against Sheets — but it is pre-existing, unprobed, and
    // must not be quietly turned into a plausible-looking 0 by the
    // text-and-boolean carve-out. `had_content` is set by text and booleans
    // only, never by a catch-all, and this pins that.
    assert_eq!(
        max_fn(&[Value::Array(vec![
            Value::Date(43831.0),
            Value::Date(44197.0)
        ])]),
        Value::Error(ErrorKind::Ref)
    );
    // Same for a date arriving through a nested-row range materialization.
    assert_eq!(
        max_fn(&[Value::Array(vec![Value::Array(vec![Value::Date(43831.0)])])]),
        Value::Error(ErrorKind::Ref)
    );
}

#[test]
fn max_one_non_blank_lifts_the_array_out_of_the_ref_rule() {
    assert_eq!(
        max_fn(&[Value::Array(vec![
            Value::Empty,
            Value::Text("z".to_string()),
        ])]),
        Value::Number(0.0)
    );
    assert_eq!(
        max_fn(&[Value::Array(vec![Value::Empty, Value::Number(4.0)])]),
        Value::Number(4.0)
    );
}

#[test]
fn max_negative_numbers() {
    assert_eq!(
        max_fn(&[Value::Number(-3.0), Value::Number(-1.0), Value::Number(-5.0)]),
        Value::Number(-1.0)
    );
}
