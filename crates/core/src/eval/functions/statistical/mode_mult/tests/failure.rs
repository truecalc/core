use super::super::*;
use crate::types::{ErrorKind, Value};

#[test]
fn mode_mult_returns_ref_error() {
    // MODE.MULT is an array-spill function; in scalar context GS returns #REF!
    assert_eq!(
        mode_mult_fn(&[Value::Number(1.0), Value::Number(2.0), Value::Number(3.0)]),
        Value::Error(ErrorKind::Ref)
    );
}

#[test]
fn mode_mult_no_args_returns_ref_error() {
    assert_eq!(mode_mult_fn(&[]), Value::Error(ErrorKind::Ref));
}
