use super::super::*;
use crate::types::Value;

fn run1(n: f64) -> Value {
    dollar_fn(&[Value::Number(n)])
}

fn run2(n: f64, decimals: f64) -> Value {
    dollar_fn(&[Value::Number(n), Value::Number(decimals)])
}

#[test]
fn default_two_decimals() {
    assert_eq!(run1(1234.567), Value::Text("$1,234.57".into()));
}

#[test]
fn three_decimals() {
    assert_eq!(run2(1234.567, 3.0), Value::Text("$1,234.567".into()));
}

#[test]
fn zero_decimals() {
    assert_eq!(run2(1234.567, 0.0), Value::Text("$1,235".into()));
}

#[test]
fn negative_number() {
    assert_eq!(run1(-1234.567), Value::Text("-$1,234.57".into()));
}

#[test]
fn zero() {
    assert_eq!(run1(0.0), Value::Text("$0.00".into()));
}

#[test]
fn half() {
    assert_eq!(run1(0.5), Value::Text("$0.50".into()));
}

#[test]
fn million() {
    assert_eq!(run1(1_000_000.0), Value::Text("$1,000,000.00".into()));
}

#[test]
fn negative_decimals() {
    assert_eq!(run2(1234.567, -1.0), Value::Text("$1,230".into()));
}

#[test]
fn one_dollar_len_five() {
    if let Value::Text(s) = run1(1.0) {
        assert_eq!(s.len(), 5, "expected '$1.00' (5 chars), got: {}", s);
    } else {
        panic!("expected Text");
    }
}
