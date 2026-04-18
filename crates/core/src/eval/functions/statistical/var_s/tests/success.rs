use super::super::*;
use crate::types::Value;

#[test]
fn var_s_basic() {
    // [2, 4, 4, 4, 5, 5, 7, 9]: mean=5, sample var=4.571...
    // Using simpler: [2, 4, 6]: mean=4, var=((2-4)²+(4-4)²+(6-4)²)/(3-1)=8/2=4
    let result = var_s_fn(&[Value::Number(2.0), Value::Number(4.0), Value::Number(6.0)]);
    assert_eq!(result, Value::Number(4.0));
}

#[test]
fn var_s_two_values() {
    // [1, 3]: mean=2, var=((1-2)²+(3-2)²)/1 = 2/1 = 2
    let result = var_s_fn(&[Value::Number(1.0), Value::Number(3.0)]);
    assert_eq!(result, Value::Number(2.0));
}

#[test]
fn var_s_ignores_bool_and_text() {
    // Only numbers counted: [2, 6] → mean=4, sample var=((2-4)²+(6-4)²)/1=8
    let result = var_s_fn(&[
        Value::Number(2.0),
        Value::Number(6.0),
        Value::Bool(true),
        Value::Text("x".to_string()),
    ]);
    assert_eq!(result, Value::Number(8.0));
}
