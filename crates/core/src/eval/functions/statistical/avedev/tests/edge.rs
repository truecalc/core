use super::super::*;
use crate::types::Value;

#[test]
fn avedev_single_value_returns_zero() {
    // Single value: mean=7, |7-7|=0 → avedev=0
    assert_eq!(avedev_fn(&[Value::Number(7.0)]), Value::Number(0.0));
}

#[test]
fn avedev_all_same_values_returns_zero() {
    assert_eq!(
        avedev_fn(&[Value::Number(4.0), Value::Number(4.0), Value::Number(4.0)]),
        Value::Number(0.0)
    );
}

#[test]
fn avedev_negative_numbers() {
    // mean=-3, deviations: |-5-(-3)|=2, |-1-(-3)|=2 → avedev=2.0
    let result = avedev_fn(&[Value::Number(-5.0), Value::Number(-1.0)]);
    assert_eq!(result, Value::Number(2.0));
}
