use crate::eval::coercion::to_number;
use crate::eval::functions::check_arity;
use crate::types::{ErrorKind, Value};

/// `SEQUENCE(rows, [cols], [start], [step])`
///
/// Returns an array of sequential numbers.
/// Default: cols=1, start=1, step=1.
/// Always returns a nested 2-D `Array` of `rows` row-arrays, each with `cols`
/// elements, matching Google Sheets orientation: `SEQUENCE(N)` / `SEQUENCE(N,1)`
/// is an N-row × 1-column column vector (`[[1],[2],[3]]`), not a 1×N row.
pub fn sequence_fn(args: &[Value]) -> Value {
    if args.is_empty() {
        // GS: SEQUENCE() with no args -> #REF!
        return Value::Error(ErrorKind::Ref);
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

    // Row-major nested 2-D array. A single column (cols == 1) yields a column
    // vector `[[v], [v], ...]`; a single row (rows == 1) yields `[[v, v, ...]]`.
    let outer: Vec<Value> = (0..rows)
        .map(|r| {
            let row: Vec<Value> = (0..cols)
                .map(|c| Value::Number(start + step * (r * cols + c) as f64))
                .collect();
            Value::Array(row)
        })
        .collect();
    Value::Array(outer)
}

#[cfg(test)]
mod tests;
