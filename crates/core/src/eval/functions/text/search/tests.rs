use super::*;
use crate::types::{ErrorKind, Value};

fn search(find: &str, within: &str) -> Value {
    search_fn(&[Value::Text(find.into()), Value::Text(within.into())])
}

fn search_from(find: &str, within: &str, start: f64) -> Value {
    search_fn(&[
        Value::Text(find.into()),
        Value::Text(within.into()),
        Value::Number(start),
    ])
}

#[test]
fn basic_found() {
    assert_eq!(search("o", "Hello World"), Value::Number(5.0));
}

#[test]
fn case_insensitive() {
    assert_eq!(search("WORLD", "Hello World"), Value::Number(7.0));
    assert_eq!(search("h", "Hello"), Value::Number(1.0));
}

#[test]
fn question_mark_wildcard() {
    // h?llo matches Hello at position 1
    assert_eq!(search("h?llo", "Hello World"), Value::Number(1.0));
}

#[test]
fn star_wildcard() {
    // h*o matches "Hello" ... "o" at position 1
    assert_eq!(search("h*o", "Hello World"), Value::Number(1.0));
}

#[test]
fn start_position() {
    // second 'o' in "Hello World" is at position 8
    assert_eq!(search_from("o", "Hello World", 6.0), Value::Number(8.0));
}

#[test]
fn not_found_returns_value_error() {
    assert_eq!(search("z", "Hello"), Value::Error(ErrorKind::Value));
}

#[test]
fn start_pos_out_of_range() {
    assert_eq!(search_from("o", "Hello", 100.0), Value::Error(ErrorKind::Value));
}

#[test]
fn no_args_returns_na() {
    assert_eq!(search_fn(&[]), Value::Error(ErrorKind::NA));
}

#[test]
fn trailing_star_wildcard() {
    assert_eq!(search("*orld", "Hello World"), Value::Number(1.0));
}

#[test]
fn coercion_number_as_find_text() {
    // SEARCH(4, "abc123456") -> find "4" in the string
    let r = search_fn(&[Value::Number(4.0), Value::Text("abc123456".into())]);
    // "4" is at position 7 (a=1,b=2,c=3,1=4,2=5,3=6,4=7)
    assert_eq!(r, Value::Number(7.0));
}
