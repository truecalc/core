use crate::types::{ErrorKind, Value};

/// `SKEW(value1, ...)` — Fisher's sample skewness.
/// Formula: (n/((n-1)(n-2))) * Σ((x-mean)/s)^3
/// where s is the sample standard deviation.
/// Flattens `Value::Array` args. Ignores Text, Bool, Empty.
/// Requires n ≥ 3. Returns `#DIV/0!` if n < 3 or s = 0.
pub fn skew_fn(args: &[Value]) -> Value {
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
    let n = nums.len();
    if n == 0 {
        return Value::Error(ErrorKind::NA);
    }
    if n < 3 {
        return Value::Error(ErrorKind::DivByZero);
    }
    let mean = nums.iter().sum::<f64>() / n as f64;
    let variance = nums.iter().map(|&x| (x - mean).powi(2)).sum::<f64>() / (n - 1) as f64;
    let s = variance.sqrt();
    if s == 0.0 {
        return Value::Error(ErrorKind::DivByZero);
    }
    let nf = n as f64;
    let sum3 = nums.iter().map(|&x| ((x - mean) / s).powi(3)).sum::<f64>();
    let result = (nf / ((nf - 1.0) * (nf - 2.0))) * sum3;
    if !result.is_finite() {
        return Value::Error(ErrorKind::Num);
    }
    Value::Number(result)
}

#[cfg(test)]
mod tests;
