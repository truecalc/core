use crate::eval::coercion::to_number;
use crate::eval::functions::check_arity;
use crate::types::{ErrorKind, Value};

/// `MUNIT(dimension)` — returns an n×n identity matrix as a nested Array.
pub fn munit_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 1, 1) {
        return err;
    }
    let n = match to_number(args[0].clone()) {
        Err(e) => return e,
        Ok(v) => v,
    };
    if n <= 0.0 {
        return Value::Error(ErrorKind::Num);
    }
    let n = n as usize;
    let rows: Vec<Value> = (0..n)
        .map(|r| {
            let row: Vec<Value> = (0..n)
                .map(|c| Value::Number(if r == c { 1.0 } else { 0.0 }))
                .collect();
            Value::Array(row)
        })
        .collect();
    Value::Array(rows)
}
