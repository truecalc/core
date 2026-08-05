use super::super::*;
use crate::types::{ErrorKind, Value};

#[test]
fn mode_mult_returns_smallest_mode() {
    // A single tied mode collapses to a plain scalar.
    assert_eq!(
        mode_mult_fn(&[
            Value::Number(1.0),
            Value::Number(2.0),
            Value::Number(2.0)
        ]),
        Value::Number(2.0)
    );
}

#[test]
fn mode_mult_returns_all_tied_modes_ascending() {
    // Google Sheets returns every tied mode sorted ascending, not in
    // order of first appearance: {3,3,1,1,2} -> {1;3}, not {3;1}.
    assert_eq!(
        mode_mult_fn(&[Value::Array(vec![
            Value::Number(3.0),
            Value::Number(3.0),
            Value::Number(1.0),
            Value::Number(1.0),
            Value::Number(2.0),
        ])]),
        Value::Array(vec![
            Value::Array(vec![Value::Number(1.0)]),
            Value::Array(vec![Value::Number(3.0)]),
        ])
    );
}
