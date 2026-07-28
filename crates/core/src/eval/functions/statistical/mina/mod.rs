use crate::types::{ErrorKind, Value};

/// `MINA(value1, ...)` — smallest value, coercing booleans (TRUE=1, FALSE=0).
/// - Numbers included directly.
/// - Booleans coerced: TRUE=1, FALSE=0.
/// - Text in direct args → `#VALUE!`.
/// - Empty → skip.
/// - Empty array argument → `#REF!`.
/// - Array of nothing but blanks → 0.
/// - No args → `#N/A`.
///
/// **Dates participate as bare serials** and carry their type out, exactly as
/// they do for MIN: a date-only range answers the earliest date, a date beside
/// a plain number is compared on the serial (so a small plain number beats
/// every date), and the result is date-typed whenever a date took part — even
/// when the plain number won. Captured in Google Sheets alongside the MAX/MIN
/// forms, which agree on every date input; the rows land separately.
pub fn mina_fn(args: &[Value]) -> Value {
    if args.is_empty() {
        return Value::Error(ErrorKind::NA);
    }
    let mut result: Option<f64> = None;
    // See `fold_array_min` for why this flag exists.
    let mut skipped_sparkline = false;
    let mut saw_date = false;
    for arg in args {
        match arg {
            Value::Sparkline(_) => skipped_sparkline = true,
            Value::Number(n) => {
                result = Some(result.map_or(*n, |cur: f64| cur.min(*n)));
            }
            Value::Date(n) => {
                saw_date = true;
                result = Some(result.map_or(*n, |cur: f64| cur.min(*n)));
            }
            Value::Bool(b) => {
                let n = if *b { 1.0 } else { 0.0 };
                result = Some(result.map_or(n, |cur: f64| cur.min(n)));
            }
            Value::Text(_) => return Value::Error(ErrorKind::Value),
            Value::Empty => {}
            Value::Array(inner) => {
                // An empty argument is #REF!, as it is for MIN and MAX
                // (`=MINA({})`). Note MINA reaches the *other* answers by a
                // different route: text folds in as 0 rather than being
                // skipped, so `=MINA({"a","b"})` is already 0 without any
                // "populated but numberless" rule.
                if inner.is_empty() {
                    return Value::Error(ErrorKind::Ref);
                }
                // In array context: Numbers included, Bool→1/0, Text→0, Empty→skip.
                // Recurses into nested arrays (e.g. a vertical range
                // materializes as nested one-element row arrays).
                if let Err(e) =
                    fold_array_min(inner, &mut result, &mut skipped_sparkline, &mut saw_date)
                {
                    return e;
                }
            }
            Value::Error(e) => return Value::Error(e.clone()),
            Value::ErrorMsg(e, m) => return Value::ErrorMsg(e.clone(), m.clone()),
            // Listed rather than a catch-all so a new `Value` variant is a
            // compile error here instead of a silent skip.
            Value::Zoned(_) => {}
        }
    }
    match result {
        // A date anywhere in scope makes the answer date-typed, whether or not
        // the date is the value that won.
        Some(n) if saw_date => Value::Date(n),
        Some(n) => Value::Number(n),
        None if skipped_sparkline => Value::Number(0.0),
        // An array of nothing but blanks is 0, not #N/A: `=MINA(A1:A3)` over
        // empty cells answers the same 0 that MIN, MAX and MAXA give it — see
        // `is_blank_only_array` for every range shape that was probed, the
        // controls that prove the ranges resolved, and where the rows live. A
        // blank argument with no array in sight keeps the #N/A below — that
        // shape is unprobed.
        None if super::stat_helpers::is_blank_only_array(args) => Value::Number(0.0),
        None    => Value::Error(ErrorKind::NA),
    }
}

/// Recurse into nested arrays (e.g. a vertical range materializes as nested
/// one-element row arrays) so every cell is visited, folding into `result`
/// with MINA's array-context coercion rules.
/// A sparkline is skipped wherever it appears, and an aggregate whose scope
/// holds nothing else answers 0 — the same answer whether it arrived as a
/// direct argument or through a range (google.tsv: `=MINA(SPARKLINE({1,2,3}))`
/// and `=MINA(Data!K1:K1)` are both 0). The flag is what distinguishes "skipped a
/// sparkline" from "saw nothing usable at all", which stay different answers.
fn fold_array_min(
    arr: &[Value],
    result: &mut Option<f64>,
    skipped_sparkline: &mut bool,
    saw_date: &mut bool,
) -> Result<(), Value> {
    for v in arr {
        let n = match v {
            Value::Sparkline(_) => {
                *skipped_sparkline = true;
                continue;
            }
            Value::Number(n) => *n,
            // A date folds in as its bare serial and types the answer.
            Value::Date(n) => {
                *saw_date = true;
                *n
            }
            Value::Bool(b) => if *b { 1.0 } else { 0.0 },
            Value::Text(_) => 0.0,
            Value::Empty => continue,
            Value::Array(inner) => {
                fold_array_min(inner, result, skipped_sparkline, saw_date)?;
                continue;
            }
            Value::Error(e) => return Err(Value::Error(e.clone())),
            Value::ErrorMsg(e, m) => return Err(Value::ErrorMsg(e.clone(), m.clone())),
            // Listed rather than a catch-all so a new `Value` variant is a
            // compile error here instead of inheriting "skipped" by accident.
            Value::Zoned(_) => continue,
        };
        *result = Some(result.map_or(n, |cur: f64| cur.min(n)));
    }
    Ok(())
}

#[cfg(test)]
mod tests;
