use super::super::*;
use crate::types::Value;

#[test]
fn avedev_basic() {
    // mean=3, deviations: |1-3|=2, |2-3|=1, |3-3|=0, |4-3|=1, |5-3|=2 → avedev=6/5=1.2
    let result = avedev_fn(&[
        Value::Number(1.0),
        Value::Number(2.0),
        Value::Number(3.0),
        Value::Number(4.0),
        Value::Number(5.0),
    ]);
    assert_eq!(result, Value::Number(1.2));
}

#[test]
fn avedev_two_values() {
    // mean=3, deviations: |1-3|=2, |5-3|=2 → avedev=2.0
    let result = avedev_fn(&[Value::Number(1.0), Value::Number(5.0)]);
    assert_eq!(result, Value::Number(2.0));
}

#[test]
fn avedev_ignores_bool_and_text() {
    // bool and text are ignored, only numbers count
    // mean=5, deviations: |5-5|=0 → avedev=0
    let result = avedev_fn(&[
        Value::Number(5.0),
        Value::Bool(true),
        Value::Text("hello".to_string()),
    ]);
    assert_eq!(result, Value::Number(0.0));
}
