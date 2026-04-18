use crate::types::{ErrorKind, Value};

/// `GEOMEAN(value1, ...)` — geometric mean of numeric arguments.
/// Flattens `Value::Array` args. Ignores Text, Bool, Empty.
/// All values must be > 0, else `#NUM!`. Requires at least 1 value.
pub fn geomean_fn(args: &[Value]) -> Value {
    let mut nums: Vec<f64> = Vec::new();
    for arg in args {
        match arg {
            Value::Number(n) => nums.push(*n),
            Value::Array(arr) => {
                for v in arr {
                    if let Value::Number(n) = v {
                        nums.push(*n);
                    }
                }
            }
            _ => {}
        }
    }
    if args.is_empty() {
        return Value::Error(ErrorKind::NA);
    }
    if nums.is_empty() {
        return Value::Error(ErrorKind::Num);
    }
    let mut log_sum = 0.0_f64;
    for &x in &nums {
        if x <= 0.0 {
            return Value::Error(ErrorKind::Num);
        }
        log_sum += x.ln();
    }
    let result = (log_sum / nums.len() as f64).exp();
    if !result.is_finite() {
        return Value::Error(ErrorKind::Num);
    }
    Value::Number(result)
}

#[cfg(test)]
mod tests;
