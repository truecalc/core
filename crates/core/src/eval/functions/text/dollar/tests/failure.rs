use super::super::*;
use crate::types::{ErrorKind, Value};

#[test]
fn no_args_returns_na() {
    assert_eq!(dollar_fn(&[]), Value::Error(ErrorKind::NA));
}
