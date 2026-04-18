use super::super::*;
use crate::types::Value;

#[test]
fn zero() {
    let args = [Value::Number(0.0)];
    assert_eq!(to_dollars_fn(&args), Value::Number(0.0));
}

#[test]
fn negative_integer() {
    let args = [Value::Number(-5.0)];
    assert_eq!(to_dollars_fn(&args), Value::Number(-5.0));
}

#[test]
fn negative_decimal() {
    let args = [Value::Number(-0.99)];
    assert_eq!(to_dollars_fn(&args), Value::Number(-0.99));
}
