use crate::types::{ErrorKind, Value};

/// `MAX(value1, ...)` — largest numeric value in the arguments.
/// Direct args: Numbers, Bool (TRUE=1, FALSE=0), parseable text coerced to number.
/// Array elements: Numbers only; text/Bool → skip; errors propagate.
///
/// "No numbers" is *two* rules, not one:
///
/// - an **empty** array argument is `#REF!`: `=MAX({})` — the one rule with
///   an in-repo row (statistical.tsv);
/// - a **populated** array holding nothing numeric is 0: `=MAX({"a","b"})`
///   and `=MAX({TRUE,FALSE})` are both 0, even though neither text nor
///   booleans contribute a number in array context. Captured in Google
///   Sheets; the rows land separately, since they fail until this code exists.
///
/// An array of nothing but *blanks* — what `=MAX(A1:A3)` over an untouched
/// column materializes as — is 0 as well. That is captured across seven range
/// shapes, each with a populated control; the shapes, the controls and where
/// the rows live are set out on [`stat_helpers::is_blank_only_array`], which
/// is what decides the case. Deciding it there rather than through
/// `array_had_content` is deliberate: a blank sitting next to something else
/// numberless changes nothing.
///
/// This change moves the blank-only array and nothing else. `array_had_content`
/// is still set by text and booleans alone, so it exempts exactly what those
/// captures cover and makes no claim past them.
///
/// [`stat_helpers::is_blank_only_array`]: super::stat_helpers::is_blank_only_array
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
    let mut array_had_content = false;
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
                if let Err(e) = max_array_into(
                    elems,
                    &mut result,
                    &mut skipped_sparkline,
                    &mut array_had_content,
                ) {
                    return e;
                }
            }
            Value::Error(e) => return Value::Error(e.clone()),
            Value::ErrorMsg(e, m) => return Value::ErrorMsg(e.clone(), m.clone()),
            _ => {}
        }
    }
    // A skipped sparkline is not "nothing usable": the aggregate had something
    // in scope, so it answers 0 rather than falling into the rule below
    // (google.tsv: `=MAX(Data!K1:K1)` is 0). This runs first, so it decides
    // every sparkline case before `array_had_content` is consulted at all.
    if skipped_sparkline && result.is_none() {
        return Value::Number(0.0);
    }
    // An array holding text or booleans answers 0 (`=MAX({"a","b"})` and
    // `=MAX({TRUE,FALSE})` are both 0). `array_had_content` is set by exactly
    // those two variants and nothing else, so it exempts what that capture
    // covers and makes no claim beyond it.
    if had_array && !array_had_content && result.is_none() {
        // An array of nothing but blanks is a further exemption, and it is
        // captured: `=MAX(A1:A3)` over empty cells is 0, the same answer MIN,
        // MAXA and MINA give it — see `is_blank_only_array` for every range
        // shape that was probed and where the rows live. The check is on the
        // arguments as a whole, so a blank mixed with anything else is
        // untouched by this rule.
        if super::stat_helpers::is_blank_only_array(args) {
            return Value::Number(0.0);
        }
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
///
/// `had_content` is set by text and booleans *only* — the two variants the
/// capture covers (`=MAX({"a","b"})` and `=MAX({TRUE,FALSE})` are both 0).
/// It is deliberately not a catch-all: every other non-numeric variant, most
/// notably `Date`, leaves it alone and so keeps the `#REF!` MAX has always
/// answered. Listing the variants rather than falling through also stops a
/// future `Value` kind inheriting content-hood by accident. An all-blank array
/// no longer ends at `#REF!` either, but it gets there without this flag — see
/// the blank-only check in `max_fn`.
fn max_array_into(
    elems: &[Value],
    result: &mut Option<f64>,
    skipped_sparkline: &mut bool,
    had_content: &mut bool,
) -> Result<(), Value> {
    for elem in elems {
        match elem {
            Value::Number(n) => {
                *result = Some(result.map_or(*n, |cur: f64| cur.max(*n)));
            }
            Value::Text(_) | Value::Bool(_) => *had_content = true,
            Value::Sparkline(_) => *skipped_sparkline = true,
            Value::Error(e) => return Err(Value::Error(e.clone())),
            Value::ErrorMsg(e, m) => return Err(Value::ErrorMsg(e.clone(), m.clone())),
            Value::Array(inner) => max_array_into(inner, result, skipped_sparkline, had_content)?,
            _ => {}
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests;
