use super::super::*;
use crate::types::{ErrorKind, Value};

#[test]
fn no_args_returns_na() {
    assert_eq!(fixed_fn(&[]), Value::Error(ErrorKind::NA));
}
