use crate::types::{ErrorKind, Value};

/// Collect numeric values from args, flattening arrays.
/// Numbers and Dates are included. Bool/Text/Empty are ignored.
/// Used for range/array contexts where GS skips non-numerics.
pub fn collect_nums(args: &[Value]) -> Vec<f64> {
    let mut nums = Vec::new();
    collect_nums_into(args, &mut nums);
    nums
}

fn collect_nums_into(args: &[Value], out: &mut Vec<f64>) {
    for arg in args {
        match arg {
            Value::Number(n) => out.push(*n),
            Value::Date(n) => out.push(*n),
            Value::Array(inner) => collect_nums_into(inner, out),
            _ => {}
        }
    }
}

/// Collect numeric values respecting Google Sheets direct-arg semantics.
///
/// Rules for top-level (direct scalar) args:
///   - Number/Date  → include
///   - Bool         → coerce (TRUE=1, FALSE=0)
///   - Text that parses as number → coerce
///   - Text that does NOT parse  → return Err(#VALUE!)
///   - Empty        → skip
///
/// Rules for values inside an Array literal `{…}`:
///   - Number/Date  → include
///   - Bool         → skip (Google Sheets skips bools in array context for AVERAGE etc.)
///   - Text         → skip
///   - Empty        → skip
///
/// Returns `Ok(nums)` or `Err(Value::Error(ErrorKind::Value))`.
pub fn collect_nums_direct(args: &[Value]) -> Result<Vec<f64>, Value> {
    let mut nums = Vec::new();
    for arg in args {
        match arg {
            Value::Number(n) => nums.push(*n),
            Value::Date(n) => nums.push(*n),
            Value::Bool(b) => nums.push(if *b { 1.0 } else { 0.0 }),
            Value::Text(s) => {
                let trimmed = s.trim();
                if trimmed.is_empty() {
                    return Err(Value::Error(ErrorKind::Value));
                }
                match trimmed.parse::<f64>() {
                    Ok(v) if v.is_finite() => nums.push(v),
                    _ => return Err(Value::Error(ErrorKind::Value)),
                }
            }
            Value::Empty => {} // skip
            Value::Array(inner) => {
                // Array context: skip non-numeric silently
                collect_nums_into(inner, &mut nums);
            }
            Value::Error(e) => return Err(Value::Error(e.clone())),
        }
    }
    Ok(nums)
}

/// Collect numeric values for AVERAGEA with Google Sheets direct-arg semantics.
///
/// AVERAGEA rules:
///   Direct scalar args:
///     - Number/Date  → include
///     - Bool         → coerce (TRUE=1, FALSE=0)
///     - Text ""      → push 0.0 (empty string counts as 0)
///     - Text parseable → coerce to that value
///     - Text non-parseable non-empty → return Err(#VALUE!)
///     - Empty        → skip
///
///   Array context `{…}`:
///     - Number/Date  → include
///     - Bool         → coerce (TRUE=1, FALSE=0)
///     - Text (any)   → push 0.0 (counts as 0)
///     - Empty        → skip
pub fn collect_nums_a_direct(args: &[Value]) -> Result<Vec<f64>, Value> {
    let mut nums = Vec::new();
    for arg in args {
        match arg {
            Value::Number(n) => nums.push(*n),
            Value::Date(n) => nums.push(*n),
            Value::Bool(b) => nums.push(if *b { 1.0 } else { 0.0 }),
            Value::Text(s) => {
                let trimmed = s.trim();
                if trimmed.is_empty() {
                    // Empty string direct arg → counts as 0 for AVERAGEA
                    nums.push(0.0);
                } else {
                    match trimmed.parse::<f64>() {
                        Ok(v) if v.is_finite() => nums.push(v),
                        // Non-parseable non-empty direct text → #VALUE!
                        _ => return Err(Value::Error(ErrorKind::Value)),
                    }
                }
            }
            Value::Empty => {} // skip
            Value::Array(inner) => {
                // Array context: bools coerce, text→0
                collect_nums_a_into(inner, &mut nums);
            }
            Value::Error(e) => return Err(Value::Error(e.clone())),
        }
    }
    Ok(nums)
}

/// Collect numeric values from args, flattening arrays.
/// "A" variant: also includes Bool (TRUE=1, FALSE=0) and Text as 0.
pub fn collect_nums_a(args: &[Value]) -> Vec<f64> {
    let mut nums = Vec::new();
    collect_nums_a_into(args, &mut nums);
    nums
}

pub fn collect_nums_a_into(args: &[Value], out: &mut Vec<f64>) {
    for arg in args {
        match arg {
            Value::Number(n) => out.push(*n),
            Value::Date(n) => out.push(*n),
            Value::Bool(b) => out.push(if *b { 1.0 } else { 0.0 }),
            Value::Text(_) => out.push(0.0),
            Value::Array(inner) => collect_nums_a_into(inner, out),
            Value::Empty => {}
            Value::Error(_) => {}
        }
    }
}
