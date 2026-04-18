use super::super::*;
use crate::types::Value;

fn run1(n: f64) -> Value {
    fixed_fn(&[Value::Number(n)])
}

fn run2(n: f64, decimals: f64) -> Value {
    fixed_fn(&[Value::Number(n), Value::Number(decimals)])
}

fn run3(n: f64, decimals: f64, no_commas: bool) -> Value {
    fixed_fn(&[Value::Number(n), Value::Number(decimals), Value::Bool(no_commas)])
}

#[test]
fn default_two_decimals_with_commas() {
    assert_eq!(run1(1234.567), Value::Text("1,234.57".into()));
}

#[test]
fn three_decimals() {
    assert_eq!(run2(1234.567, 3.0), Value::Text("1,234.567".into()));
}

#[test]
fn zero_decimals_rounds() {
    assert_eq!(run2(1234.567, 0.0), Value::Text("1,235".into()));
}

#[test]
fn no_commas_flag() {
    assert_eq!(run3(1234.567, 2.0, true), Value::Text("1234.57".into()));
}

#[test]
fn negative_number() {
    assert_eq!(run1(-1234.567), Value::Text("-1,234.57".into()));
}
