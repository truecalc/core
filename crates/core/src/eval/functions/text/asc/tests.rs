use super::*;
use crate::types::{ErrorKind, Value};

fn run(s: &str) -> Value {
    asc_fn(&[Value::Text(s.to_string())])
}

#[test]
fn ascii_unchanged() {
    assert_eq!(run("Hello"), Value::Text("Hello".into()));
    assert_eq!(run("ABC"), Value::Text("ABC".into()));
    assert_eq!(run("hello"), Value::Text("hello".into()));
    assert_eq!(run(" "), Value::Text(" ".into()));
    assert_eq!(run(""), Value::Text("".into()));
}

#[test]
fn numeric_string_unchanged() {
    assert_eq!(run("123"), Value::Text("123".into()));
}

#[test]
fn fullwidth_ascii_converted() {
    // U+FF21 'Ａ' → 'A'
    let fullwidth = "\u{FF21}\u{FF22}\u{FF23}";
    assert_eq!(asc_fn(&[Value::Text(fullwidth.into())]), Value::Text("ABC".into()));
}

#[test]
fn ideographic_space_converted() {
    // U+3000 → U+0020
    let s = "\u{3000}";
    assert_eq!(asc_fn(&[Value::Text(s.into())]), Value::Text(" ".into()));
}

#[test]
fn no_args_returns_na() {
    assert_eq!(asc_fn(&[]), Value::Error(ErrorKind::NA));
}

#[test]
fn too_many_args_returns_na() {
    let r = asc_fn(&[Value::Text("a".into()), Value::Text("b".into())]);
    assert_eq!(r, Value::Error(ErrorKind::NA));
}
