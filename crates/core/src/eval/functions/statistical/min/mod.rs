use crate::types::{ErrorKind, Value};

/// `MIN(value1, ...)` — smallest numeric value in the arguments.
/// Direct args: Numbers, Dates, Bool (TRUE=1, FALSE=0), parseable text coerced
/// to number.
/// Array elements: Numbers and Dates; text/Bool → skip; errors propagate.
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
/// An array holding only *blanks* — what `=MIN(A1:A3)` over an untouched
/// column materializes as — is 0 too, and is now captured rather than assumed:
/// seven range shapes, each with a populated control, are laid out on
/// [`stat_helpers::is_blank_only_array`] along with the note that those rows
/// are not in this repo yet. MIN was the one of the four already giving that
/// answer, so it needs no code for this rule either — the predicate is not
/// called from here at all; MAX, MAXA and MINA were brought to it.
///
/// [`stat_helpers::is_blank_only_array`]: super::stat_helpers::is_blank_only_array
///
/// **Dates participate as bare serials** and carry their type out. A date-only
/// range answers the earliest date, a date beside a plain number is compared on
/// the serial with no special casing (so a small plain number beats every
/// date), and the result is date-typed whenever a date took part — even when
/// the plain number won.
///
/// Captured and extrapolated are not the same thing here, so keep them apart:
///
/// - **Values** are captured in Google Sheets for every form — a date-only
///   column, a date/number column, array literals of both shapes, and dates
///   passed as direct arguments.
/// - **Typing** is captured for the *range* forms only, read back through the
///   cell that holds the result (`=MIN(<date-only range>)` and
///   `=MIN(<date/number range>)` both come back `date` — the second even
///   though a plain 5 won). The literal and direct-argument rows report
///   `number`, but that is an artifact of the capture harness reading them
///   through an `INDEX(...,1,1)` wrapper, which drops the cell's date format —
///   it is not a Sheets answer. So the date typing of
///   `=MIN({DATE(...),DATE(...)})` is **extrapolated** from the range forms,
///   not probed.
///
/// **None of those rows are in this repo yet.** They come off the
/// conformance-fixtures pipeline and land in a separate fixtures-only PR —
/// they fail until this code exists, and CI rejects a PR that mixes fixture
/// TSVs with code. Same arrangement as the blank-only rows described on
/// `stat_helpers::is_blank_only_array`. A reviewer working from this repo
/// alone can check the unit tests and this comment; the Sheets answers
/// themselves have to be taken from that pipeline.
///
/// Two consequences are filed rather than fixed here:
///
/// - `COUNT` does not count dates, so `=COUNT(MIN(<date range>))` answers 0
///   where it used to answer 1 — a pre-existing `COUNT` gap this change makes
///   reachable. See #780.
/// - `MAXA`/`MINA` silently drop a `Zoned` sitting beside a `Date`, where
///   `MAX`/`MIN` route the same input through `zoned_extreme` and error.
///   Unprobed on both sides. See #781.
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
    let mut saw_date = false;
    for arg in args {
        match arg {
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
                if let Err(e) = min_array_into(elems, &mut result, &mut saw_date) {
                    return e;
                }
            }
            Value::Error(e) => return Value::Error(e.clone()),
            Value::ErrorMsg(e, m) => return Value::ErrorMsg(e.clone(), m.clone()),
            // Listed rather than a catch-all so a new `Value` variant is a
            // compile error here instead of a silent skip. A `Zoned` only
            // reaches this loop when no other argument was zone-aware, which
            // `zoned_extreme` above has already ruled on.
            Value::Zoned(_) | Value::Sparkline(_) => {}
        }
    }
    match result {
        // A date anywhere in scope makes the answer date-typed, whether or not
        // the date is the value that won.
        Some(n) if saw_date => Value::Date(n),
        Some(n) => Value::Number(n),
        None => Value::Number(0.0),
    }
}

/// Recursively fold a nested array's numbers into `result` for MIN's
/// array-context rules (Bool/Text/Empty skipped, errors propagate).
/// A `Date` folds in as its bare serial and raises `saw_date`, which types the
/// answer; every other variant keeps the arm it already had.
fn min_array_into(
    elems: &[Value],
    result: &mut Option<f64>,
    saw_date: &mut bool,
) -> Result<(), Value> {
    for elem in elems {
        match elem {
            Value::Number(n) => {
                *result = Some(result.map_or(*n, |cur: f64| cur.min(*n)));
            }
            Value::Date(n) => {
                *saw_date = true;
                *result = Some(result.map_or(*n, |cur: f64| cur.min(*n)));
            }
            Value::Error(e) => return Err(Value::Error(e.clone())),
            Value::ErrorMsg(e, m) => return Err(Value::ErrorMsg(e.clone(), m.clone())),
            Value::Array(inner) => min_array_into(inner, result, saw_date)?,
            // Listed rather than a catch-all so a new `Value` variant is a
            // compile error here instead of inheriting "skipped" by accident.
            Value::Text(_)
            | Value::Bool(_)
            | Value::Empty
            | Value::Zoned(_)
            | Value::Sparkline(_) => {}
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests;
