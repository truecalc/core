use crate::eval::functions::check_arity;
use crate::types::{ErrorKind, Value};

pub fn na_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 0, 0) {
        return err;
    }
    Value::Error(ErrorKind::NA)
}

pub fn true_fn(args: &[Value]) -> Value {
    if !args.is_empty() {
        return Value::Error(ErrorKind::NA);
    }
    Value::Bool(true)
}

pub fn false_fn(args: &[Value]) -> Value {
    if !args.is_empty() {
        return Value::Error(ErrorKind::NA);
    }
    Value::Bool(false)
}
