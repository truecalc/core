use super::super::*;
use crate::types::Value;

#[test]
fn matches_digits() {
    assert_eq!(
        regexmatch_fn(&[Value::Text("hello123".into()), Value::Text("[0-9]+".into())]),
        Value::Bool(true)
    );
}

#[test]
fn no_match_returns_false() {
    assert_eq!(
        regexmatch_fn(&[Value::Text("hello".into()), Value::Text("[0-9]+".into())]),
        Value::Bool(false)
    );
}

#[test]
fn partial_match_at_symbol() {
    assert_eq!(
        regexmatch_fn(&[Value::Text("test@email.com".into()), Value::Text("@".into())]),
        Value::Bool(true)
    );
}
