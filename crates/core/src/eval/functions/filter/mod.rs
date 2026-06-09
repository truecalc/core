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

    // Length mismatch between array and include -> #N/A (Google Sheets behaviour)
    if arr_elems.len() != inc_elems.len() {
        return Value::Error(ErrorKind::NA);
    }

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
        // Unwrap single-element inner arrays (column arrays from {a;b;c} syntax)
        Value::Array(items) if items.len() == 1 => is_truthy(&items[0]),
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

fn compare_sort_values(a: &Value, b: &Value) -> std::cmp::Ordering {
    match (a, b) {
        (Value::Number(x), Value::Number(y)) => x.partial_cmp(y).unwrap_or(std::cmp::Ordering::Equal),
        (Value::Text(x), Value::Text(y)) => x.cmp(y),
        (Value::Bool(x), Value::Bool(y)) => x.cmp(y),
        _ => std::cmp::Ordering::Equal,
    }
}

/// `SORTN(array, [n], [display_ties_mode], [sort_column], [is_ascending])` ---
/// returns the top N rows of a sorted array.
///
/// Google Sheets semantics: a flat 1-D row array counts as **one row**, so
/// `n` limits the number of rows to return, not the number of elements.
/// With one row any valid `n >= 1` returns that single row unchanged.
///
/// Validation:
/// - `n = 0` or `n < 0`  -> #VALUE!
/// - `display_ties_mode` outside 0-3 -> #VALUE!
pub fn sortn_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 1, 5) {
        return err;
    }

    // Validate and extract n (must be >= 1 when provided)
    let n_limit: Option<usize> = if let Some(v) = args.get(1) {
        match v {
            Value::Number(n) => {
                let n_val = *n;
                if n_val <= 0.0 {
                    return Value::Error(ErrorKind::Value);
                }
                Some(n_val as usize)
            }
            _ => None,
        }
    } else {
        None
    };

    // Validate ties_mode: must be 0, 1, 2, or 3 when provided
    if let Some(v) = args.get(2) {
        if let Value::Number(m) = v {
            let m_int = *m as i64;
            if !(0..=3).contains(&m_int) {
                return Value::Error(ErrorKind::Value);
            }
        }
    }

    let sort_col = args.get(3).and_then(|v| match v {
        Value::Number(n) => Some(*n as usize),
        _ => None,
    }).unwrap_or(1);

    let ascending = args.get(4).map(|v| match v {
        Value::Bool(b) => *b,
        Value::Number(n) => *n >= 0.0,
        _ => true,
    }).unwrap_or(true);

    match &args[0] {
        Value::Array(outer) => {
            let is_2d = outer.iter().any(|e| matches!(e, Value::Array(_)));
            if is_2d {
                let mut rows: Vec<Value> = outer.clone();
                let col_idx = sort_col.saturating_sub(1);
                rows.sort_by(|a, b| {
                    let va = match a { Value::Array(r) => r.get(col_idx).cloned().unwrap_or(Value::Empty), other => other.clone() };
                    let vb = match b { Value::Array(r) => r.get(col_idx).cloned().unwrap_or(Value::Empty), other => other.clone() };
                    let cmp = compare_sort_values(&va, &vb);
                    if ascending { cmp } else { cmp.reverse() }
                });
                let limit = n_limit.unwrap_or(rows.len()).min(rows.len());
                Value::Array(rows.into_iter().take(limit).collect())
            } else {
                // 1-D flat row array: treated as a single row by Google Sheets.
                // SORTN limits rows; with 1 row, any n >= 1 returns the whole row
                // unchanged (elements within the single row are never rearranged).
                Value::Array(outer.clone())
            }
        }
        other => other.clone(),
    }
}

#[cfg(test)]
mod tests;

pub fn register_filter(registry: &mut Registry) {
    registry.register_eager("FILTER", filter_fn, FunctionMeta { category: "filter", signature: "FILTER(array, include, [if_empty])",  description: "Filter an array by a boolean mask" });
    registry.register_eager("SORTN",  sortn_fn,  FunctionMeta { category: "filter", signature: "SORTN(array, [n], [display_ties_mode], ...)", description: "Return top N rows of an array sorted" });
}
