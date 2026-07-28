use crate::types::{ErrorKind, Value};

/// `MAXA(value1, ...)` — largest value, coercing booleans (TRUE=1, FALSE=0).
/// - Numbers included directly.
/// - Booleans coerced: TRUE=1, FALSE=0.
/// - Text in direct args → `#VALUE!`.
/// - Empty → skip.
/// - No args → `#N/A`.
pub fn maxa_fn(args: &[Value]) -> Value {
    if args.is_empty() {
        return Value::Error(ErrorKind::NA);
    }
    let mut result: Option<f64> = None;
    // See `fold_array_max` for why this flag exists.
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
            Value::Text(_) => return Value::Error(ErrorKind::Value),
            Value::Empty => {}
            Value::Array(inner) => {
                // In array context: Numbers included, Bool→1/0, Text→0, Empty→skip.
                // Recurses into nested arrays (e.g. a vertical range
                // materializes as nested one-element row arrays).
                if let Err(e) = fold_array_max(inner, &mut result, &mut skipped_sparkline) {
                    return e;
                }
            }
            Value::Error(e) => return Value::Error(e.clone()),
            Value::ErrorMsg(e, m) => return Value::ErrorMsg(e.clone(), m.clone()),
            _ => {}
        }
    }
    match result {
        Some(n) => Value::Number(n),
        None if skipped_sparkline => Value::Number(0.0),
        None    => Value::Error(ErrorKind::NA),
    }
}

/// Recurse into nested arrays (e.g. a vertical range materializes as nested
/// one-element row arrays) so every cell is visited, folding into `result`
/// with MAXA's array-context coercion rules.
/// A sparkline is skipped wherever it appears, and an aggregate whose scope
/// holds nothing else answers 0 — the same answer whether it arrived as a
/// direct argument or through a range (google.tsv: `=MAXA(SPARKLINE({1,2,3}))`
/// and `=MAXA(Data!K1:K1)` are both 0). The flag is what distinguishes "skipped a
/// sparkline" from "saw nothing usable at all", which stay different answers.
fn fold_array_max(
    arr: &[Value],
    result: &mut Option<f64>,
    skipped_sparkline: &mut bool,
) -> Result<(), Value> {
    for v in arr {
        let n = match v {
            Value::Sparkline(_) => {
                *skipped_sparkline = true;
                continue;
            }
            Value::Number(n) => *n,
            Value::Bool(b) => if *b { 1.0 } else { 0.0 },
            Value::Text(_) => 0.0,
            Value::Empty => continue,
            Value::Array(inner) => {
                fold_array_max(inner, result, skipped_sparkline)?;
                continue;
            }
            Value::Error(e) => return Err(Value::Error(e.clone())),
            Value::ErrorMsg(e, m) => return Err(Value::ErrorMsg(e.clone(), m.clone())),
            _ => continue,
        };
        *result = Some(result.map_or(n, |cur: f64| cur.max(n)));
    }
    Ok(())
}

#[cfg(test)]
mod tests;
