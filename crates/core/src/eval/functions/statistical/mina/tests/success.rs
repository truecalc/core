use super::super::mina_fn;
use crate::types::Value;

#[test]
fn min_of_numbers() {
    assert_eq!(
        mina_fn(&[Value::Number(1.0), Value::Number(2.0), Value::Number(3.0)]),
        Value::Number(1.0)
    );
}

#[test]
fn true_coerced_to_1() {
    // TRUE=1, so min(1, 2.0, 3.0) = 1.0
    assert_eq!(
        mina_fn(&[Value::Bool(true), Value::Number(2.0), Value::Number(3.0)]),
        Value::Number(1.0)
    );
}

#[test]
fn false_coerced_to_0() {
    // FALSE=0 < 1.0 → min(0, 1) = 0.0
    assert_eq!(
        mina_fn(&[Value::Bool(false), Value::Number(1.0)]),
        Value::Number(0.0)
    );
}

#[test]
fn false_loses_to_negatives() {
    // FALSE=0, but -5.0 < 0 → min = -5.0
    assert_eq!(
        mina_fn(&[Value::Bool(false), Value::Number(-5.0)]),
        Value::Number(-5.0)
    );
}
