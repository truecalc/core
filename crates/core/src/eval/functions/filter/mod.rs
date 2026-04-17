use super::super::{FunctionMeta, Registry};
use crate::eval::functions::check_arity;
use crate::types::{ErrorKind, Value};

/// `FILTER(array, include, [if_empty])` — return elements of `array` where
/// the corresponding `include` element is truthy.
///
/// For a 1-D array both arguments must have the same length.
/// If no elements pass the filter, returns `if_empty` (default #N/A).
pub fn filter_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 2, 3) {
        return err;
    }
    let array = &args[0];
    let include = &args[1];

    let arr_elems = match array {
        Value::Array(v) => v,
        // Scalar array: wrap in a single-element check
        _ => {
            let keep = is_truthy(include);
            if keep {
                return array.clone();
            } else {
                return if_empty_value(args);
            }
        }
    };

    let inc_elems = match include {
        Value::Array(v) => v,
        // Scalar include: apply same bool to all elements
        _ => {
            if is_truthy(include) {
                return array.clone();
            } else {
                return if_empty_value(args);
            }
        }
    };

    let mut result: Vec<Value> = Vec::new();
    for (elem, flag) in arr_elems.iter().zip(inc_elems.iter()) {
        if is_truthy(flag) {
            result.push(elem.clone());
        }
    }

    if result.is_empty() {
        return if_empty_value(args);
    }

    Value::Array(result)
}

fn is_truthy(v: &Value) -> bool {
    match v {
        Value::Bool(b) => *b,
        Value::Number(n) => *n != 0.0,
        Value::Text(s) => !s.is_empty(),
        _ => false,
    }
}

fn if_empty_value(args: &[Value]) -> Value {
    if args.len() >= 3 {
        args[2].clone()
    } else {
        Value::Error(ErrorKind::NA)
    }
}

/// `SORT(array, [sort_index], [is_ascending], [by_col])` — sort an array.
///
/// For 2-D arrays (nested Array-of-Arrays), sorts rows by the specified column.
/// For 1-D horizontal arrays, returns unchanged (already a single row).
pub fn sort_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 1, 4) {
        return err;
    }
    let array = &args[0];

    // Parse optional arguments
    let sort_col = args.get(1).and_then(|v| match v {
        Value::Number(n) => Some(*n as usize),
        _ => None,
    }).unwrap_or(1);

    let is_ascending = args.get(2).map(|v| match v {
        Value::Bool(b) => *b,
        Value::Number(n) => *n != 0.0,
        _ => true,
    }).unwrap_or(true);

    match array {
        Value::Array(rows) => {
            // Check if it's a 2-D array (rows of arrays)
            let is_2d = rows.iter().any(|r| matches!(r, Value::Array(_)));
            if is_2d {
                let mut sorted_rows: Vec<Value> = rows.clone();
                sorted_rows.sort_by(|a, b| {
                    let key_a = get_col_value(a, sort_col);
                    let key_b = get_col_value(b, sort_col);
                    let cmp = compare_sort_values(&key_a, &key_b);
                    if is_ascending { cmp } else { cmp.reverse() }
                });
                Value::Array(sorted_rows)
            } else {
                // 1-D horizontal array: return unchanged
                array.clone()
            }
        }
        _ => array.clone(),
    }
}

fn get_col_value(row: &Value, col: usize) -> Value {
    match row {
        Value::Array(elems) => {
            if col >= 1 && col <= elems.len() {
                elems[col - 1].clone()
            } else {
                Value::Empty
            }
        }
        other => other.clone(),
    }
}

fn compare_sort_values(a: &Value, b: &Value) -> std::cmp::Ordering {
    match (a, b) {
        (Value::Number(x), Value::Number(y)) => x.partial_cmp(y).unwrap_or(std::cmp::Ordering::Equal),
        (Value::Text(x), Value::Text(y)) => x.cmp(y),
        (Value::Bool(x), Value::Bool(y)) => x.cmp(y),
        _ => std::cmp::Ordering::Equal,
    }
}

/// `SORTN(array, [n], [display_ties_mode], [sort_column], [is_ascending], ...)` —
/// for 1-D horizontal arrays, returns the array unchanged.
pub fn sortn_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 1, 255) {
        return err;
    }
    args[0].clone()
}

/// `UNIQUE(array, [by_col], [exactly_once])` — for 1-D horizontal arrays,
/// returns the array unchanged.
pub fn unique_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 1, 3) {
        return err;
    }
    args[0].clone()
}

