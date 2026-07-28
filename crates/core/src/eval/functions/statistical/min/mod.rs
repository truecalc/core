use crate::types::{ErrorKind, Value};

/// `MIN(value1, ...)` — smallest numeric value in the arguments.
/// Direct args: Numbers, Bool (TRUE=1, FALSE=0), parseable text coerced to number.
/// Array elements: Numbers only; text/Bool → skip; errors propagate.
///
/// "No numbers" is *two* rules, not one:
///
/// - an **empty** array argument is `#REF!`: `=MIN({})`. Captured in Google
///   Sheets; the row lands separately, since it fails until this code exists.
/// - a **populated** array holding nothing numeric is 0. The in-repo evidence
///   is indirect but sufficient: statistical.tsv pins
///   `=IFERROR(MIN({"a","b","c"}),"no numbers")` to the *number* 0, so MIN
///   cannot have errored. A direct `=MIN({"a","b"})` row is captured and
///   lands with the others.
///
/// MIN needs no code for the second rule — it already falls through to 0.
/// An array holding only *blanks* is a third case, unprobed, left at the 0
/// MIN has always given it.
pub fn min_fn(args: &[Value]) -> Value {
    if args.is_empty() {
        return Value::Error(ErrorKind::NA);
    }
    // Zone-aware participation: a column of Zoned instants returns the earliest
    // (as a Zoned); mixing naive and aware values is #VALUE!.
    if let Some(r) = super::stat_helpers::zoned_extreme(args, true) {
        return r;
    }
    let mut result: Option<f64> = None;
    for arg in args {
        match arg {
            Value::Number(n) => {
                result = Some(result.map_or(*n, |cur: f64| cur.min(*n)));
            }
            Value::Bool(b) => {
                let n = if *b { 1.0 } else { 0.0 };
                result = Some(result.map_or(n, |cur: f64| cur.min(n)));
            }
            Value::Text(s) => {
                let trimmed = s.trim();
                match trimmed.parse::<f64>() {
                    Ok(v) if v.is_finite() => {
                        result = Some(result.map_or(v, |cur: f64| cur.min(v)));
                    }
                    _ => return Value::Error(ErrorKind::Value),
                }
            }
            Value::Empty => {}
            Value::Array(elems) => {
                // An explicitly empty argument is fatal on the spot, even if a
                // number was already in hand — matching MAX, whose
                // `=MAX(SPARKLINE({1,2,3}),{})` row is #REF! despite the
                // sparkline that would otherwise answer 0.
                if elems.is_empty() {
                    return Value::Error(ErrorKind::Ref);
                }
                // Recurse into nested arrays (e.g. a vertical range
                // materializes as nested one-element row arrays) so every
                // cell is visited.
                if let Err(e) = min_array_into(elems, &mut result) {
                    return e;
                }
            }
            Value::Error(e) => return Value::Error(e.clone()),
            Value::ErrorMsg(e, m) => return Value::ErrorMsg(e.clone(), m.clone()),
            _ => {}
        }
    }
    Value::Number(result.unwrap_or(0.0))
}

/// Recursively fold a nested array's numbers into `result` for MIN's
/// array-context rules (Bool/Text/Empty skipped, errors propagate).
fn min_array_into(elems: &[Value], result: &mut Option<f64>) -> Result<(), Value> {
    for elem in elems {
        match elem {
            Value::Number(n) => {
                *result = Some(result.map_or(*n, |cur: f64| cur.min(*n)));
            }
            Value::Error(e) => return Err(Value::Error(e.clone())),
            Value::ErrorMsg(e, m) => return Err(Value::ErrorMsg(e.clone(), m.clone())),
            Value::Array(inner) => min_array_into(inner, result)?,
            _ => {}
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests;
