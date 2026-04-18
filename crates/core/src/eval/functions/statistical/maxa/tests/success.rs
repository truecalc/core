use super::super::maxa_fn;
use crate::types::Value;

#[test]
fn max_of_numbers() {
    assert_eq!(
        maxa_fn(&[Value::Number(1.0), Value::Number(2.0), Value::Number(3.0)]),
        Value::Number(3.0)
    );
}

#[test]
fn true_coerced_to_1() {
    // TRUE=1, so max(1, 2.0, 3.0) = 3.0
    assert_eq!(
        maxa_fn(&[Value::Bool(true), Value::Number(2.0), Value::Number(3.0)]),
        Value::Number(3.0)
    );
}

#[test]
fn false_coerced_to_0() {
    // FALSE=0 alongside 0.0 → max(0, 0) = 0.0
    assert_eq!(
        maxa_fn(&[Value::Bool(false), Value::Number(0.0)]),
        Value::Number(0.0)
    );
}

#[test]
fn true_wins_over_negatives() {
    // TRUE=1 > -5.0
    assert_eq!(
        maxa_fn(&[Value::Bool(true), Value::Number(-5.0)]),
        Value::Number(1.0)
    );
}
