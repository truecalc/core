use crate::types::{ErrorKind, Value};
use super::stat_helpers::collect_nums;

/// `COVARIANCE.S(array1, array2)` — sample covariance of two arrays.
/// Requires exactly 2 args of equal length ≥2. Returns `#N/A` if lengths differ.
/// Returns `#DIV/0!` if n < 2.
pub fn covariance_s_fn(args: &[Value]) -> Value {
    if args.len() != 2 {
        return Value::Error(ErrorKind::NA);
    }
    let xs = collect_nums(std::slice::from_ref(&args[0]));
    let ys = collect_nums(std::slice::from_ref(&args[1]));
    if xs.len() != ys.len() {
        return Value::Error(ErrorKind::NA);
    }
    let n = xs.len();
    if n < 2 {
        return Value::Error(ErrorKind::Num);
    }
    let mean_x = xs.iter().sum::<f64>() / n as f64;
    let mean_y = ys.iter().sum::<f64>() / n as f64;
    let cov = xs.iter().zip(ys.iter())
        .map(|(x, y)| (x - mean_x) * (y - mean_y))
        .sum::<f64>() / (n - 1) as f64;
    if !cov.is_finite() {
        return Value::Error(ErrorKind::Num);
    }
    Value::Number(cov)
}

#[cfg(test)]
mod tests;
