use crate::eval::functions::check_arity;
use crate::types::{ErrorKind, Value};
use super::percentile_inc::collect_numbers;

/// `PERCENTRANK.INC(array, x, [significance])` — inclusive percentile rank.
/// Returns fraction of values in array that are <= x (with interpolation for between values).
/// sig=3 default. If x < min or x > max: `#N/A`.
pub fn percentrank_inc_fn(args: &[Value]) -> Value {
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

    // Formula: (count_below + fraction_within) / (n - 1)
    let result = percentrank_inc_calc(&nums, x);
    Value::Number(round_to_sig(result, sig))
}

pub fn percentrank_inc_calc(sorted: &[f64], x: f64) -> f64 {
    let n = sorted.len();
    let count_below = sorted.iter().filter(|&&v| v < x).count();
    let count_equal = sorted.iter().filter(|&&v| v == x).count();

    if count_equal > 0 {
        // x is exactly in the data
        count_below as f64 / (n - 1) as f64
    } else {
        // x is between sorted[count_below-1] and sorted[count_below]
        let lo_rank = (count_below - 1) as f64 / (n - 1) as f64;
        let hi_rank = count_below as f64 / (n - 1) as f64;
        let lo_val = sorted[count_below - 1];
        let hi_val = sorted[count_below];
        let frac = (x - lo_val) / (hi_val - lo_val);
        lo_rank + frac * (hi_rank - lo_rank)
    }
}

pub fn round_to_sig(v: f64, sig: usize) -> f64 {
    if v == 0.0 {
        return 0.0;
    }
    let factor = 10f64.powi(sig as i32);
    (v * factor).round() / factor
}

#[cfg(test)]
mod tests;
