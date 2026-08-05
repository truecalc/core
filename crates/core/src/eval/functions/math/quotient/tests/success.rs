use super::super::*;
use crate::types::Value;

#[test]
fn quotient_basic() {
    assert_eq!(
        quotient_fn(&[Value::Number(5.0), Value::Number(2.0)]),
        Value::Number(2.0)
    );
}

#[test]
fn quotient_exact() {
    assert_eq!(
        quotient_fn(&[Value::Number(6.0), Value::Number(3.0)]),
        Value::Number(2.0)
    );
}

#[test]
fn quotient_with_negatives() {
    assert_eq!(
        quotient_fn(&[Value::Number(-7.0), Value::Number(2.0)]),
        Value::Number(-3.0)
    );
}
