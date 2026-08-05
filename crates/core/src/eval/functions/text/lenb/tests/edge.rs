use super::super::*;
use crate::types::Value;

#[test]
fn empty_string() {
    assert_eq!(
        lenb_fn(&[Value::Text("".to_string())]),
        Value::Number(0.0)
    );
}
