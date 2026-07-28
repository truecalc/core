use crate::types::{ErrorKind, Value};

/// `MAX(value1, ...)` — largest numeric value in the arguments.
/// Direct args: Numbers, Bool (TRUE=1, FALSE=0), parseable text coerced to number.
/// Array elements: Numbers only; text/Bool → skip; errors propagate.
/// Empty array arg → #REF!. No numbers → 0.0.
pub fn max_fn(args: &[Value]) -> Value {
    if args.is_empty() {
        return Value::Error(ErrorKind::NA);
    }
    // Zone-aware participation: a column of Zoned instants returns the latest
    // (as a Zoned); mixing naive and aware values is #VALUE!.
    if let Some(r) = super::stat_helpers::zoned_extreme(args, false) {
        return r;
    }
    let mut result: Option<f64> = None;
    let mut had_array = false;
    let mut skipped_sparkline = false;
    for arg in args {
        match arg {
            Value::Sparkline(_) => skipped_sparkline = true,
            Value::Number(n) => {
                result = Some(result.map_or(*n, |cur: f64| cur.max(*n)));
            }
            Value::Bool(b) => {
                let n = if *b { 1.0 } else { 0.0 };
                result = Some(result.map_or(n, |cur: f64| cur.max(n)));
            }
            Value::Text(s) => {
                let trimmed = s.trim();
                match trimmed.parse::<f64>() {
                    Ok(v) if v.is_finite() => {
                        result = Some(result.map_or(v, |cur: f64| cur.max(v)));
                    }
                    _ => return Value::Error(ErrorKind::Value),
                }
            }
            Value::Empty => {}
            Value::Array(elems) => {
                had_array = true;
                if elems.is_empty() {
                    return Value::Error(ErrorKind::Ref);
                }
                // Recurse into nested arrays (e.g. a vertical range
                // materializes as nested one-element row arrays) so every
                // cell is visited.
                if let Err(e) = max_array_into(elems, &mut result, &mut skipped_sparkline) {
                    return e;
                }
            }
            Value::Error(e) => return Value::Error(e.clone()),
            Value::ErrorMsg(e, m) => return Value::ErrorMsg(e.clone(), m.clone()),
            _ => {}
        }
    }
    // A skipped sparkline is not "nothing usable": the aggregate had something
    // in scope, so it answers 0 rather than falling into the numberless-array
    // rule below (google.tsv: `=MAX(Data!K1:K1)` is 0). Scoped to a sparkline
    // so `=MAX({"a"})` and friends keep their pre-existing `#REF!`.
    if skipped_sparkline && result.is_none() {
        return Value::Number(0.0);
    }
    // Empty array with no numbers → Ref
    if had_array && result.is_none() {
        return Value::Error(ErrorKind::Ref);
    }
    Value::Number(result.unwrap_or(0.0))
}

/// Recursively fold a nested array's numbers into `result` for MAX's
/// array-context rules (Bool/Text/Empty skipped, errors propagate).
/// A sparkline is skipped wherever it appears, and an aggregate whose scope
/// holds nothing else answers 0 — the same answer whether it arrived as a
/// direct argument or through a range (google.tsv: `=MAX(SPARKLINE({1,2,3}))`
/// and `=MAX(Data!K1:K1)` are both 0). The flag is what distinguishes "skipped a
/// sparkline" from "saw nothing usable at all", which stay different answers.
fn max_array_into(
    elems: &[Value],
    result: &mut Option<f64>,
    skipped_sparkline: &mut bool,
) -> Result<(), Value> {
    for elem in elems {
        match elem {
            Value::Number(n) => {
                *result = Some(result.map_or(*n, |cur: f64| cur.max(*n)));
            }
            Value::Sparkline(_) => *skipped_sparkline = true,
            Value::Error(e) => return Err(Value::Error(e.clone())),
            Value::ErrorMsg(e, m) => return Err(Value::ErrorMsg(e.clone(), m.clone())),
            Value::Array(inner) => max_array_into(inner, result, skipped_sparkline)?,
            _ => {}
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests;
