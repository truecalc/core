use crate::types::{ErrorKind, Value};

/// Collect numeric values from args (flattening Arrays). Returns sorted Vec<f64>.
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

/// Find the most frequent value. If tie, return the smallest. If all unique or no values: `#N/A`.
pub fn mode_single(nums: &[f64]) -> Value {
    if nums.is_empty() {
        return Value::Error(ErrorKind::NA);
    }
    // Sort a copy to count frequencies
    let mut sorted = nums.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());

    let mut best_val = sorted[0];
    let mut best_count = 0usize;
    let mut cur_val = sorted[0];
    let mut cur_count = 0usize;

    for &x in &sorted {
        if x == cur_val {
            cur_count += 1;
        } else {
            if cur_count > best_count {
                best_count = cur_count;
                best_val = cur_val;
            }
            cur_val = x;
            cur_count = 1;
        }
    }
    // Check last run
    if cur_count > best_count {
        best_count = cur_count;
        best_val = cur_val;
    }

    if best_count < 2 {
        return Value::Error(ErrorKind::NA);
    }
    Value::Number(best_val)
}

/// `MODE(value1, ...)` — most frequently occurring value.
/// If tie, returns smallest. If all unique or no values: `#N/A`.
/// Flattens `Value::Array` args. Ignores Text, Bool, Empty.
pub fn mode_fn(args: &[Value]) -> Value {
    let nums = collect_nums(args);
    mode_single(&nums)
}

#[cfg(test)]
mod tests;
