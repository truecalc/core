use crate::types::{ErrorKind, Value};
use super::stat_helpers::collect_paired;

/// `COVARIANCE.P(array1, array2)` — population covariance of two arrays.
pub fn covariance_p_fn(args: &[Value]) -> Value {
    if args.len() != 2 {
        return Value::Error(ErrorKind::NA);
    }
    let (xs, ys) = match collect_paired(&args[0], &args[1]) {
        Ok(v) => v,
        Err(e) => return e,
    };
    let n = xs.len();
    if n == 0 {
        return Value::Error(ErrorKind::Ref);
    }
    let mean_x = xs.iter().sum::<f64>() / n as f64;
    let mean_y = ys.iter().sum::<f64>() / n as f64;
    let cov = xs.iter().zip(ys.iter())
        .map(|(x, y)| (x - mean_x) * (y - mean_y))
        .sum::<f64>() / n as f64;
    if !cov.is_finite() {
        return Value::Error(ErrorKind::Num);
    }
    Value::Number(cov)
}

#[cfg(test)]
mod tests;
