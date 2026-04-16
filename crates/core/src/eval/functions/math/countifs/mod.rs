use crate::types::{ErrorKind, Value};
use super::criterion::{flatten_to_vec, matches_criterion, parse_criterion};

/// `COUNTIFS(range1, criterion1, [range2, criterion2, ...])` — count rows where
/// all (range, criterion) pairs match simultaneously.
///
/// Arguments must come in pairs: even total count >= 2.
/// All ranges must have the same length; if they differ, zip stops at the shortest.
pub fn countifs_fn(args: &[Value]) -> Value {
    // Need at least 2 args, and even count.
    if args.len() < 2 || !args.len().is_multiple_of(2) {
        return Value::Error(ErrorKind::NA);
    }

    // Build (range_vec, criterion) pairs.
    let pairs: Vec<(Vec<&Value>, _)> = args
        .chunks(2)
        .map(|chunk| {
            let range = flatten_to_vec(&chunk[0]);
            let crit = parse_criterion(&chunk[1]);
            (range, crit)
        })
        .collect();

    // Use the first range's length as the row count.
    let row_count = pairs[0].0.len();

    let mut count = 0usize;
    for i in 0..row_count {
        if pairs.iter().all(|(range, crit)| {
            range.get(i).is_some_and(|v| matches_criterion(v, crit))
        }) {
            count += 1;
        }
    }

    Value::Number(count as f64)
}

#[cfg(test)]
mod tests;
