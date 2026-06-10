use crate::eval::functions::check_arity;
use crate::types::{ErrorKind, Value};
use super::array_utils::{flatten_to_flat, flatten_to_rows, values_equal, value_compare, wildcard_match_value, has_wildcards};

/// `LOOKUP(search_key, search_range, [result_range])`
/// Approximate lookup: find the largest value <= search_key.
/// Uses a full linear scan (not binary search) to handle unsorted ranges correctly.
pub fn lookup_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 2, 3) {
        return err;
    }

    let search_key = &args[0];

    // 2-argument array form: args[1] is a 2D array, no explicit result_range.
    // If array has more columns than rows: search first ROW, return corresponding last ROW value.
    // If array has more rows than columns (or square): search first COLUMN, return last COLUMN value.
    if args.len() == 2 {
        let rows = flatten_to_rows(&args[1]);
        let nrows = rows.len();
        let ncols = rows.first().map(|r| r.len()).unwrap_or(0);

        if nrows > 1 || ncols > 1 {
            if ncols > nrows {
                // Horizontal: search first row, return from last row
                if rows.is_empty() || rows[0].is_empty() {
                    return Value::Error(ErrorKind::NA);
                }
                let search_vec: Vec<Value> = rows[0].clone();
                let result_vec: Vec<Value> = rows[nrows - 1].clone();
                return lookup_vector(search_key, &search_vec, Some(&result_vec));
            } else {
                // Vertical: search first col, return from last col
                let search_vec: Vec<Value> = rows.iter().map(|r| r[0].clone()).collect();
                let result_vec: Vec<Value> = rows.iter().map(|r| r[r.len() - 1].clone()).collect();
                return lookup_vector(search_key, &search_vec, Some(&result_vec));
            }
        }
    }

    let search_range = flatten_to_flat(&args[1]);
    let result_range: Option<Vec<Value>> = if args.len() == 3 {
        Some(flatten_to_flat(&args[2]))
    } else {
        None
    };

    lookup_vector(search_key, &search_range, result_range.as_deref())
}

/// Core LOOKUP vector search: find last value <= search_key, return corresponding result value.
/// Full linear scan (handles unsorted ranges — GS behaviour).
fn lookup_vector(search_key: &Value, search_range: &[Value], result_range: Option<&[Value]>) -> Value {
    let mut found_idx: Option<usize> = None;
    for (i, v) in search_range.iter().enumerate() {
        match value_compare(v, search_key) {
            Some(std::cmp::Ordering::Less) | Some(std::cmp::Ordering::Equal) => {
                found_idx = Some(i);
            }
            _ => {}
        }
    }

    match found_idx {
        None => Value::Error(ErrorKind::NA),
        Some(idx) => match result_range {
            Some(result) => {
                if idx < result.len() {
                    result[idx].clone()
                } else {
                    Value::Error(ErrorKind::NA)
                }
            }
            None => search_range[idx].clone(),
        },
    }
}

/// Find the position of `search_key` in `arr` using the given `match_mode` and `search_mode`.
/// Returns the 0-based index, or None if not found.
fn xlookup_find(
    arr: &[Value],
    search_key: &Value,
    match_mode: i64,
    search_mode: i64,
) -> Option<usize> {
    match match_mode {
        0 => {
            if search_mode == -1 {
                if has_wildcards(search_key) {
                    arr.iter().rposition(|v| wildcard_match_value(search_key, v))
                } else {
                    arr.iter().rposition(|v| values_equal(v, search_key))
                }
            } else if has_wildcards(search_key) {
                arr.iter().position(|v| wildcard_match_value(search_key, v))
            } else {
                arr.iter().position(|v| values_equal(v, search_key))
            }
        }
        1 => {
            let mut res: Option<usize> = None;
            for (i, v) in arr.iter().enumerate() {
                if values_equal(v, search_key) { return Some(i); }
                if let Some(std::cmp::Ordering::Greater) = value_compare(v, search_key) {
                    res = Some(i);
                    break;
                }
            }
            res
        }
        -1 => {
            let mut res: Option<usize> = None;
            for (i, v) in arr.iter().enumerate() {
                if values_equal(v, search_key) { return Some(i); }
                match value_compare(v, search_key) {
                    Some(std::cmp::Ordering::Less) => { res = Some(i); }
                    Some(std::cmp::Ordering::Greater) => break,
                    _ => {}
                }
            }
            res
        }
        2 => {
            if has_wildcards(search_key) {
                if search_mode == -1 {
                    arr.iter().rposition(|v| wildcard_match_value(search_key, v))
                } else {
                    arr.iter().position(|v| wildcard_match_value(search_key, v))
                }
            } else if search_mode == -1 {
                arr.iter().rposition(|v| values_equal(v, search_key))
            } else {
                arr.iter().position(|v| values_equal(v, search_key))
            }
        }
        _ => {
            if search_mode == -1 {
                arr.iter().rposition(|v| values_equal(v, search_key))
            } else {
                arr.iter().position(|v| values_equal(v, search_key))
            }
        }
    }
}

