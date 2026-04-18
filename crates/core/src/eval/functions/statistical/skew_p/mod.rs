use crate::types::{ErrorKind, Value};

/// `SKEW.P(value1, ...)` — population skewness.
/// Formula: (1/n) * Σ((x-mean)/σ)^3
/// where σ is the population standard deviation (sqrt(Σ(x-mean)²/n)).
/// Flattens `Value::Array` args. Ignores Text, Bool, Empty.
/// Requires n ≥ 3. Returns `#N/A` if n=0, `#DIV/0!` if n < 3 or σ = 0.
pub fn skew_p_fn(args: &[Value]) -> Value {
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
    let pop_variance = nums.iter().map(|&x| (x - mean).powi(2)).sum::<f64>() / n as f64;
    let sigma = pop_variance.sqrt();
    if sigma == 0.0 {
        return Value::Error(ErrorKind::DivByZero);
    }
    let nf = n as f64;
    let sum3 = nums.iter().map(|&x| ((x - mean) / sigma).powi(3)).sum::<f64>();
    let result = sum3 / nf;
    if !result.is_finite() {
        return Value::Error(ErrorKind::Num);
    }
    Value::Number(result)
}

#[cfg(test)]
mod tests;
