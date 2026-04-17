use crate::eval::evaluate_expr;
use crate::eval::functions::{check_arity_len, EvalCtx};
use crate::parser::ast::Expr;
use crate::types::{ErrorKind, Value};
use super::criterion::{flatten_to_vec, matches_criterion, parse_criterion};

/// `SUMIF(range, criterion, [sum_range])` — sum the `sum_range` elements for which
/// the corresponding `range` element matches `criterion`.
///
/// Returns `#N/A` when `range` or `sum_range` is a literal array constant (`{...}`).
pub fn sumif_fn(args: &[Expr], ctx: &mut EvalCtx<'_>) -> Value {
    if let Some(err) = check_arity_len(args.len(), 2, 3) {
        return err;
    }
    // When sum_range is provided (3-arg form), literal array constants are not valid
    // as either range or sum_range — Google Sheets returns #N/A.
    if args.len() == 3 && (matches!(args[0], Expr::Array(..)) || matches!(args[2], Expr::Array(..))) {
        return Value::Error(ErrorKind::NA);
    }

    let range_val = evaluate_expr(&args[0], ctx);
    if matches!(range_val, Value::Error(_)) {
        return range_val;
    }
    let crit_val = evaluate_expr(&args[1], ctx);
    if matches!(crit_val, Value::Error(_)) {
        return crit_val;
    }

    let range = flatten_to_vec(&range_val);
    let crit = parse_criterion(&crit_val);

    let sum_range_val;
    let sum_range: Vec<&Value> = if args.len() == 3 {
        sum_range_val = evaluate_expr(&args[2], ctx);
        if matches!(sum_range_val, Value::Error(_)) {
            return sum_range_val;
        }
        flatten_to_vec(&sum_range_val)
    } else {
        range.clone()
    };

    let mut total = 0.0_f64;
    for (r_val, s_val) in range.iter().zip(sum_range.iter()) {
        if matches_criterion(r_val, &crit) {
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
