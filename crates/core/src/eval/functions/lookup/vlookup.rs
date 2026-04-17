use crate::eval::functions::check_arity;
use crate::types::{ErrorKind, Value};
use super::array_utils::{flatten_to_rows, values_equal, value_compare};

/// `VLOOKUP(search_key, range, index, [is_sorted])`
/// Searches the first column of range, returns value from the `index` column (1-based).
/// is_sorted: TRUE=approximate match (default), FALSE=exact match.
pub fn vlookup_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 3, 4) {
        return err;
    }

    let search_key = &args[0];
    let range = &args[1];
    let col_index = match &args[2] {
        Value::Number(n) => n.trunc() as usize,
        _ => return Value::Error(ErrorKind::Value),
    };
    let is_sorted = if args.len() == 4 {
        match &args[3] {
            Value::Bool(b) => *b,
            Value::Number(n) => *n != 0.0,
            _ => true,
        }
    } else {
        true
    };

    if col_index < 1 {
        return Value::Error(ErrorKind::Value);
    }

    let rows = flatten_to_rows(range);
    if rows.is_empty() {
        return Value::Error(ErrorKind::NA);
    }

    if is_sorted {
        // Approximate match: largest value in first col <= search_key
        let mut found_row: Option<&Vec<Value>> = None;
        for row in &rows {
            if row.is_empty() { continue; }
            let first = &row[0];
            match value_compare(first, search_key) {
                Some(std::cmp::Ordering::Less) | Some(std::cmp::Ordering::Equal) => {
                    found_row = Some(row);
                }
                Some(std::cmp::Ordering::Greater) => break,
                None => {}
            }
        }
        match found_row {
            None => Value::Error(ErrorKind::NA),
            Some(row) => {
                if col_index > row.len() {
                    Value::Error(ErrorKind::Ref)
                } else {
                    row[col_index - 1].clone()
                }
            }
        }
    } else {
        // Exact match
        for row in &rows {
            if row.is_empty() { continue; }
            if values_equal(&row[0], search_key) {
                if col_index > row.len() {
                    return Value::Error(ErrorKind::Ref);
                }
                return row[col_index - 1].clone();
            }
        }
        Value::Error(ErrorKind::NA)
    }
}

/// `HLOOKUP(search_key, range, index, [is_sorted])`
/// Searches the first row of range, returns value from the `index` row (1-based).
pub fn hlookup_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 3, 4) {
        return err;
    }

    let search_key = &args[0];
    let range = &args[1];
    let row_index = match &args[2] {
        Value::Number(n) => n.trunc() as usize,
        _ => return Value::Error(ErrorKind::Value),
    };
    let is_sorted = if args.len() == 4 {
        match &args[3] {
            Value::Bool(b) => *b,
            Value::Number(n) => *n != 0.0,
            _ => true,
        }
    } else {
        true
    };

    if row_index < 1 {
        return Value::Error(ErrorKind::Value);
    }

    let rows = flatten_to_rows(range);
    if rows.is_empty() {
        return Value::Error(ErrorKind::NA);
    }

    let first_row = &rows[0];

    if is_sorted {
        let mut found_col: Option<usize> = None;
        for (i, cell) in first_row.iter().enumerate() {
            match value_compare(cell, search_key) {
                Some(std::cmp::Ordering::Less) | Some(std::cmp::Ordering::Equal) => {
                    found_col = Some(i);
                }
                Some(std::cmp::Ordering::Greater) => break,
                None => {}
            }
        }
        match found_col {
            None => Value::Error(ErrorKind::NA),
            Some(col) => {
                if row_index > rows.len() {
                    return Value::Error(ErrorKind::Ref);
                }
                let target_row = &rows[row_index - 1];
                if col < target_row.len() {
                    target_row[col].clone()
                } else {
                    Value::Error(ErrorKind::NA)
                }
            }
        }
    } else {
        // Exact match in first row
        let mut found_col: Option<usize> = None;
        for (i, cell) in first_row.iter().enumerate() {
            if values_equal(cell, search_key) {
                found_col = Some(i);
                break;
            }
        }
        match found_col {
            None => Value::Error(ErrorKind::NA),
            Some(col) => {
                if row_index > rows.len() {
                    return Value::Error(ErrorKind::Ref);
                }
                let target_row = &rows[row_index - 1];
                if col < target_row.len() {
                    target_row[col].clone()
                } else {
                    Value::Error(ErrorKind::NA)
                }
            }
        }
    }
}
