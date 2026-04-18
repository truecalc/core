use super::super::*;
use crate::types::Value;

#[test]
fn replace_word() {
    assert_eq!(
        regexreplace_fn(&[
            Value::Text("hello world".into()),
            Value::Text("world".into()),
            Value::Text("earth".into()),
        ]),
        Value::Text("hello earth".into())
    );
}

#[test]
fn replace_all_matches() {
    assert_eq!(
        regexreplace_fn(&[
            Value::Text("aabbcc".into()),
            Value::Text("[bc]+".into()),
            Value::Text("X".into()),
        ]),
        Value::Text("aaX".into())
    );
}

#[test]
fn replace_vowels() {
    assert_eq!(
        regexreplace_fn(&[
            Value::Text("hello".into()),
            Value::Text("[aeiou]".into()),
            Value::Text("*".into()),
        ]),
        Value::Text("h*ll*".into())
    );
}

#[test]
fn replace_digits() {
    assert_eq!(
        regexreplace_fn(&[
            Value::Text("abc123".into()),
            Value::Text("[0-9]+".into()),
            Value::Text("NUM".into()),
        ]),
        Value::Text("abcNUM".into())
    );
}

#[test]
fn no_match_returns_unchanged() {
    assert_eq!(
        regexreplace_fn(&[
            Value::Text("hello".into()),
            Value::Text("z".into()),
            Value::Text("X".into()),
        ]),
        Value::Text("hello".into())
    );
}
