use super::super::*;
use crate::types::Value;

#[test]
fn devsq_basic() {
    // mean=3, deviations squared: (1-3)²=4, (2-3)²=1, (3-3)²=0, (4-3)²=1, (5-3)²=4 → devsq=10
    let result = devsq_fn(&[
        Value::Number(1.0),
        Value::Number(2.0),
        Value::Number(3.0),
        Value::Number(4.0),
        Value::Number(5.0),
    ]);
    assert_eq!(result, Value::Number(10.0));
}

#[test]
fn devsq_two_values() {
    // mean=3, deviations squared: (1-3)²=4, (5-3)²=4 → devsq=8
    let result = devsq_fn(&[Value::Number(1.0), Value::Number(5.0)]);
    assert_eq!(result, Value::Number(8.0));
}

#[test]
fn devsq_ignores_bool_and_text() {
    // bool and text are ignored
    let result = devsq_fn(&[
        Value::Number(2.0),
        Value::Number(4.0),
        Value::Bool(true),
        Value::Text("x".to_string()),
    ]);
    // mean=3, devsq=(2-3)²+(4-3)²=2
    assert_eq!(result, Value::Number(2.0));
}
