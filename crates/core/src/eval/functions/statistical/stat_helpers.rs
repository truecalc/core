use crate::types::{ErrorKind, Value, ZonedInstant};

/// Resolve `MIN`/`MAX` over zone-aware values.
///
/// Returns `None` when no `Zoned` value participates (the caller falls through
/// to the numeric path). Otherwise: a propagated error if one is present;
/// `#VALUE!` if `Zoned` is mixed with any numeric-contributing value
/// (Number/Bool/Text); else the earliest (`want_min`) or latest `Zoned`,
/// preserving the winning value's own zone.
pub fn zoned_extreme(args: &[Value], want_min: bool) -> Option<Value> {
    fn walk(
        v: &Value,
        best: &mut Option<ZonedInstant>,
        saw_zoned: &mut bool,
        saw_numeric: &mut bool,
        error: &mut Option<Value>,
        want_min: bool,
    ) {
        match v {
            Value::Zoned(z) => {
                *saw_zoned = true;
                let take = match best.as_ref() {
                    None => true,
                    Some(c) => {
                        if want_min {
                            z.utc_nanos < c.utc_nanos
                        } else {
                            z.utc_nanos > c.utc_nanos
                        }
                    }
                };
                if take {
                    *best = Some((**z).clone());
                }
            }
            Value::Number(_) | Value::Date(_) | Value::Bool(_) | Value::Text(_) => *saw_numeric = true,
            Value::Error(e) => {
                if error.is_none() {
                    *error = Some(Value::Error(e.clone()));
                }
            }
            Value::Empty => {}
            Value::Array(elems) => {
                for e in elems {
                    walk(e, best, saw_zoned, saw_numeric, error, want_min);
                }
            }
        }
    }

    let mut best: Option<ZonedInstant> = None;
    let mut saw_zoned = false;
    let mut saw_numeric = false;
    let mut error: Option<Value> = None;
    for a in args {
        walk(a, &mut best, &mut saw_zoned, &mut saw_numeric, &mut error, want_min);
    }
    if !saw_zoned {
        return None;
    }
    if let Some(e) = error {
        return Some(e);
    }
    if saw_numeric {
        return Some(Value::Error(ErrorKind::Value));
    }
    best.map(|z| Value::Zoned(Box::new(z)))
}

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

/// Like collect_nums_into but propagates errors. Returns Some(err) if an error is found.
fn collect_nums_into_checked(args: &[Value], out: &mut Vec<f64>) -> Option<Value> {
    for arg in args {
        match arg {
            Value::Number(n) => out.push(*n),
            Value::Date(n) => out.push(*n),
            Value::Array(inner) => {
                if let Some(err) = collect_nums_into_checked(inner, out) {
                    return Some(err);
                }
            }
            Value::Error(e) => return Some(Value::Error(e.clone())),
            _ => {}
        }
    }
    None
}

/// Like collect_nums_a_into but propagates errors. Returns Some(err) if an error is found.
fn collect_nums_a_into_checked(args: &[Value], out: &mut Vec<f64>) -> Option<Value> {
    for arg in args {
        match arg {
            Value::Number(n) => out.push(*n),
            Value::Date(n) => out.push(*n),
            Value::Bool(b) => out.push(if *b { 1.0 } else { 0.0 }),
            Value::Text(_) => out.push(0.0),
            Value::Array(inner) => {
                if let Some(err) = collect_nums_a_into_checked(inner, out) {
                    return Some(err);
                }
            }
            Value::Error(e) => return Some(Value::Error(e.clone())),
            Value::Empty => {}
            Value::Zoned(_) => {}
        }
    }
    None
}

/// Collect numbers from a single Value with error propagation.
/// Array elements: Numbers only, errors propagate. Non-numeric scalar → empty.
pub fn collect_numbers_checked(v: &Value) -> Result<Vec<f64>, Value> {
    match v {
        Value::Array(arr) => {
            let mut nums = Vec::new();
            if let Some(err) = collect_nums_into_checked(arr, &mut nums) {
                return Err(err);
            }
            Ok(nums)
        }
        Value::Number(n) => Ok(vec![*n]),
        Value::Error(e) => Err(Value::Error(e.clone())),
        _ => Ok(vec![]),
    }
}

