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

fn compare_sort_values(a: &Value, b: &Value) -> std::cmp::Ordering {
    match (a, b) {
        (Value::Number(x), Value::Number(y)) => x.partial_cmp(y).unwrap_or(std::cmp::Ordering::Equal),
        (Value::Text(x), Value::Text(y)) => x.cmp(y),
        (Value::Bool(x), Value::Bool(y)) => x.cmp(y),
        _ => std::cmp::Ordering::Equal,
    }
}

/// `SORTN(array, [n], [display_ties_mode], [sort_column], [is_ascending])` —
/// returns the top N sorted elements of a 1-D array (or rows of a 2-D array).
pub fn sortn_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 1, 5) {
        return err;
    }

    let n_limit: Option<usize> = args.get(1).and_then(|v| match v {
        Value::Number(n) => Some(*n as usize),
        _ => None,
    });

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
                // 1D: sort elements and take top N
                let mut elems: Vec<Value> = outer.clone();
                elems.sort_by(|a, b| {
                    let cmp = compare_sort_values(a, b);
                    if ascending { cmp } else { cmp.reverse() }
                });
                let limit = n_limit.unwrap_or(elems.len()).min(elems.len());
                Value::Array(elems.into_iter().take(limit).collect())
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