/// `XLOOKUP(search_key, lookup_array, return_array, [if_not_found], [match_mode], [search_mode])`
pub fn xlookup_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 3, 6) {
        return err;
    }

    let search_key = &args[0];
    let lookup_array = flatten_to_flat(&args[1]);
    let return_array = flatten_to_flat(&args[2]);

    if lookup_array.is_empty() {
        return Value::Error(ErrorKind::Ref);
    }

    let if_not_found: Option<Value> = if args.len() >= 4 {
        Some(args[3].clone())
    } else {
        None
    };
    let match_mode = if args.len() >= 5 {
        match &args[4] {
            Value::Number(n) => n.trunc() as i64,
            _ => 0,
        }
    } else {
        0
    };
    let search_mode = if args.len() >= 6 {
        match &args[5] {
            Value::Number(n) => n.trunc() as i64,
            _ => 1,
        }
    } else {
        1
    };

    let result_idx = xlookup_find(&lookup_array, search_key, match_mode, search_mode);

    match result_idx {
        Some(idx) => {
            if idx < return_array.len() {
                return_array[idx].clone()
            } else {
                Value::Error(ErrorKind::NA)
            }
        }
        None => match if_not_found {
            Some(v) => v,
            None => Value::Error(ErrorKind::NA),
        },
    }
}

/// `XMATCH(search_key, lookup_array, [match_mode], [search_mode])`
/// Returns 1-based position of search_key in lookup_array.
pub fn xmatch_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 2, 4) {
        return err;
    }

    let search_key = &args[0];
    let lookup_array = flatten_to_flat(&args[1]);
    let match_mode = if args.len() >= 3 {
        match &args[2] {
            Value::Number(n) => n.trunc() as i64,
            _ => 0,
        }
    } else {
        0
    };
    let search_mode = if args.len() >= 4 {
        match &args[3] {
            Value::Number(n) => n.trunc() as i64,
            _ => 1,
        }
    } else {
        1
    };

    match match_mode {
        0 => {
            let pos = if has_wildcards(search_key) {
                if search_mode == -1 {
                    lookup_array.iter().rposition(|v| wildcard_match_value(search_key, v))
                } else {
                    lookup_array.iter().position(|v| wildcard_match_value(search_key, v))
                }
            } else if search_mode == -1 {
                lookup_array.iter().rposition(|v| values_equal(v, search_key))
            } else {
                lookup_array.iter().position(|v| values_equal(v, search_key))
            };
            match pos {
                Some(idx) => Value::Number((idx + 1) as f64),
                None => Value::Error(ErrorKind::NA),
            }
        }
        1 => {
            let mut best_pos: Option<usize> = None;
            let mut best_val: Option<&Value> = None;
            for (i, v) in lookup_array.iter().enumerate() {
                if values_equal(v, search_key) {
                    return Value::Number((i + 1) as f64);
                }
                if let Some(std::cmp::Ordering::Greater) = value_compare(v, search_key) {
                    let is_better = match best_val {
                        None => true,
                        Some(bv) => value_compare(v, bv) == Some(std::cmp::Ordering::Less),
                    };
                    if is_better {
                        best_pos = Some(i + 1);
                        best_val = Some(v);
                    }
                }
            }
            match best_pos {
                Some(pos) => Value::Number(pos as f64),
                None => Value::Error(ErrorKind::NA),
            }
        }
        -1 => {
            let mut best_pos: Option<usize> = None;
            let mut best_val: Option<&Value> = None;
            for (i, v) in lookup_array.iter().enumerate() {
                match value_compare(v, search_key) {
                    Some(std::cmp::Ordering::Less) | Some(std::cmp::Ordering::Equal) => {
                        let is_better = match best_val {
                            None => true,
                            Some(bv) => value_compare(v, bv) == Some(std::cmp::Ordering::Greater),
                        };
                        if is_better {
                            best_pos = Some(i + 1);
                            best_val = Some(v);
                        }
                    }
                    _ => {}
                }
            }
            match best_pos {
                Some(pos) => Value::Number(pos as f64),
                None => Value::Error(ErrorKind::NA),
            }
        }
        2 => {
            let pos = if has_wildcards(search_key) {
                if search_mode == -1 {
                    lookup_array.iter().rposition(|v| wildcard_match_value(search_key, v))
                } else {
                    lookup_array.iter().position(|v| wildcard_match_value(search_key, v))
                }
            } else if search_mode == -1 {
                lookup_array.iter().rposition(|v| values_equal(v, search_key))
            } else {
                lookup_array.iter().position(|v| values_equal(v, search_key))
            };
            match pos {
                Some(idx) => Value::Number((idx + 1) as f64),
                None => Value::Error(ErrorKind::NA),
            }
        }
        _ => Value::Error(ErrorKind::Value),
    }
}
