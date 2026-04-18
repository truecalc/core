use super::super::countunique_fn;
use crate::types::{ErrorKind, Value};

#[test]
fn zero_args_returns_na() {
    assert_eq!(countunique_fn(&[]), Value::Error(ErrorKind::NA));
}
