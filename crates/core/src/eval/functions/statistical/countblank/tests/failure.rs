use super::super::countblank_fn;
use crate::types::{ErrorKind, Value};

#[test]
fn no_args_returns_na() {
    assert_eq!(countblank_fn(&[]), Value::Error(ErrorKind::NA));
}
