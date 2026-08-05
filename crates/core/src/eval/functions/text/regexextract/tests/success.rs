use super::super::*;
use crate::types::Value;

#[test]
fn extract_digits() {
    assert_eq!(
        regexextract_fn(&[Value::Text("foo123bar".into()), Value::Text("[0-9]+".into())]),
        Value::Text("123".into())
    );
}

#[test]
fn extract_at_domain() {
    assert_eq!(
        regexextract_fn(&[Value::Text("hello@world.com".into()), Value::Text("@[^.]+".into())]),
        Value::Text("@world".into())
    );
}

#[test]
fn extract_year() {
    assert_eq!(
        regexextract_fn(&[Value::Text("2024-01-15".into()), Value::Text("[0-9]{4}".into())]),
        Value::Text("2024".into())
    );
}

#[test]
fn extract_full_match() {
    assert_eq!(
        regexextract_fn(&[Value::Text("abc".into()), Value::Text("[a-z]+".into())]),
        Value::Text("abc".into())
    );
}

#[test]
fn extract_first_word() {
    assert_eq!(
        regexextract_fn(&[Value::Text("hello world".into()), Value::Text("[a-z]+".into())]),
        Value::Text("hello".into())
    );
}

#[test]
fn extract_single_digit() {
    assert_eq!(
        regexextract_fn(&[Value::Text("a1b2".into()), Value::Text("[0-9]".into())]),
        Value::Text("1".into())
    );
}
