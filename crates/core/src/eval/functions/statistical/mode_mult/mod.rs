use crate::types::{ErrorKind, Value};

/// `MODE.MULT(value1, ...)` — returns the first (lowest-index) mode.
/// Direct args: Bool/text coerced like MODE.SNGL. Errors propagate.
/// Returns #NA if no value appears more than once.
pub fn mode_mult_fn(args: &[Value]) -> Value {
    if args.is_empty() {
        return Value::Error(ErrorKind::NA);
    }
    // Collect numbers with direct-arg coercion semantics and error propagation.
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
                    match v {
                        Value::Number(n) => nums.push(*n),
                        Value::Error(e) => return Value::Error(e.clone()),
                        _ => {}
                    }
                }
            }
            Value::Error(e) => return Value::Error(e.clone()),
            _ => {}
        }
    }
    if nums.is_empty() {
        return Value::Error(ErrorKind::NA);
    }
    // Find maximum frequency.
    let mut freq: std::collections::HashMap<u64, usize> = std::collections::HashMap::new();
    for &n in &nums {
        *freq.entry(n.to_bits()).or_insert(0) += 1;
    }
    let max_count = *freq.values().max().unwrap_or(&0);
    if max_count < 2 {
        return Value::Error(ErrorKind::NA);
    }
    // Return the smallest value that has max_count occurrences (Google Sheets returns minimum mode).
    let mode = nums.iter()
        .filter(|&&n| *freq.get(&n.to_bits()).unwrap_or(&0) == max_count)
        .cloned()
        .fold(f64::INFINITY, f64::min);
    if mode.is_finite() { Value::Number(mode) } else { Value::Error(ErrorKind::NA) }
}

#[cfg(test)]
mod tests;
