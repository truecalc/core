use super::super::*;
use crate::types::Value;

#[test]
fn devsq_single_value_returns_zero() {
    // Single value: mean=7, devsq=(7-7)²=0
    assert_eq!(devsq_fn(&[Value::Number(7.0)]), Value::Number(0.0));
}

#[test]
fn devsq_all_same_values_returns_zero() {
    assert_eq!(
        devsq_fn(&[Value::Number(4.0), Value::Number(4.0), Value::Number(4.0)]),
        Value::Number(0.0)
    );
}

#[test]
fn devsq_negative_numbers() {
    // mean=-3, devsq=(-5-(-3))²+(-1-(-3))²=4+4=8
    let result = devsq_fn(&[Value::Number(-5.0), Value::Number(-1.0)]);
    assert_eq!(result, Value::Number(8.0));
}
