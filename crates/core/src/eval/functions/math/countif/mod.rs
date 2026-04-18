use crate::eval::functions::check_arity;
use crate::types::Value;
use super::criterion::{flatten_to_vec, matches_criterion, parse_criterion};

/// `COUNTIF(range, criterion)` — count elements in `range` that match `criterion`.
///
/// `range` may be a `Value::Array` (flattened) or a scalar.
/// `criterion` supports numeric comparisons (`>N`, `>=N`, `<N`, `<=N`, `<>N`, `=N`),
/// exact text match (case-insensitive), and wildcard patterns (`*`, `?`).
pub fn countif_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 2, 2) {
        return err;
    }
    let range = flatten_to_vec(&args[0]);
    let crit = parse_criterion(&args[1]);
    let count = range.iter().filter(|v| matches_criterion(v, &crit)).count();
    Value::Number(count as f64)
}

#[cfg(test)]
mod tests;
