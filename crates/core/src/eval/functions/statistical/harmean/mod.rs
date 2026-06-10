use crate::types::{ErrorKind, Value};

/// `HARMEAN(value1, ...)` — harmonic mean: n / Σ(1/x_i).
/// Flattens `Value::Array` args. Ignores Text, Bool, Empty.
/// All values must be > 0, else `#NUM!`. Requires at least 1 value.
pub fn harmean_fn(args: &[Value]) -> Value {
    if args.is_empty() {
        return Value::Error(ErrorKind::NA);
    }
    let mut nums: Vec<f64> = Vec::new();
    for arg in args {
        match arg {
            Value::Number(n) => nums.push(*n),
            Value::Bool(b) => nums.push(if *b { 1.0 } else { 0.0 }),
            Value::Text(s) => {
                let trimmed = s.trim();
                match trimmed.parse::<f64>() {
                    Ok(v) if v.is_finite() => nums.push(v),
                    _ => return Value::Error(ErrorKind::Value),
                }
            }
            Value::Empty => {}
            Value::Array(arr) => {
                for v in arr {
                    if let Value::Number(n) = v {
                        nums.push(*n);
                    }
                }
            }
            Value::Error(e) => return Value::Error(e.clone()),
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
