use super::super::*;
use crate::types::{ErrorKind, Value};

#[test]
fn too_many_args() {
    assert_eq!(now_fn(&[Value::Number(0.0)]), Value::Error(ErrorKind::NA));
}
