use super::super::maxa_fn;
use crate::types::Value;

#[test]
fn empty_values_skipped() {
    // Empty is skipped; max of remaining values
    assert_eq!(
        maxa_fn(&[Value::Empty, Value::Number(7.0), Value::Empty]),
        Value::Number(7.0)
    );
}

#[test]
fn single_false_returns_zero() {
    // Only FALSE=0 → result is 0.0
    assert_eq!(maxa_fn(&[Value::Bool(false)]), Value::Number(0.0));
}

#[test]
fn negative_numbers_max() {
    assert_eq!(
        maxa_fn(&[Value::Number(-3.0), Value::Number(-1.0), Value::Number(-5.0)]),
        Value::Number(-1.0)
    );
}

#[test]
fn bool_and_number_mixed() {
    // TRUE=1 and FALSE=0 mixed with numbers: max(1, 0, 10, 0) = 10
    assert_eq!(
        maxa_fn(&[
            Value::Bool(true),
            Value::Bool(false),
            Value::Number(10.0),
            Value::Number(0.0)
        ]),
        Value::Number(10.0)
    );
}
