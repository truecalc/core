use crate::eval::functions::check_arity;
use crate::types::{ErrorKind, Value};
use super::percentile_inc::collect_numbers;

/// `PERCENTILE.EXC(array, k)` — exclusive percentile, k strictly in (0,1).
/// Uses index = k*(n+1)-1 formula for interpolation.
pub fn percentile_exc_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 2, 2) {
        return err;
    }
    let k = match &args[1] {
        Value::Number(n) => *n,
        _ => return Value::Error(ErrorKind::Num),
    };
    if k <= 0.0 || k >= 1.0 {
        return Value::Error(ErrorKind::Num);
    }
    let mut nums = collect_numbers(&args[0]);
    if nums.is_empty() {
        return Value::Error(ErrorKind::Num);
    }
    nums.sort_by(|a, b| a.partial_cmp(b).unwrap());
    match percentile_exc_calc(&nums, k) {
        Some(v) => Value::Number(v),
        None => Value::Error(ErrorKind::Num),
    }
}

/// Calculate exclusive percentile for a sorted slice.
/// Returns None if k is out of valid range for the dataset.
pub fn percentile_exc_calc(sorted: &[f64], k: f64) -> Option<f64> {
    let n = sorted.len();
    let idx = k * (n + 1) as f64 - 1.0;
    if idx < 0.0 || idx > (n - 1) as f64 {
        return None;
    }
    let lo = idx.floor() as usize;
    let hi = idx.ceil() as usize;
    let result = if lo == hi {
        sorted[lo]
    } else {
        let frac = idx - lo as f64;
        sorted[lo] * (1.0 - frac) + sorted[hi] * frac
    };
    Some(result)
}

#[cfg(test)]
mod tests;
