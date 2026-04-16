use crate::types::{ErrorKind, Value};
use super::criterion::{flatten_to_vec, matches_criterion, parse_criterion};

/// `SUMIFS(sum_range, range1, criterion1, [range2, criterion2, ...])` — sum
/// values in `sum_range` where all (range, criterion) pairs match.
///
/// Requires at least 3 arguments: sum_range + one (range, criterion) pair.
/// After sum_range, remaining args must come in (range, criterion) pairs (even count).
pub fn sumifs_fn(args: &[Value]) -> Value {
    // Need at least 3 args, and (args.len() - 1) must be even → args.len() odd and >= 3
    if args.len() < 3 || args.len().is_multiple_of(2) {
        return Value::Error(ErrorKind::NA);
    }

    let sum_range = flatten_to_vec(&args[0]);

    // Build (range_vec, criterion) pairs from remaining args.
    let pairs: Vec<(Vec<&Value>, _)> = args[1..]
        .chunks(2)
        .map(|chunk| {
            let range = flatten_to_vec(&chunk[0]);
            let crit = parse_criterion(&chunk[1]);
            (range, crit)
        })
        .collect();

    let mut total = 0.0_f64;

    for (i, s_val) in sum_range.iter().enumerate() {
        if pairs.iter().all(|(range, crit)| {
            range.get(i).is_some_and(|v| matches_criterion(v, crit))
        }) {
            match s_val {
                Value::Number(n) => total += n,
                Value::Bool(b) => total += if *b { 1.0 } else { 0.0 },
                Value::Text(s) => {
                    if let Ok(n) = s.parse::<f64>() {
                        total += n;
                    }
                }
                _ => {}
            }
        }
    }

    if !total.is_finite() {
        return Value::Error(ErrorKind::Num);
    }
    Value::Number(total)
}

#[cfg(test)]
mod tests;
