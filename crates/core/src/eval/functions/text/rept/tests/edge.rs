use super::super::*;
use crate::types::Value;

#[test]
fn zero_times() {
    assert_eq!(
        rept_fn(&[Value::Text("x".to_string()), Value::Number(0.0)]),
        Value::Text("".to_string())
    );
}

#[test]
fn empty_text_repeated() {
    assert_eq!(
        rept_fn(&[Value::Text("".to_string()), Value::Number(5.0)]),
        Value::Text("".to_string())
    );
}

#[test]
fn fractional_times_truncated() {
    assert_eq!(
        rept_fn(&[Value::Text("ab".to_string()), Value::Number(2.9)]),
        Value::Text("abab".to_string())
    );
}
