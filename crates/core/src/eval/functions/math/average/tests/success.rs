use super::super::*;
use crate::types::Value;

#[test]
fn average_of_three() {
    assert_eq!(
        average_fn(&[Value::Number(1.0), Value::Number(2.0), Value::Number(3.0)]),
        Value::Number(2.0)
    );
}

#[test]
fn average_single() {
    assert_eq!(average_fn(&[Value::Number(5.0)]), Value::Number(5.0));
}

#[test]
fn average_with_bool() {
    // TRUE=1, FALSE=0 → (1+0)/2 = 0.5
    assert_eq!(
        average_fn(&[Value::Bool(true), Value::Bool(false)]),
        Value::Number(0.5)
    );
}

#[test]
fn average_with_numeric_text() {
    assert_eq!(
        average_fn(&[Value::Text("4.0".to_string()), Value::Number(2.0)]),
        Value::Number(3.0)
    );
}
