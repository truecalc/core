use crate::eval::functions::check_arity;
use crate::types::{ErrorKind, Value};
use super::criterion::{flatten_to_vec, matches_criterion, parse_criterion};

/// `AVERAGEIF(range, criterion, [average_range])` — average the `average_range` elements
/// for which the corresponding `range` element matches `criterion`.
///
/// If `average_range` is omitted, `range` is both the test range and the values to average.
/// Returns `Value::Error(ErrorKind::DivByZero)` when no elements match.
/// Non-finite results return `Value::Error(ErrorKind::Num)`.
pub fn averageif_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 2, 3) {
        return err;
    }
    // Google Sheets returns #N/A when a separate average_range is provided and the
    // range argument is an inline array literal (not a cell reference).
    if args.len() == 3 && matches!(args[0], Value::Array(_)) {
        return Value::Error(ErrorKind::NA);
    }
    let range = flatten_to_vec(&args[0]);
    let crit = parse_criterion(&args[1]);
    let avg_range: Vec<&Value> = if args.len() == 3 {
        flatten_to_vec(&args[2])
    } else {
        range.clone()
    };

    let mut total = 0.0_f64;
    let mut count = 0usize;
    for (r_val, a_val) in range.iter().zip(avg_range.iter()) {
        if matches_criterion(r_val, &crit) {
            match a_val {
                Value::Number(n) => {
                    total += n;
                    count += 1;
                }
                Value::Bool(b) => {
                    total += if *b { 1.0 } else { 0.0 };
                    count += 1;
                }
                Value::Text(s) => {
                    if let Ok(n) = s.parse::<f64>() {
                        total += n;
                        count += 1;
                    }
                    // Non-numeric text: skip (no count increment)
                }
                _ => {} // Empty, Error: skip
            }
        }
    }
    if count == 0 {
        return Value::Error(ErrorKind::DivByZero);
    }
    let avg = total / count as f64;
    if !avg.is_finite() {
        return Value::Error(ErrorKind::Num);
    }
    Value::Number(avg)
}

#[cfg(test)]
mod tests;
