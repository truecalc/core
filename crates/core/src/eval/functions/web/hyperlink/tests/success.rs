use super::super::hyperlink_fn;
use crate::types::Value;

fn text(s: &str) -> Value {
    Value::Text(s.to_string())
}

#[test]
fn url_only_returns_url() {
    let url = "https://example.com";
    assert_eq!(hyperlink_fn(&[text(url)]), text(url));
}

#[test]
fn url_and_label_returns_label() {
    assert_eq!(
        hyperlink_fn(&[text("https://example.com"), text("Click here")]),
        text("Click here")
    );
}
