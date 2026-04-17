use super::super::isurl_fn;
use crate::types::Value;

#[test]
fn https_url_returns_true() {
    assert_eq!(
        isurl_fn(&[Value::Text("https://example.com".to_string())]),
        Value::Bool(true)
    );
}

#[test]
fn http_url_returns_true() {
    assert_eq!(
        isurl_fn(&[Value::Text("http://foo.bar".to_string())]),
        Value::Bool(true)
    );
}
