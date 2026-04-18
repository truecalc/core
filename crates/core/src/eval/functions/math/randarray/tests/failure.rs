use super::super::randarray_fn;
use crate::types::{ErrorKind, Value};

fn n(v: f64) -> Value {
    Value::Number(v)
}

#[test]
fn zero_rows_returns_num_error() {
    assert_eq!(randarray_fn(&[n(0.0)]), Value::Error(ErrorKind::Num));
}

#[test]
fn negative_rows_returns_num_error() {
    assert_eq!(randarray_fn(&[n(-1.0)]), Value::Error(ErrorKind::Num));
}

#[test]
fn zero_cols_returns_num_error() {
    assert_eq!(randarray_fn(&[n(2.0), n(0.0)]), Value::Error(ErrorKind::Num));
}

#[test]
fn negative_cols_returns_num_error() {
    assert_eq!(
        randarray_fn(&[n(2.0), n(-3.0)]),
        Value::Error(ErrorKind::Num)
    );
}

#[test]
fn too_many_args_returns_na() {
    let args = vec![n(1.0); 6];
    assert_eq!(randarray_fn(&args), Value::Error(ErrorKind::NA));
}
