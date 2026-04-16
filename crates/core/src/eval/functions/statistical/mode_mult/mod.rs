use crate::types::{ErrorKind, Value};

fn collect_nums(args: &[Value]) -> Vec<f64> {
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
    nums
}

/// `MODE.MULT(value1, ...)` — returns all most frequently occurring values as an array.
/// If no value appears more than once, returns `#N/A`.
/// If there is a single mode, returns `Value::Number` for that mode.
/// If there are tied modes, returns `Value::Array` of those modes sorted ascending.
pub fn mode_mult_fn(args: &[Value]) -> Value {
    let nums = collect_nums(args);
    if nums.is_empty() {
        return Value::Error(ErrorKind::NA);
    }

    let mut sorted = nums.clone();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());

    // Count frequency of each distinct value
    let mut freq: Vec<(f64, usize)> = Vec::new();
    let mut i = 0;
    while i < sorted.len() {
        let val = sorted[i];
        let mut count = 0;
        while i < sorted.len() && sorted[i] == val {
            count += 1;
            i += 1;
        }
        freq.push((val, count));
    }

    let max_count = freq.iter().map(|(_, c)| *c).max().unwrap_or(0);
    if max_count < 2 {
        return Value::Error(ErrorKind::NA);
    }

    let modes: Vec<f64> = freq.iter()
        .filter(|(_, c)| *c == max_count)
        .map(|(v, _)| *v)
        .collect();

    if modes.len() == 1 {
        Value::Number(modes[0])
    } else {
        Value::Array(modes.into_iter().map(Value::Number).collect())
    }
}

#[cfg(test)]
mod tests;
