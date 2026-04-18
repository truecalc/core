use super::super::*;
use crate::types::Value;

#[test]
fn max_of_numbers() {
    assert_eq!(
        max_fn(&[Value::Number(1.0), Value::Number(5.0), Value::Number(3.0)]),
        Value::Number(5.0)
    );
}

#[test]
fn max_single_number() {
    assert_eq!(max_fn(&[Value::Number(7.0)]), Value::Number(7.0));
}

#[test]
fn max_bool_coerced() {
    // TRUE=1, FALSE=0; max(1, 10, 0) = 10
    assert_eq!(
        max_fn(&[Value::Bool(true), Value::Number(10.0), Value::Bool(false)]),
        Value::Number(10.0)
    );
}
