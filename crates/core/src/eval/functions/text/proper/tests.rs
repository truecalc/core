use super::*;
use crate::types::Value;

fn run(s: &str) -> Value {
    proper_fn(&[Value::Text(s.to_string())])
}

#[test]
fn basic_lowercase() {
    assert_eq!(run("hello world"), Value::Text("Hello World".into()));
}

#[test]
fn all_uppercase() {
    assert_eq!(run("HELLO WORLD"), Value::Text("Hello World".into()));
}

#[test]
fn mixed_case() {
    assert_eq!(run("hElLo WoRlD"), Value::Text("Hello World".into()));
}

#[test]
fn empty_string() {
    assert_eq!(run(""), Value::Text("".into()));
}

#[test]
fn apostrophe_boundary() {
    assert_eq!(run("john's car"), Value::Text("John'S Car".into()));
}

#[test]
fn digit_boundary() {
    assert_eq!(run("hello2world"), Value::Text("Hello2World".into()));
}

#[test]
fn hyphen_boundary() {
    assert_eq!(run("mary-ann"), Value::Text("Mary-Ann".into()));
}

#[test]
fn coercion_number() {
    let result = proper_fn(&[Value::Number(123.0)]);
    assert_eq!(result, Value::Text("123".into()));
}

#[test]
fn coercion_true() {
    let result = proper_fn(&[Value::Bool(true)]);
    assert_eq!(result, Value::Text("True".into()));
}

#[test]
fn coercion_false() {
    let result = proper_fn(&[Value::Bool(false)]);
    assert_eq!(result, Value::Text("False".into()));
}

#[test]
fn no_args_returns_na() {
    use crate::types::ErrorKind;
    assert_eq!(proper_fn(&[]), Value::Error(ErrorKind::NA));
}

#[test]
fn too_many_args_returns_na() {
    use crate::types::ErrorKind;
    let r = proper_fn(&[Value::Text("a".into()), Value::Text("b".into())]);
    assert_eq!(r, Value::Error(ErrorKind::NA));
}
