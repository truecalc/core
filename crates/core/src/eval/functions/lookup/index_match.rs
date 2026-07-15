use crate::eval::functions::check_arity;
use crate::types::{ErrorKind, Value};
use super::array_utils::{flatten_to_rows, flatten_to_flat, values_equal, value_compare, wildcard_match_value, has_wildcards};

/// Helper: coerce a Value to an index (i64). Handles Number and Bool; propagates errors.
/// Returns Err(Value::Error(...)) if coercion fails.
fn coerce_index(v: &Value) -> Result<i64, Value> {
    match v {
        Value::Number(n) => Ok(n.trunc() as i64),
        Value::Bool(b) => Ok(if *b { 1 } else { 0 }),
        Value::Error(e) => Err(Value::Error(e.clone())),
        Value::ErrorMsg(e, m) => Err(Value::ErrorMsg(e.clone(), m.clone())),
        _ => Err(Value::Error(ErrorKind::Value)),
    }
}

/// `INDEX(array, row, [col])` — returns the value at row/col of array.
/// Row and col are 1-based. Negative -> #VALUE!, out of bounds -> #REF!.
/// row=0 or col=0 means "all" (returns entire row/col or for 1D: first element).
pub fn index_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 2, 3) {
        return err;
    }

    let array_val = &args[0];

    let row_idx_raw = match coerce_index(&args[1]) {
        Ok(n) => n,
        Err(e) => return e,
    };
    if row_idx_raw < 0 {
        return Value::Error(ErrorKind::Value);
    }
    let row_idx = row_idx_raw as usize;

    let col_idx = if args.len() == 3 {
        let col_raw = match coerce_index(&args[2]) {
            Ok(n) => n,
            Err(e) => return e,
        };
        if col_raw < 0 {
            return Value::Error(ErrorKind::Value);
        }
        col_raw as usize
    } else {
        0
    };

    let rows = flatten_to_rows(array_val);
    let is_2d = matches!(array_val, Value::Array(v) if v.iter().any(|e| matches!(e, Value::Array(_))));

    if is_2d {
        // row=0 means return entire column; col=0 means return entire row
        if row_idx == 0 && col_idx == 0 {
            return array_val.clone();
        }
        if row_idx > rows.len() {
            return Value::Error(ErrorKind::Ref);
        }
        if row_idx == 0 {
            // Return entire column col_idx across all rows
            let col = col_idx;
            if col < 1 || rows.iter().any(|r| col > r.len()) {
                return Value::Error(ErrorKind::Ref);
            }
            let col_vals: Vec<Value> = rows.iter().map(|r| r[col - 1].clone()).collect();
            return Value::Array(col_vals);
        }
        let row = &rows[row_idx - 1];
        if col_idx == 0 {
            return Value::Array(row.clone());
        }
        if col_idx > row.len() {
            return Value::Error(ErrorKind::Ref);
        }
        row[col_idx - 1].clone()
    } else {
        let flat = flatten_to_flat(array_val);
        if col_idx == 0 {
            // row=0: return entire array; row>=1: return that element
            if row_idx == 0 {
                // GS: INDEX(row_vector, 0) returns first element
                return flat.first().cloned().unwrap_or(Value::Error(ErrorKind::Ref));
            }
            if row_idx > flat.len() {
                return Value::Error(ErrorKind::Ref);
            }
            flat[row_idx - 1].clone()
        } else if row_idx == 0 || row_idx == 1 {
            // Row vector: row=0 or row=1 both select by column index
            if col_idx > flat.len() {
                return Value::Error(ErrorKind::Ref);
            }
            flat[col_idx - 1].clone()
        } else if col_idx == 1 {
            if row_idx > flat.len() {
                return Value::Error(ErrorKind::Ref);
            }
            flat[row_idx - 1].clone()
        } else {
            Value::Error(ErrorKind::Ref)
        }
    }
}

/// `MATCH(search_key, range, [match_type])` -- returns 1-based position of search_key.
/// match_type: 0=exact, 1=largest <= key (sorted asc, default), -1=smallest >= key (sorted desc).
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

    // Flatten range. Column vectors {"a";"b";"c"} parse as
    // Array([Array(["a"]), Array(["b"]), Array(["c"])]) -- flatten to 1D.
    let flat: Vec<Value> = match range_val {
        Value::Array(outer) => {
            let all_single = outer.iter().all(|e| matches!(e, Value::Array(v) if v.len() == 1));
            let any_multi = outer.iter().any(|e| matches!(e, Value::Array(v) if v.len() > 1));
            if all_single && outer.iter().any(|e| matches!(e, Value::Array(_))) {
                outer.iter().map(|e| match e {
                    Value::Array(v) => v[0].clone(),
                    other => other.clone(),
                }).collect()
            } else if any_multi {
                return Value::Error(ErrorKind::NA);
            } else {
                flatten_to_flat(range_val)
            }
        }
        _ => flatten_to_flat(range_val),
    };

    if flat.is_empty() {
        return Value::Error(ErrorKind::NA);
    }

    match match_type {
        0 => {
            for (i, v) in flat.iter().enumerate() {
                let matched = if has_wildcards(search_key) {
                    wildcard_match_value(search_key, v)
                } else {
                    values_equal(v, search_key)
                };
                if matched {
                    return Value::Number((i + 1) as f64);
                }
            }
            Value::Error(ErrorKind::NA)
        }
        1 => {
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
            let mut result: Option<usize> = None;
            for (i, v) in flat.iter().enumerate() {
                match value_compare(v, search_key) {
                    Some(std::cmp::Ordering::Greater) | Some(std::cmp::Ordering::Equal) => {
                        result = Some(i + 1);
                    }
                    Some(std::cmp::Ordering::Less) => break,
                    None => {}
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
