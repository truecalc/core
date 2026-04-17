use crate::eval::functions::check_arity;
use crate::types::{ErrorKind, Value};
use super::array_utils::{flatten_to_flat, values_equal, value_compare};

/// `LOOKUP(search_key, search_range, [result_range])`
/// Approximate lookup in sorted range (binary search semantics, but linear scan OK).
pub fn lookup_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 2, 3) {
        return err;
    }

    let search_key = &args[0];
    let search_range = flatten_to_flat(&args[1]);
    let result_range: Option<Vec<Value>> = if args.len() == 3 {
        Some(flatten_to_flat(&args[2]))
    } else {
        None
    };

    // Largest value <= search_key
    let mut found_idx: Option<usize> = None;
    for (i, v) in search_range.iter().enumerate() {
        match value_compare(v, search_key) {
            Some(std::cmp::Ordering::Less) | Some(std::cmp::Ordering::Equal) => {
                found_idx = Some(i);
            }
            Some(std::cmp::Ordering::Greater) => break,
            None => {}
        }
    }

    match found_idx {
        None => Value::Error(ErrorKind::NA),
        Some(idx) => match &result_range {
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

/// `XLOOKUP(search_key, lookup_array, return_array, [if_not_found], [match_mode], [search_mode])`
pub fn xlookup_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 3, 6) {
        return err;
    }

    let search_key = &args[0];
    let lookup_array = flatten_to_flat(&args[1]);
    let return_array = flatten_to_flat(&args[2]);
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

    let result_idx = match match_mode {
        0 => lookup_array.iter().position(|v| values_equal(v, search_key)),
        1 => {
            // Next larger or equal
            let mut res: Option<usize> = None;
            for (i, v) in lookup_array.iter().enumerate() {
                if values_equal(v, search_key) { res = Some(i); break; }
                if let Some(std::cmp::Ordering::Greater) = value_compare(v, search_key) {
                    res = Some(i);
                    break;
                }
            }
            res
        }
        -1 => {
            // Next smaller or equal
            let mut res: Option<usize> = None;
            for (i, v) in lookup_array.iter().enumerate() {
                if values_equal(v, search_key) { res = Some(i); break; }
                match value_compare(v, search_key) {
                    Some(std::cmp::Ordering::Less) | Some(std::cmp::Ordering::Equal) => {
                        res = Some(i);
                    }
                    Some(std::cmp::Ordering::Greater) => break,
                    _ => {}
                }
            }
            res
        }
        _ => lookup_array.iter().position(|v| values_equal(v, search_key)),
    };

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

    match match_mode {
        0 => {
            // Exact match
            match lookup_array.iter().position(|v| values_equal(v, search_key)) {
                Some(idx) => Value::Number((idx + 1) as f64),
                None => Value::Error(ErrorKind::NA),
            }
        }
        1 => {
            // Less than or equal (sorted ascending)
            let mut result: Option<usize> = None;
            for (i, v) in lookup_array.iter().enumerate() {
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
            // Greater than or equal (sorted descending)
            let mut result: Option<usize> = None;
            for (i, v) in lookup_array.iter().enumerate() {
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