/// Like collect_nums_a_direct but also propagates errors from inside arrays.
pub fn collect_nums_a_checked(args: &[Value]) -> Result<Vec<f64>, Value> {
    let mut nums = Vec::new();
    for arg in args {
        match arg {
            Value::Number(n) => nums.push(*n),
            Value::Date(n) => nums.push(*n),
            Value::Bool(b) => nums.push(if *b { 1.0 } else { 0.0 }),
            Value::Text(s) => {
                let trimmed = s.trim();
                if trimmed.is_empty() {
                    nums.push(0.0);
                } else {
                    match trimmed.parse::<f64>() {
                        Ok(v) if v.is_finite() => nums.push(v),
                        _ => return Err(Value::Error(ErrorKind::Value)),
                    }
                }
            }
            Value::Empty => {}
            Value::Array(inner) => {
                if let Some(err) = collect_nums_a_into_checked(inner, &mut nums) {
                    return Err(err);
                }
            }
            Value::Zoned(_) => return Err(Value::Error(ErrorKind::Value)),
            Value::Error(e) => return Err(Value::Error(e.clone())),
        }
    }
    Ok(nums)
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
                // Array context: skip non-numeric, but propagate errors
                if let Some(err) = collect_nums_into_checked(inner, &mut nums) {
                    return Err(err);
                }
            }
            Value::Zoned(_) => return Err(Value::Error(ErrorKind::Value)),
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
            Value::Zoned(_) => return Err(Value::Error(ErrorKind::Value)),
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
            Value::Zoned(_) => {}
        }
    }
}

/// Collect paired numeric values with GS paired-deletion semantics.
/// Used for CORREL, PEARSON, COVARIANCE, COVAR, SLOPE, INTERCEPT, RSQ, STEYX, FORECAST.
///
/// Each arg is flattened. At each index position:
///   - Error in either → propagate immediately.
///   - Both are Number → include pair.
///   - Otherwise (Bool, Text, Empty in either) → skip pair.
///
/// Returns Err(Ref) if either flattened array is empty.
/// Returns Err(NA) if flattened lengths differ.
pub fn collect_paired(x_arg: &Value, y_arg: &Value) -> Result<(Vec<f64>, Vec<f64>), Value> {
    let xs_raw = flatten_to_values(x_arg);
    let ys_raw = flatten_to_values(y_arg);
    if xs_raw.is_empty() || ys_raw.is_empty() {
        return Err(Value::Error(ErrorKind::Ref));
    }
    if xs_raw.len() != ys_raw.len() {
        return Err(Value::Error(ErrorKind::NA));
    }
    let mut xs = Vec::new();
    let mut ys = Vec::new();
    for (x, y) in xs_raw.iter().zip(ys_raw.iter()) {
        if let Value::Error(e) = x { return Err(Value::Error(e.clone())); }
        if let Value::Error(e) = y { return Err(Value::Error(e.clone())); }
        if let (Value::Number(xn), Value::Number(yn)) = (x, y) {
            xs.push(*xn);
            ys.push(*yn);
        }
        // Any non-Number (Bool, Text, Empty) in either position → skip pair
    }
    Ok((xs, ys))
}

fn flatten_to_values(v: &Value) -> Vec<Value> {
    match v {
        Value::Array(inner) => {
            let mut out = Vec::new();
            flatten_array_into(inner, &mut out);
            out
        }
        other => vec![other.clone()],
    }
}

fn flatten_array_into(arr: &[Value], out: &mut Vec<Value>) {
    for v in arr {
        match v {
            Value::Array(inner) => flatten_array_into(inner, out),
            other => out.push(other.clone()),
        }
    }
}
