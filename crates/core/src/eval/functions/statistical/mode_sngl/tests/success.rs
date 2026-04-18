use super::super::*;
use crate::types::Value;

#[test]
fn mode_sngl_simple() {
    assert_eq!(
        mode_sngl_fn(&[
            Value::Number(1.0),
            Value::Number(2.0),
            Value::Number(2.0)
        ]),
        Value::Number(2.0)
    );
}
