use super::super::mina_fn;
use crate::types::{ErrorKind, Value};

#[test]
fn empty_values_skipped() {
    // Empty is skipped; min of remaining values
    assert_eq!(
        mina_fn(&[Value::Empty, Value::Number(7.0), Value::Empty]),
        Value::Number(7.0)
    );
}

#[test]
fn single_true_returns_one() {
    // Only TRUE=1 → result is 1.0
    assert_eq!(mina_fn(&[Value::Bool(true)]), Value::Number(1.0));
}

#[test]
fn negative_numbers_min() {
    assert_eq!(
        mina_fn(&[Value::Number(-3.0), Value::Number(-1.0), Value::Number(-5.0)]),
        Value::Number(-5.0)
    );
}

#[test]
fn bool_and_number_mixed() {
    // TRUE=1 and FALSE=0 mixed with numbers: min(1, 0, 10, 0) = 0
    assert_eq!(
        mina_fn(&[
            Value::Bool(true),
            Value::Bool(false),
            Value::Number(10.0),
            Value::Number(0.0)
        ]),
        Value::Number(0.0)
    );
}

#[test]
fn mina_empty_array_is_ref_error() {
    // `=MINA({})` is #REF!, as it is for MIN and MAX. Reached by the
    // empty-argument check alone: text folds in as 0 here, so a populated
    // array is never numberless in the first place.
    assert_eq!(mina_fn(&[Value::Array(vec![])]), Value::Error(ErrorKind::Ref));
    assert_eq!(
        mina_fn(&[Value::Number(1.0), Value::Array(vec![])]),
        Value::Error(ErrorKind::Ref)
    );
}

#[test]
fn mina_text_only_array_is_zero() {
    // `=MINA({"a","b"})` is 0 — text counts as zero rather than being
    // skipped, so this needs no separate rule.
    assert_eq!(
        mina_fn(&[Value::Array(vec![
            Value::Text("a".to_string()),
            Value::Text("b".to_string()),
        ])]),
        Value::Number(0.0)
    );
}
