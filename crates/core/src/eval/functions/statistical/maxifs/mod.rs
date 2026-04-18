use crate::eval::functions::math::criterion::{flatten_to_vec, matches_criterion, parse_criterion};
use crate::types::{ErrorKind, Value};

/// `MAXIFS(max_range, criteria_range1, criteria1, ...)` — maximum value in `max_range`
/// where all criteria match.
///
/// Google Sheets returns `#N/A` when any range argument is an inline array literal.
/// Returns `0.0` when no rows match (for cell-range inputs).
/// Requires at least 3 args; total args must be odd.
pub fn maxifs_fn(args: &[Value]) -> Value {
    // Require at least 3 args and an odd count.
    if args.len() < 3 || args.len().is_multiple_of(2) {
        return Value::Error(ErrorKind::NA);
    }
    // Google Sheets returns #N/A when any range is an inline array literal.
    if args.iter().step_by(2).any(|a| matches!(a, Value::Array(_))) {
        return Value::Error(ErrorKind::NA);
    }

    let max_range = flatten_to_vec(&args[0]);
    let num_criteria = (args.len() - 1) / 2;

    let mut result: Option<f64> = None;

    'outer: for (i, m_val) in max_range.iter().enumerate() {
        for k in 0..num_criteria {
            let crit_range = flatten_to_vec(&args[1 + k * 2]);
            let crit = parse_criterion(&args[2 + k * 2]);
            let crit_val = crit_range.get(i).copied().unwrap_or(&Value::Empty);
            if !matches_criterion(crit_val, &crit) {
                continue 'outer;
            }
        }
        if let Value::Number(n) = m_val {
            result = Some(match result {
                None      => *n,
                Some(cur) => cur.max(*n),
            });
        }
    }

    Value::Number(result.unwrap_or(0.0))
}

#[cfg(test)]
mod tests;
