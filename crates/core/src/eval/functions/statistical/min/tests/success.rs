use super::super::*;
use crate::types::Value;

#[test]
fn min_of_numbers() {
    assert_eq!(
        min_fn(&[Value::Number(5.0), Value::Number(1.0), Value::Number(3.0)]),
        Value::Number(1.0)
    );
}

#[test]
fn min_single_number() {
    assert_eq!(min_fn(&[Value::Number(7.0)]), Value::Number(7.0));
}

#[test]
fn min_bool_coerced() {
    // TRUE=1, FALSE=0; min(1, 2, 0) = 0
    assert_eq!(
        min_fn(&[Value::Bool(true), Value::Number(2.0), Value::Bool(false)]),
        Value::Number(0.0)
    );
}
