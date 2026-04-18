use crate::eval::coercion::to_number;
use crate::eval::functions::check_arity;
use crate::types::{ErrorKind, Value};

/// `SEQUENCE(rows, [cols], [start], [step])`
///
/// Returns an array of sequential numbers.
/// Default: cols=1, start=1, step=1.
/// If cols=1: returns a flat column vector Array([start, start+step, ...]).
/// If cols>1: returns a nested 2D Array.
pub fn sequence_fn(args: &[Value]) -> Value {
    if args.is_empty() {
        // No args: SEQUENCE() = SEQUENCE(1) = [1]
        return Value::Number(1.0);
    }
    if let Some(err) = check_arity(args, 1, 4) {
        return err;
    }
    let rows = match to_number(args[0].clone()) {
        Err(e) => return e,
        Ok(v) => v as usize,
    };
    let cols = if args.len() >= 2 {
        match to_number(args[1].clone()) {
            Err(e) => return e,
            Ok(v) => v as usize,
        }
    } else {
        1
    };
    let start = if args.len() >= 3 {
        match to_number(args[2].clone()) {
            Err(e) => return e,
            Ok(v) => v,
        }
    } else {
        1.0
    };
    let step = if args.len() >= 4 {
        match to_number(args[3].clone()) {
            Err(e) => return e,
            Ok(v) => v,
        }
    } else {
        1.0
    };

    if rows == 0 || cols == 0 {
        return Value::Error(ErrorKind::Num);
    }

    if cols == 1 {
        // Return flat array (column vector)
        let items: Vec<Value> = (0..rows)
            .map(|i| Value::Number(start + step * i as f64))
            .collect();
        Value::Array(items)
    } else {
        // Return nested 2D array
        let mut val = start;
        let outer: Vec<Value> = (0..rows)
            .map(|_| {
                let row: Vec<Value> = (0..cols)
                    .map(|_| {
                        let v = Value::Number(val);
                        val += step;
                        v
                    })
                    .collect();
                Value::Array(row)
            })
            .collect();
        Value::Array(outer)
    }
}
