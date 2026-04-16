use super::*;
use crate::types::{ErrorKind, Value};

fn run(s: &str) -> Value {
    clean_fn(&[Value::Text(s.to_string())])
}

#[test]
fn plain_text_unchanged() {
    assert_eq!(run("Hello"), Value::Text("Hello".into()));
}

#[test]
fn empty_string() {
    assert_eq!(run(""), Value::Text("".into()));
}

#[test]
fn removes_control_chars() {
    let input = format!("{}Hello", '\x01');
    assert_eq!(run(&input), Value::Text("Hello".into()));
}

#[test]
fn removes_all_control_range() {
    // Build a string with chars 1..31 prepended
    let controls: String = (1u8..32).map(|b| b as char).collect();
    let input = format!("{}ABC", controls);
    assert_eq!(run(&input), Value::Text("ABC".into()));
}

#[test]
fn space_preserved() {
    // space (32) should NOT be removed
    assert_eq!(run("hello world"), Value::Text("hello world".into()));
}

#[test]
fn no_args_returns_na() {
    assert_eq!(clean_fn(&[]), Value::Error(ErrorKind::NA));
}
