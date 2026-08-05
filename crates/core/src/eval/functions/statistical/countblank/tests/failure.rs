use crate::eval::functions::check_arity_len;
use crate::types::{ErrorKind, Value};

#[test]
fn no_args_returns_na() {
    // countblank_fn checks arity 1..255; 0 args -> NA
    assert_eq!(
        check_arity_len(0, 1, 255),
        Some(Value::Error(ErrorKind::NA))
    );
}
