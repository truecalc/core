use crate::eval::coercion::to_number;
use crate::eval::functions::check_arity;
use crate::types::{ErrorKind, Value};

pub fn product_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 1, 255) {
        return err;
    }
    let mut product = 1.0_f64;
    let mut contributed = false;
    for arg in args {
        match product_top_level(arg) {
            Err(e) => return e,
            // A skipped argument is *absent*, not a factor of 1: with nothing
            // left to multiply, PRODUCT is 0, not the multiplicative identity
            // (google.tsv: `=PRODUCT(SPARKLINE({1,2,3}))` is 0, matching SUM,
            // MAX and COUNT of a lone sparkline, while `=PRODUCT(S,3)` is 3).
            Ok(None) => {}
            Ok(Some(n)) => {
                product *= n;
                contributed = true;
            }
        }
    }
    if !contributed {
        return Value::Number(0.0);
    }
    if !product.is_finite() {
        return Value::Error(ErrorKind::Num);
    }
    Value::Number(product)
}

/// Top-level: arrays use array-context (booleans/text skipped).
/// Direct scalars use full coercion.
///
/// `Ok(None)` means the argument contributed no factor at all — only a
/// sparkline does that, and only as a direct argument. An array still yields
/// `Ok(Some(_))` even when array-context rules skipped everything inside it,
/// which is pre-existing behaviour this does not disturb.
fn product_top_level(v: &Value) -> Result<Option<f64>, Value> {
    match v {
        Value::Array(_) => product_array_value(v),
        // Aggregates skip a sparkline in any position, direct argument or not
        // (google.tsv: `=PRODUCT(SPARKLINE({1,2,3}),3)` is 3 and
        // `=PRODUCT(SPARKLINE({1,2,3}))` is 0).
        Value::Sparkline(_) => Ok(None),
        other => to_number(other.clone()).map(Some),
    }
}

/// Array-context: booleans, text, empty silently skipped (contribute 1).
///
/// `Ok(None)` means "nothing here contributed a factor", which only a sparkline
/// produces — an aggregate's answer must not depend on whether the sparkline
/// arrived directly or through a range (google.tsv: `=PRODUCT(K1:K1)` over a
/// cell holding a sparkline is 0, the same as `=PRODUCT(SPARKLINE({1,2,3}))`).
/// Booleans, text and blanks keep contributing the identity factor 1 as before,
/// and an empty array is not a skip.
fn product_array_value(v: &Value) -> Result<Option<f64>, Value> {
    match v {
        Value::Array(elems) => {
            if elems.is_empty() {
                return Ok(Some(1.0));
            }
            let mut p = 1.0_f64;
            let mut contributed = false;
            for elem in elems {
                if let Some(n) = product_array_value(elem)? {
                    p *= n;
                    contributed = true;
                }
            }
            Ok(if contributed { Some(p) } else { None })
        }
        Value::Bool(_) | Value::Text(_) | Value::Empty => Ok(Some(1.0)),
        Value::Zoned(_) => Ok(Some(1.0)),
        Value::Sparkline(_) => Ok(None),
        Value::Error(_) | Value::ErrorMsg(_, _) => Err(v.clone()),
        Value::Number(n) | Value::Date(n) => Ok(Some(*n)),
    }
}

#[cfg(test)]
mod tests;
