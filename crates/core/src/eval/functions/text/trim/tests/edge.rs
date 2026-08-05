use super::super::*;
use crate::types::Value;

#[test]
fn empty_string() {
    assert_eq!(
        trim_fn(&[Value::Text("".to_string())]),
        Value::Text("".to_string())
    );
}

#[test]
fn only_spaces() {
    assert_eq!(
        trim_fn(&[Value::Text("   ".to_string())]),
        Value::Text("".to_string())
    );
}

#[test]
fn tabs_and_newlines() {
    assert_eq!(
        trim_fn(&[Value::Text("\t a \n b \t".to_string())]),
        Value::Text("a b".to_string())
    );
}
