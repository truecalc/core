use crate::types::{ErrorKind, Value};

/// `MIN(value1, ...)` — smallest numeric value in the arguments.
/// Direct args: Numbers, Bool (TRUE=1, FALSE=0), parseable text coerced to number.
/// Array elements: Numbers only; text/Bool → skip; errors propagate.
///
/// "No numbers" is *two* rules, not one — the fixtures separate them:
///
/// - an **absent** argument is `#REF!`: `=MIN({})` and `=MIN(<blank range>)`;
/// - a **populated** argument holding nothing numeric is 0:
///   `=MIN({"a","b"})`, and `=IFERROR(MIN({"a","b","c"}),"no numbers")` is
///   pinned to the number 0.
///
/// So blanks read as absent while text reads as present-but-unusable, which
/// is why `array_had_content` ignores `Empty` but counts everything else.
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
    let mut had_array = false;
    let mut array_had_content = false;
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
                had_array = true;
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
                if let Err(e) = min_array_into(elems, &mut result, &mut array_had_content) {
                    return e;
                }
            }
            Value::Error(e) => return Value::Error(e.clone()),
            Value::ErrorMsg(e, m) => return Value::ErrorMsg(e.clone(), m.clone()),
            _ => {}
        }
    }
    // Arrays that held only blanks are as absent as `{}` is: `=MIN(<range of
    // blank cells>)` is #REF!, while `=MIN({"a","b"})` is 0.
    if had_array && !array_had_content && result.is_none() {
        return Value::Error(ErrorKind::Ref);
    }
    Value::Number(result.unwrap_or(0.0))
}

/// Recursively fold a nested array's numbers into `result` for MIN's
/// array-context rules (Bool/Text/Empty skipped, errors propagate).
///
/// `had_content` records whether the array held *anything* other than a
/// blank. Text and booleans do not contribute a number here, but they do make
/// the array present rather than absent, which is what separates
/// `=MIN({"a","b"})` (0) from `=MIN(<blank range>)` (#REF!).
fn min_array_into(
    elems: &[Value],
    result: &mut Option<f64>,
    had_content: &mut bool,
) -> Result<(), Value> {
    for elem in elems {
        match elem {
            Value::Number(n) => {
                *had_content = true;
                *result = Some(result.map_or(*n, |cur: f64| cur.min(*n)));
            }
            Value::Error(e) => return Err(Value::Error(e.clone())),
            Value::ErrorMsg(e, m) => return Err(Value::ErrorMsg(e.clone(), m.clone())),
            Value::Array(inner) => min_array_into(inner, result, had_content)?,
            Value::Empty => {}
            _ => *had_content = true,
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests;
