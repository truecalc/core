use super::super::*;
use crate::types::{ErrorKind, Value};

#[test]
fn kurt_less_than_4_returns_div0() {
    assert_eq!(
        kurt_fn(&[Value::Number(1.0), Value::Number(2.0), Value::Number(3.0)]),
        Value::Error(ErrorKind::DivByZero)
    );
}

#[test]
fn kurt_all_same_values_returns_div0() {
    // s = 0
    assert_eq!(
        kurt_fn(&[
            Value::Number(5.0),
            Value::Number(5.0),
            Value::Number(5.0),
            Value::Number(5.0)
        ]),
        Value::Error(ErrorKind::DivByZero)
    );
}
