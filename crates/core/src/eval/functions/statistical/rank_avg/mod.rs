use crate::eval::functions::check_arity;
use crate::types::{ErrorKind, Value};

/// `RANK.AVG(number, ref, [order])` — rank with average rank for ties.
/// order=0 (default) → descending rank. order≠0 → ascending.
/// If number not found in ref: `#N/A`.
pub fn rank_avg_fn(args: &[Value]) -> Value {
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

    if nums.is_empty() || !nums.contains(&x) {
        return Value::Error(ErrorKind::NA);
    }

    // Count values that are strictly better (lower rank position) and ties
    let count_better = if ascending {
        nums.iter().filter(|&&n| n < x).count()
    } else {
        nums.iter().filter(|&&n| n > x).count()
    };
    let count_equal = nums.iter().filter(|&&n| n == x).count();

    // Average rank for ties: low_rank + (count_equal - 1) / 2
    // low_rank = count_better + 1
    // high_rank = count_better + count_equal
    // avg = (low + high) / 2 = count_better + 1 + (count_equal - 1) / 2
    let avg_rank = (count_better + 1) as f64 + (count_equal - 1) as f64 / 2.0;
    Value::Number(avg_rank)
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
