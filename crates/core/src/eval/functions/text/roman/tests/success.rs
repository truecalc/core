use super::super::*;
use crate::types::Value;

fn run(n: f64) -> Value {
    roman_fn(&[Value::Number(n)])
}

#[test]
fn fourteen() {
    assert_eq!(run(14.0), Value::Text("XIV".into()));
}

#[test]
fn mcmxcix() {
    assert_eq!(run(1999.0), Value::Text("MCMXCIX".into()));
}

#[test]
fn one() {
    assert_eq!(run(1.0), Value::Text("I".into()));
}

#[test]
fn four() {
    assert_eq!(run(4.0), Value::Text("IV".into()));
}

#[test]
fn max_value() {
    assert_eq!(run(3999.0), Value::Text("MMMCMXCIX".into()));
}

#[test]
fn four_hundred() {
    assert_eq!(run(400.0), Value::Text("CD".into()));
}
