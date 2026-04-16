use crate::types::{ErrorKind, Value};

/// `HARMEAN(value1, ...)` — harmonic mean: n / Σ(1/x_i).
/// Flattens `Value::Array` args. Ignores Text, Bool, Empty.
/// All values must be > 0, else `#NUM!`. Requires at least 1 value.
pub fn harmean_fn(args: &[Value]) -> Value {
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
    if nums.is_empty() {
        return Value::Error(ErrorKind::Num);
    }
    let mut recip_sum = 0.0_f64;
    for &x in &nums {
        if x <= 0.0 {
            return Value::Error(ErrorKind::Num);
        }
        recip_sum += 1.0 / x;
    }
    let result = nums.len() as f64 / recip_sum;
    if !result.is_finite() {
        return Value::Error(ErrorKind::Num);
    }
    Value::Number(result)
}

#[cfg(test)]
mod tests;
