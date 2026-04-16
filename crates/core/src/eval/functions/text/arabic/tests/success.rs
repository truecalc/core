use super::super::*;
use crate::types::Value;

fn run(s: &str) -> Value {
    arabic_fn(&[Value::Text(s.to_string())])
}

#[test]
fn basic_xiv() {
    assert_eq!(run("XIV"), Value::Number(14.0));
}

#[test]
fn mcmxcix() {
    assert_eq!(run("MCMXCIX"), Value::Number(1999.0));
}

#[test]
fn single_i() {
    assert_eq!(run("I"), Value::Number(1.0));
}

#[test]
fn iv() {
    assert_eq!(run("IV"), Value::Number(4.0));
}

#[test]
fn xl() {
    assert_eq!(run("XL"), Value::Number(40.0));
}

#[test]
fn cd() {
    assert_eq!(run("CD"), Value::Number(400.0));
}

#[test]
fn mmmcmxcix() {
    assert_eq!(run("MMMCMXCIX"), Value::Number(3999.0));
}

#[test]
fn empty_returns_zero() {
    assert_eq!(run(""), Value::Number(0.0));
}

#[test]
fn lowercase_input() {
    assert_eq!(run("xiv"), Value::Number(14.0));
}
