use super::super::*;
use crate::types::Value;

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
    let controls: String = (1u8..32).map(|b| b as char).collect();
    let input = format!("{}ABC", controls);
    assert_eq!(run(&input), Value::Text("ABC".into()));
}

#[test]
fn space_preserved() {
    assert_eq!(run("hello world"), Value::Text("hello world".into()));
}