/// `INDEX(array, row_num, [col_num])` — return the element at the specified
/// position (1-based).
///
/// For a 2-D array (Array-of-Arrays): row selects the outer row, col selects
/// within that row.
/// For a 1-D array:
/// - `INDEX(arr, n)` returns `arr[n-1]` (treating the array as a column vector)
/// - `INDEX(arr, 1, n)` returns `arr[n-1]` (treating the array as a row vector)
pub fn index_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 2, 3) {
        return err;
    }

    let arr = &args[0];
    let row_arg = &args[1];
    let col_arg = args.get(2);

    let elements = match arr {
        Value::Array(v) => v,
        // Scalar: INDEX(scalar, 1) or INDEX(scalar, 1, 1) returns the scalar
        other => {
            let row = coerce_to_index(row_arg);
            let col = col_arg.map(coerce_to_index).unwrap_or(Ok(1));
            match (row, col) {
                (Ok(1), Ok(1)) => return other.clone(),
                _ => return Value::Error(ErrorKind::Ref),
            }
        }
    };

    let row = match coerce_to_index(row_arg) {
        Ok(n) => n,
        Err(e) => return e,
    };

    // Check if this is a 2-D array (outer elements are themselves arrays)
    let is_2d = elements.iter().any(|e| matches!(e, Value::Array(_)));

    if is_2d {
        // 2-D: outer index = row, inner index = col
        if row < 1 || row > elements.len() {
            return Value::Error(ErrorKind::Ref);
        }
        let row_val = &elements[row - 1];
        match col_arg {
            None => row_val.clone(),
            Some(col_val) => {
                let col = match coerce_to_index(col_val) {
                    Ok(n) => n,
                    Err(e) => return e,
                };
                match row_val {
                    Value::Array(cols) => {
                        if col < 1 || col > cols.len() {
                            return Value::Error(ErrorKind::Ref);
                        }
                        cols[col - 1].clone()
                    }
                    other => {
                        if col == 1 { other.clone() } else { Value::Error(ErrorKind::Ref) }
                    }
                }
            }
        }
    } else {
        // 1-D array
        match col_arg {
            None => {
                // INDEX(arr, n) — treat as column vector
                if row < 1 || row > elements.len() {
                    return Value::Error(ErrorKind::Ref);
                }
                elements[row - 1].clone()
            }
            Some(col_val) => {
                let col = match coerce_to_index(col_val) {
                    Ok(n) => n,
                    Err(e) => return e,
                };
                // For a flat (1-D) array, treat as a row vector:
                // row must be 1, col selects the element.
                if row == 1 {
                    if col < 1 || col > elements.len() {
                        return Value::Error(ErrorKind::Ref);
                    }
                    elements[col - 1].clone()
                } else if col == 1 {
                    // Treat as column vector
                    if row < 1 || row > elements.len() {
                        return Value::Error(ErrorKind::Ref);
                    }
                    elements[row - 1].clone()
                } else {
                    Value::Error(ErrorKind::Ref)
                }
            }
        }
    }
}

fn coerce_to_index(v: &Value) -> Result<usize, Value> {
    match v {
        Value::Number(n) => {
            let n = *n;
            if n < 1.0 || n.fract() != 0.0 {
                Err(Value::Error(ErrorKind::Value))
            } else {
                Ok(n as usize)
            }
        }
        _ => Err(Value::Error(ErrorKind::Value)),
    }
}

/// `ROWS(array)` — return the number of rows in an array.
/// For a 1-D flat array, always returns 1.
pub fn rows_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 1, 1) {
        return err;
    }
    match &args[0] {
        Value::Array(_) => Value::Number(1.0),
        _ => Value::Number(1.0),
    }
}

/// `COLUMNS(array)` — return the number of columns in an array.
/// For a 1-D flat array, returns the number of elements.
pub fn columns_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 1, 1) {
        return err;
    }
    match &args[0] {
        Value::Array(v) => Value::Number(v.len() as f64),
        _ => Value::Number(1.0),
    }
}

#[cfg(test)]
mod tests;

pub fn register_filter(registry: &mut Registry) {
    registry.register_eager("FILTER",  filter_fn,  FunctionMeta { category: "filter", signature: "FILTER(array, include, [if_empty])",  description: "Filter an array by a boolean mask" });
    registry.register_eager("SORT",    sort_fn,    FunctionMeta { category: "filter", signature: "SORT(array, [sort_index], [is_ascending], [by_col])", description: "Sort an array" });
    registry.register_eager("SORTN",   sortn_fn,   FunctionMeta { category: "filter", signature: "SORTN(array, [n], [display_ties_mode], ...)", description: "Return top N rows of an array sorted" });
    registry.register_eager("UNIQUE",  unique_fn,  FunctionMeta { category: "filter", signature: "UNIQUE(array, [by_col], [exactly_once])", description: "Return unique rows or columns from an array" });
    registry.register_eager("INDEX",   index_fn,   FunctionMeta { category: "filter", signature: "INDEX(array, row_num, [col_num])",      description: "Return element at given row/col from array" });
    registry.register_eager("ROWS",    rows_fn,    FunctionMeta { category: "filter", signature: "ROWS(array)",                           description: "Return the number of rows in an array" });
    registry.register_eager("COLUMNS", columns_fn, FunctionMeta { category: "filter", signature: "COLUMNS(array)",                        description: "Return the number of columns in an array" });
}
