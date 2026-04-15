use crate::eval::functions::check_arity;
use crate::types::{ErrorKind, Value};

/// `ISNUMBER(value)` — TRUE if value is a Number.
pub fn isnumber_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 1, 1) {
        return err;
    }
    Value::Bool(matches!(args[0], Value::Number(_)))
}

/// `ISTEXT(value)` — TRUE if value is a Text string.
pub fn istext_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 1, 1) {
        return err;
    }
    Value::Bool(matches!(args[0], Value::Text(_)))
}

/// `ISERROR(value)` — TRUE if value is any Error.
pub fn iserror_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 1, 1) {
        return err;
    }
    Value::Bool(matches!(args[0], Value::Error(_)))
}

/// `ISBLANK(value)` — TRUE if value is Empty.
pub fn isblank_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 1, 1) {
        return err;
    }
    Value::Bool(matches!(args[0], Value::Empty))
}

/// `ISNA(value)` — TRUE if value is `Error(NA)` specifically.
pub fn isna_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 1, 1) {
        return err;
    }
    Value::Bool(matches!(args[0], Value::Error(ErrorKind::NA)))
}

#[cfg(test)]
mod tests;
