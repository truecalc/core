use super::super::*;
use crate::types::{ErrorKind, Value};

#[test]
fn mode_mult_returns_smallest_mode() {
    // MODE.MULT returns the smallest mode in scalar context
    assert_eq!(
        mode_mult_fn(&[
            Value::Number(1.0),
            Value::Number(2.0),
            Value::Number(2.0)
        ]),
        Value::Number(2.0)
    );
}
