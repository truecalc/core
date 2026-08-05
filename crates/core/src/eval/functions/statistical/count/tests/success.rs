use super::super::*;
use crate::types::Value;

#[test]
fn count_numbers_only() {
    assert_eq!(
        count_fn(&[Value::Number(1.0), Value::Number(2.0), Value::Number(3.0)]),
        Value::Number(3.0)
    );
}

#[test]
fn counta_all_non_empty() {
    assert_eq!(
        counta_fn(&[Value::Number(1.0), Value::Text("hi".to_string()), Value::Bool(true)]),
        Value::Number(3.0)
    );
}

#[test]
fn count_single_number() {
    assert_eq!(count_fn(&[Value::Number(42.0)]), Value::Number(1.0));
}

#[test]
fn counta_single_text() {
    assert_eq!(counta_fn(&[Value::Text("hello".to_string())]), Value::Number(1.0));
}
