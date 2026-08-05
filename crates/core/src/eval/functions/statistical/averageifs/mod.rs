use crate::eval::functions::math::criterion::{flatten_to_vec, matches_criterion, parse_criterion};
use crate::types::{ErrorKind, Value};

/// `AVERAGEIFS(average_range, criteria_range1, criteria1, ...)` — average values in
/// `average_range` where all criteria match.
///
/// Supports inline array literals as range arguments.
/// Returns `#DIV/0!` when no rows match.
/// Requires at least 3 args; total args must be odd (one average_range + pairs).
pub fn averageifs_fn(args: &[Value]) -> Value {
    // Require at least 3 args and an odd count.
    if args.len() < 3 || args.len().is_multiple_of(2) {
        return Value::Error(ErrorKind::NA);
    }
    // GS requires cell ranges, not inline array literals.
    if args.iter().any(|a| matches!(a, Value::Array(_))) {
        return Value::Error(ErrorKind::NA);
    }
    let avg_range = flatten_to_vec(&args[0]);
    let num_criteria = (args.len() - 1) / 2;

    let mut total = 0.0_f64;
    let mut count = 0usize;

    'outer: for (i, a_val) in avg_range.iter().enumerate() {
        for k in 0..num_criteria {
            let crit_range = flatten_to_vec(&args[1 + k * 2]);
            let crit = parse_criterion(&args[2 + k * 2]);
            let crit_val = crit_range.get(i).copied().unwrap_or(&Value::Empty);
            if !matches_criterion(crit_val, &crit) {
                continue 'outer;
            }
        }
        match a_val {
            Value::Number(n) => {
                total += n;
                count += 1;
            }
            Value::Bool(b) => {
                total += if *b { 1.0 } else { 0.0 };
                count += 1;
            }
            _ => {}
        }
    }

    if count == 0 {
        return Value::Error(ErrorKind::DivByZero);
    }
    Value::Number(total / count as f64)
}

#[cfg(test)]
mod tests;
