use crate::eval::coercion::to_number;
use crate::eval::functions::check_arity;
use crate::types::{ErrorKind, Value};

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Returns true if the array is a nested 2D array (outer = rows, inner = cols).
fn is_nested(items: &[Value]) -> bool {
    !items.is_empty() && matches!(items[0], Value::Array(_))
}

/// ROWS(array) — returns the number of rows in the array.
///
/// For a nested 2D array: number of outer elements.
/// For a flat 1D array: 1 (row vector).
/// For a scalar: 1.
pub fn rows_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 1, 1) {
        return err;
    }
    match &args[0] {
        Value::Array(items) => {
            if is_nested(items) {
                Value::Number(items.len() as f64)
            } else {
                // Flat array = 1 row
                Value::Number(1.0)
            }
        }
        _ => Value::Number(1.0),
    }
}

/// COLUMNS(array) — returns the number of columns in the array.
///
/// For a nested 2D array: number of elements in first inner array.
/// For a flat 1D array: number of elements (row vector).
/// For a scalar: 1.
pub fn columns_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 1, 1) {
        return err;
    }
    match &args[0] {
        Value::Array(items) => {
            if is_nested(items) {
                // Number of cols = size of first inner array
                match &items[0] {
                    Value::Array(inner) => Value::Number(inner.len() as f64),
                    _ => Value::Number(1.0),
                }
            } else {
                // Flat 1D array = 1 row of N columns
                Value::Number(items.len() as f64)
            }
        }
        _ => Value::Number(1.0),
    }
}

/// INDEX(array, row, [col]) — returns the element at (row, col) (1-based).
///
/// For a nested 2D array: outer[row-1][col-1].
/// For a flat 1D array with 2 args (row only): flat[row-1] (column vector).
/// For a flat 1D array with 3 args (row, col): flat[col-1] if row=1.
pub fn index_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 2, 3) {
        return err;
    }
    let row = match to_number(args[1].clone()) {
        Err(e) => return e,
        Ok(v) => v as usize,
    };
    match &args[0] {
        Value::Array(items) => {
            if is_nested(items) {
                // 2D nested array
                let col = if args.len() == 3 {
                    match to_number(args[2].clone()) {
                        Err(e) => return e,
                        Ok(v) => v as usize,
                    }
                } else {
                    1
                };
                if row == 0 || col == 0 {
                    return Value::Error(ErrorKind::Value);
                }
                let outer_idx = row - 1;
                if outer_idx >= items.len() {
                    return Value::Error(ErrorKind::Ref);
                }
                match &items[outer_idx] {
                    Value::Array(inner) => {
                        let inner_idx = col - 1;
                        if inner_idx >= inner.len() {
                            return Value::Error(ErrorKind::Ref);
                        }
                        inner[inner_idx].clone()
                    }
                    other => other.clone(),
                }
            } else {
                // Flat 1D array
                if row == 0 {
                    return Value::Error(ErrorKind::Value);
                }
                if args.len() == 3 {
                    // 3 args: treat as row vector (1 row, N cols)
                    let col = match to_number(args[2].clone()) {
                        Err(e) => return e,
                        Ok(v) => v as usize,
                    };
                    if row != 1 {
                        return Value::Error(ErrorKind::Ref);
                    }
                    if col == 0 || col > items.len() {
                        return Value::Error(ErrorKind::Ref);
                    }
                    items[col - 1].clone()
                } else {
                    // 2 args: treat as column vector (N rows, 1 col)
                    if row > items.len() {
                        return Value::Error(ErrorKind::Ref);
                    }
                    items[row - 1].clone()
                }
            }
        }
        other => {
            // Scalar: only valid for (1,1) or just (1)
            if row != 1 {
                return Value::Error(ErrorKind::Ref);
            }
            if args.len() == 3 {
                let col = match to_number(args[2].clone()) {
                    Err(e) => return e,
                    Ok(v) => v as usize,
                };
                if col != 1 {
                    return Value::Error(ErrorKind::Ref);
                }
            }
            other.clone()
        }
    }
}
