use crate::eval::evaluate_expr;
use crate::eval::functions::EvalCtx;
use crate::parser::ast::Expr;
use crate::types::{ErrorKind, Value};
use super::criterion::{flatten_to_vec, matches_criterion, parse_criterion};

/// `SUMIFS(sum_range, range1, criterion1, [range2, criterion2, ...])` — sum
/// values in `sum_range` where all (range, criterion) pairs match.
///
/// Supports inline array literals as range arguments.
/// Requires at least 3 arguments: sum_range + one (range, criterion) pair.
pub fn sumifs_fn(args: &[Expr], ctx: &mut EvalCtx<'_>) -> Value {
    // Need at least 3 args, and (args.len() - 1) must be even → args.len() odd and >= 3
    if args.len() < 3 || args.len().is_multiple_of(2) {
        return Value::Error(ErrorKind::NA);
    }

    let sum_range_val = evaluate_expr(&args[0], ctx);
    if matches!(sum_range_val, Value::Error(_)) {
        return sum_range_val;
    }
    let sum_range = flatten_to_vec(&sum_range_val);

    let mut range_vals = Vec::new();
    let mut crit_vals = Vec::new();
    for chunk in args[1..].chunks(2) {
        let rv = evaluate_expr(&chunk[0], ctx);
        if matches!(rv, Value::Error(_)) {
            return rv;
        }
        let cv = evaluate_expr(&chunk[1], ctx);
        if matches!(cv, Value::Error(_)) {
            return cv;
        }
        range_vals.push(rv);
        crit_vals.push(cv);
    }

    let pairs: Vec<(Vec<&Value>, _)> = range_vals.iter()
        .zip(crit_vals.iter())
        .map(|(rv, cv)| (flatten_to_vec(rv), parse_criterion(cv)))
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
