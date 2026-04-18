use super::super::*;
use crate::types::Value;

#[test]
fn zero() {
    let args = [Value::Number(0.0)];
    assert_eq!(to_percent_fn(&args), Value::Number(0.0));
}

#[test]
fn negative_quarter() {
    let args = [Value::Number(-0.25)];
    assert_eq!(to_percent_fn(&args), Value::Number(-0.25));
}

#[test]
fn negative_one() {
    let args = [Value::Number(-1.0)];
    assert_eq!(to_percent_fn(&args), Value::Number(-1.0));
}
