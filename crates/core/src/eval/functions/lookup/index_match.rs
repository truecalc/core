use crate::eval::functions::check_arity;
use crate::types::{ErrorKind, Value};
use super::array_utils::{flatten_to_rows, flatten_to_flat, values_equal, value_compare};

/// `INDEX(array, row, [col])` — returns the value at row/col of array.
/// Row and col are 1-based. Returns #REF! if out of bounds.
pub fn index_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 2, 3) {
        return err;
    }

    let array_val = &args[0];
    let row_idx = match &args[1] {
        Value::Number(n) => n.trunc() as usize,
        _ => return Value::Error(ErrorKind::Value),
    };
    let col_idx = if args.len() == 3 {
        match &args[2] {
            Value::Number(n) => n.trunc() as usize,
            _ => return Value::Error(ErrorKind::Value),
        }
    } else {
        0 // 0 means not specified
    };

    let rows = flatten_to_rows(array_val);

    // Check if 2D or 1D
    let is_2d = matches!(array_val, Value::Array(v) if v.iter().any(|e| matches!(e, Value::Array(_))));

    if is_2d {
        if row_idx < 1 || row_idx > rows.len() {
            return Value::Error(ErrorKind::Ref);
        }
        let row = &rows[row_idx - 1];
        if col_idx == 0 {
            // Return entire row as array? For conformance, return the row
            return Value::Array(row.clone());
        }
        if col_idx > row.len() {
            return Value::Error(ErrorKind::Ref);
        }
        row[col_idx - 1].clone()
    } else {
        // 1D array
        let flat = flatten_to_flat(array_val);
        if col_idx == 0 {
            // Single index → treat as column vector
            if row_idx < 1 || row_idx > flat.len() {
                return Value::Error(ErrorKind::Ref);
            }
            flat[row_idx - 1].clone()
        } else if row_idx == 1 {
            // Row index = 1, treat as row vector
            if col_idx > flat.len() {
                return Value::Error(ErrorKind::Ref);
            }
            flat[col_idx - 1].clone()
        } else if col_idx == 1 {
            // Col index = 1, treat as column vector
            if row_idx > flat.len() {
                return Value::Error(ErrorKind::Ref);
            }
            flat[row_idx - 1].clone()
        } else {
            Value::Error(ErrorKind::Ref)
        }
    }
}

/// `MATCH(search_key, range, [match_type])` — returns 1-based position of search_key.
/// match_type: 0=exact, 1=less than (sorted asc, default), -1=greater than (sorted desc).
pub fn match_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 2, 3) {
        return err;
    }

    let search_key = &args[0];
    let range_val = &args[1];
    let match_type = if args.len() == 3 {
        match &args[2] {
            Value::Number(n) => n.trunc() as i64,
            _ => return Value::Error(ErrorKind::Value),
        }
    } else {
        1
    };

    let flat = flatten_to_flat(range_val);
    if flat.is_empty() {
        return Value::Error(ErrorKind::NA);
    }

    match match_type {
        0 => {
            // Exact match
            for (i, v) in flat.iter().enumerate() {
                if values_equal(v, search_key) {
                    return Value::Number((i + 1) as f64);
                }
            }
            Value::Error(ErrorKind::NA)
        }
        1 => {
            // Largest value <= search_key in sorted ascending array
            let mut result: Option<usize> = None;
            for (i, v) in flat.iter().enumerate() {
                match value_compare(v, search_key) {
                    Some(std::cmp::Ordering::Less) | Some(std::cmp::Ordering::Equal) => {
                        result = Some(i + 1);
                    }
                    _ => break,
                }
            }
            match result {
                Some(pos) => Value::Number(pos as f64),
                None => Value::Error(ErrorKind::NA),
            }
        }
        -1 => {
            // Smallest value >= search_key in sorted descending array
            let mut result: Option<usize> = None;
            for (i, v) in flat.iter().enumerate() {
                match value_compare(v, search_key) {
                    Some(std::cmp::Ordering::Greater) | Some(std::cmp::Ordering::Equal) => {
                        result = Some(i + 1);
                    }
                    _ => break,
                }
            }
            match result {
                Some(pos) => Value::Number(pos as f64),
                None => Value::Error(ErrorKind::NA),
            }
        }
        _ => Value::Error(ErrorKind::Value),
    }
}
