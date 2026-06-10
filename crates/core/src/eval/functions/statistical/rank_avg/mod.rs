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
