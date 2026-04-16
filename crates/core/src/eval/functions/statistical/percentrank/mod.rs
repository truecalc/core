use crate::eval::functions::check_arity;
use crate::types::{ErrorKind, Value};
use super::percentile_inc::collect_numbers;
use super::percentrank_inc::{percentrank_inc_calc, round_to_sig};

/// `PERCENTRANK(array, x, [significance])` — inclusive percentile rank (same as PERCENTRANK.INC).
pub fn percentrank_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 2, 3) {
        return err;
    }
    // args[0] = array, args[1] = x, args[2] = significance (optional)
    let x = match &args[1] {
        Value::Number(n) => *n,
        _ => return Value::Error(ErrorKind::NA),
    };
    let sig = args.get(2).map(|v| match v {
        Value::Number(n) => (*n as usize).max(1),
        _ => 3,
    }).unwrap_or(3);

    let mut nums = collect_numbers(&args[0]);
    if nums.is_empty() {
        return Value::Error(ErrorKind::NA);
    }
    nums.sort_by(|a, b| a.partial_cmp(b).unwrap());

    let min = nums[0];
    let max = *nums.last().unwrap();

    if x < min || x > max {
        return Value::Error(ErrorKind::NA);
    }

    let n = nums.len();
    if n == 1 {
        return Value::Number(round_to_sig(1.0, sig));
    }

    let result = percentrank_inc_calc(&nums, x);
    Value::Number(round_to_sig(result, sig))
}

#[cfg(test)]
mod tests;
