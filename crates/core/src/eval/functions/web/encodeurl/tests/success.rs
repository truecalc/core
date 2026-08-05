use super::super::encodeurl_fn;
use crate::types::Value;

fn enc(s: &str) -> Value {
    encodeurl_fn(&[Value::Text(s.to_string())])
}

fn text(s: &str) -> Value {
    Value::Text(s.to_string())
}

#[test]
fn space_encodes_to_percent20() {
    assert_eq!(enc("hello world"), text("hello%20world"));
}

#[test]
fn plus_encodes_to_percent2b() {
    assert_eq!(enc("a+b"), text("a%2Bb"));
}

#[test]
fn ampersand_encodes_to_percent26() {
    assert_eq!(enc("a&b"), text("a%26b"));
}

#[test]
fn equals_encodes_to_percent3d() {
    assert_eq!(enc("a=b"), text("a%3Db"));
}

#[test]
fn slash_encodes_to_percent2f() {
    assert_eq!(enc("a/b"), text("a%2Fb"));
}

#[test]
fn unreserved_chars_pass_through() {
    let input = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789-_.~";
    assert_eq!(enc(input), text(input));
}
