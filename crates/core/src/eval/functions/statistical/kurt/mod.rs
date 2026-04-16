use crate::types::{ErrorKind, Value};

/// `KURT(value1, ...)` — excess kurtosis (Fisher's definition).
/// Formula: (n(n+1)/((n-1)(n-2)(n-3))) * Σ((x-mean)/s)^4 - 3(n-1)^2/((n-2)(n-3))
/// where s is the sample standard deviation.
/// Flattens `Value::Array` args. Ignores Text, Bool, Empty.
/// Requires n ≥ 4. Returns `#DIV/0!` if n < 4 or s = 0.
pub fn kurt_fn(args: &[Value]) -> Value {
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
    if n < 4 {
        return Value::Error(ErrorKind::DivByZero);
    }
    let mean = nums.iter().sum::<f64>() / n as f64;
    let variance = nums.iter().map(|&x| (x - mean).powi(2)).sum::<f64>() / (n - 1) as f64;
    let s = variance.sqrt();
    if s == 0.0 {
        return Value::Error(ErrorKind::DivByZero);
    }
    let nf = n as f64;
    let sum4 = nums.iter().map(|&x| ((x - mean) / s).powi(4)).sum::<f64>();
    let result = (nf * (nf + 1.0) / ((nf - 1.0) * (nf - 2.0) * (nf - 3.0))) * sum4
        - 3.0 * (nf - 1.0).powi(2) / ((nf - 2.0) * (nf - 3.0));
    if !result.is_finite() {
        return Value::Error(ErrorKind::Num);
    }
    Value::Number(result)
}

#[cfg(test)]
mod tests;
