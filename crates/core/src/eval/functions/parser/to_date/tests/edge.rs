use super::super::*;
use crate::types::Value;

#[test]
fn zero_serial() {
    let args = [Value::Number(0.0)];
    assert_eq!(to_date_fn(&args), Value::Date(0.0));
}

#[test]
fn negative_serial() {
    let args = [Value::Number(-1.0)];
    assert_eq!(to_date_fn(&args), Value::Date(-1.0));
}

#[test]
fn large_serial() {
    let args = [Value::Number(50000.0)];
    assert_eq!(to_date_fn(&args), Value::Date(50000.0));
}

#[test]
fn date_value_passthrough() {
    let args = [Value::Date(1.0)];
    assert_eq!(to_date_fn(&args), Value::Date(1.0));
}
