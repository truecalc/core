use crate::eval::functions::check_arity;
use crate::types::{ErrorKind, Value};
use super::percentile_inc::collect_numbers;
use super::percentrank_inc::round_to_sig;

/// `PERCENTRANK.EXC(array, x, [significance])` — exclusive percentile rank.
/// Uses (rank) / (n+1) style. x must be strictly within range (min < x < max).
pub fn percentrank_exc_fn(args: &[Value]) -> Value {
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

    // EXC: x must be strictly within range
    if x < min || x > max {
        return Value::Error(ErrorKind::NA);
    }

    let n = nums.len();
    let result = percentrank_exc_calc(&nums, x, n);
    Value::Number(round_to_sig(result, sig))
}

#[cfg(test)]
mod tests;

pub fn percentrank_exc_calc(sorted: &[f64], x: f64, n: usize) -> f64 {
    // EXC formula: rank / (n+1) where values get 1-based rank
    let count_below = sorted.iter().filter(|&&v| v < x).count();
    let count_equal = sorted.iter().filter(|&&v| v == x).count();

    if count_equal > 0 {
        // x is exactly in data: rank = count_below + 1
        (count_below + 1) as f64 / (n + 1) as f64
    } else {
        // x is between sorted[count_below-1] and sorted[count_below]
        let lo_rank = count_below as f64 / (n + 1) as f64;
        let hi_rank = (count_below + 1) as f64 / (n + 1) as f64;
        let lo_val = sorted[count_below - 1];
        let hi_val = sorted[count_below];
        let frac = (x - lo_val) / (hi_val - lo_val);
        lo_rank + frac * (hi_rank - lo_rank)
    }
}
