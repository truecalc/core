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
        Value::Bool(b) => if *b { 1.0 } else { 0.0 },
        _ => return Value::Error(ErrorKind::NA),
    };
    let nums = match collect_numbers_checked(&args[1]) {
        Ok(v) => v,
        Err(e) => return e,
    };
    let ascending = args.get(2).map(|v| match v {
        Value::Number(n) => *n != 0.0,
        Value::Bool(b) => *b,
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

fn collect_numbers_checked(v: &Value) -> Result<Vec<f64>, Value> {
    match v {
        Value::Array(arr) => {
            let mut nums = Vec::new();
            for x in arr {
                match x {
                    Value::Number(n) => nums.push(*n),
                    Value::Error(e) => return Err(Value::Error(e.clone())),
                    _ => {}
                }
            }
            Ok(nums)
        }
        Value::Number(n) => Ok(vec![*n]),
        Value::Error(e) => Err(Value::Error(e.clone())),
        _ => Ok(vec![]),
    }
}

#[cfg(test)]
mod tests;
