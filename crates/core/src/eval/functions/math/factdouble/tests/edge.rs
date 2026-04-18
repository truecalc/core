use super::super::factdouble_fn;
use crate::types::{ErrorKind, Value};

#[test]
fn overflow_returns_num_error() {
    // Very large n overflows -> #NUM!
    assert_eq!(factdouble_fn(&[Value::Number(301.0)]), Value::Error(ErrorKind::Num));
}

#[test]
fn float_truncated() {
    // FACTDOUBLE(5.9) truncates to 5: 5!! = 15
    assert_eq!(factdouble_fn(&[Value::Number(5.9)]), Value::Number(15.0));
}
