use crate::eval::functions::check_arity;
use crate::types::{ErrorKind, Value};

/// `RANK(number, ref, [order])` — rank of number in ref.
/// order=0 (default) → descending rank. order≠0 → ascending. Ties get lowest rank.
/// If number not found in ref: `#N/A`.
pub fn rank_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 2, 3) {
        return err;
    }
    let x = match &args[0] {
        Value::Number(n) => *n,
        _ => return Value::Error(ErrorKind::NA),
    };
    let nums = collect_numbers(&args[1]);
    let ascending = args.get(2).map(|v| match v {
        Value::Number(n) => *n != 0.0,
        _ => false,
    }).unwrap_or(false);

    rank_eq_impl(x, &nums, ascending)
}

fn rank_eq_impl(x: f64, nums: &[f64], ascending: bool) -> Value {
    if nums.is_empty() {
        return Value::Error(ErrorKind::NA);
    }
    // x must be in nums
    if !nums.contains(&x) {
        return Value::Error(ErrorKind::NA);
    }
    let rank = if ascending {
        // ascending: rank = count of values < x, + 1
        nums.iter().filter(|&&n| n < x).count() + 1
    } else {
        // descending: rank = count of values > x, + 1
        nums.iter().filter(|&&n| n > x).count() + 1
    };
    Value::Number(rank as f64)
}

fn collect_numbers(v: &Value) -> Vec<f64> {
    match v {
        Value::Array(arr) => arr.iter().filter_map(|x| {
            if let Value::Number(n) = x { Some(*n) } else { None }
        }).collect(),
        Value::Number(n) => vec![*n],
        _ => vec![],
    }
}

#[cfg(test)]
mod tests;
