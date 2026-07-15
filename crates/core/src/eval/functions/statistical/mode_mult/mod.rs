use crate::types::{ErrorKind, Value};

/// `MODE.MULT(value1, ...)` — returns every value tied for the highest
/// frequency, sorted ascending (a single tied value collapses to a plain
/// scalar, matching this engine's general 1x1-array convention).
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
                        Value::ErrorMsg(e, m) => return Value::ErrorMsg(e.clone(), m.clone()),
                        _ => {}
                    }
                }
            }
            Value::Error(e) => return Value::Error(e.clone()),
            Value::ErrorMsg(e, m) => return Value::ErrorMsg(e.clone(), m.clone()),
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
    // Every distinct value tied for max_count, sorted ascending (confirmed
    // against live Google Sheets: {3,3,1,1,2} → {1;3}, {5,3,3,5,1} → {3;5} —
    // both counter to first-appearance order).
    let mut seen: std::collections::HashSet<u64> = std::collections::HashSet::new();
    let mut modes: Vec<f64> = Vec::new();
    for &n in &nums {
        let bits = n.to_bits();
        if *freq.get(&bits).unwrap_or(&0) == max_count && seen.insert(bits) {
            modes.push(n);
        }
    }
    modes.sort_by(|a, b| a.partial_cmp(b).unwrap());
    match modes.len() {
        0 => Value::Error(ErrorKind::NA),
        1 => Value::Number(modes[0]),
        _ => Value::Array(modes.into_iter().map(|n| Value::Array(vec![Value::Number(n)])).collect()),
    }
}

#[cfg(test)]
mod tests;
