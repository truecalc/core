use crate::display::display_number;
use crate::eval::functions::date::serial::text_to_date_serial;
use crate::types::{ErrorKind, Value};

/// Shared number-parsing rules for text, used by both `to_number` (implicit
/// arithmetic coercion) and `VALUE()` so the two agree on every input.
/// Accepts a direct numeric literal, comma-formatted numbers
/// (`"1,234.56"`), a `$` currency prefix, a `%` percent suffix, and
/// surrounding whitespace. Does not attempt date/time parsing — callers
/// that want that fall back to `text_to_date_serial`/`text_to_time_serial`
/// themselves.
pub(crate) fn parse_number_text(text: &str) -> Option<f64> {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return Some(0.0);
    }
    if let Ok(n) = trimmed.parse::<f64>() {
        return Some(n);
    }
    // Percentage: "12%" → 0.12
    if let Some(pct) = trimmed.strip_suffix('%') {
        if let Ok(n) = pct.trim().replace(',', "").parse::<f64>() {
            return Some(n / 100.0);
        }
    }
    // Currency prefix: "$42" → 42
    if let Some(rest) = trimmed.strip_prefix('$') {
        if let Ok(n) = rest.trim().replace(',', "").parse::<f64>() {
            return Some(n);
        }
    }
    // Comma-formatted numbers: "1,234.56" → 1234.56
    let no_commas = trimmed.replace(',', "");
    no_commas.parse::<f64>().ok()
}

/// Coerce a [`Value`] to `f64` for arithmetic operations.
///
/// - `Number` → its value
/// - `Bool` → `1.0` (true) or `0.0` (false)
/// - `Empty` → `0.0`
/// - `Text` → parsed via [`parse_number_text`] (same rules as `VALUE()`),
///   falling back to date-serial parsing, or `Value::Error(ErrorKind::Value)`
///   on failure
/// - `Error` → propagated as `Err`
/// - `Array` → `Value::Error(ErrorKind::Value)`
pub fn to_number(v: Value) -> Result<f64, Value> {
    match v {
        Value::Number(n) | Value::Date(n) => Ok(n),
        Value::Bool(b)   => Ok(if b { 1.0 } else { 0.0 }),
        Value::Empty     => Ok(0.0),
        Value::Text(s)   => parse_number_text(&s)
            .or_else(|| text_to_date_serial(&s))
            .ok_or(Value::Error(ErrorKind::Value)),
        Value::Error(_) | Value::ErrorMsg(_, _) => Err(v),
        Value::Array(_)  => Err(Value::Error(ErrorKind::Value)),
        // Zoned instants have no naive numeric value; force an explicit TZSERIAL
        // downcast rather than silently mixing naive/aware time.
        Value::Zoned(_)  => Err(Value::Error(ErrorKind::Value)),
        // Arithmetic rejects a sparkline: `=SPARKLINE({1,2,3})+1` is `#VALUE!`
        // (google.tsv). `N()` (0) and the aggregates (which skip it) do not come
        // through here.
        Value::Sparkline(_) => Err(Value::Error(ErrorKind::Value)),
    }
}

/// Coerce a [`Value`] to `String` for concatenation.
///
/// - `Text` → its string
/// - `Number` → formatted via [`display_number`]
/// - `Bool` → `"TRUE"` or `"FALSE"`
/// - `Empty` → `""`
/// - `Error` → propagated as `Err`
/// - `Array` → `Value::Error(ErrorKind::Value)`
pub fn to_string_val(v: Value) -> Result<String, Value> {
    match v {
        Value::Text(s)  => Ok(s),
        Value::Number(n) | Value::Date(n) => Ok(display_number(n)),
        Value::Bool(b)  => Ok(if b { "TRUE".to_string() } else { "FALSE".to_string() }),
        Value::Empty    => Ok(String::new()),
        Value::Error(_) | Value::ErrorMsg(_, _) => Err(v),
        Value::Array(_) => Err(Value::Error(ErrorKind::Value)),
        // Self-describing canonical RFC-9557 form so concatenation is lossless.
        Value::Zoned(z) => Ok(z.to_rfc9557()),
        // Text contexts are permissive: a sparkline reads as empty text
        // (google.tsv: `LEN` is `0`, `LEFT` and `TEXT` and `TEXTJOIN` are `""`,
        // `CONCATENATE(sparkline,"x")` is `"x"`, `EXACT(sparkline,"")` is
        // `TRUE`). The `&` *operator* is the one carved-out exception and
        // rejects it before reaching here — see `eval_binary`.
        Value::Sparkline(_) => Ok(String::new()),
    }
}

/// Coerce a [`Value`] to `bool` for conditional evaluation.
///
/// - `Bool` → its value
/// - `Number` → `false` if zero, `true` otherwise
/// - `Text("TRUE"/"FALSE")` → true/false (case-insensitive, Excel/GS compatible)
/// - `Text` (other) → `Value::Error(ErrorKind::Value)`
/// - `Error` → propagated as `Err`
/// - `Empty` → `Value::Error(ErrorKind::Value)`
/// - `Array` → collapse to top-left (anchor-cell view) and recurse
pub fn to_bool(v: Value) -> Result<bool, Value> {
    match v {
        Value::Bool(b)   => Ok(b),
        Value::Number(n) | Value::Date(n) => Ok(n != 0.0),
        Value::Error(_) | Value::ErrorMsg(_, _) => Err(v),
        Value::Text(ref s) => match s.to_uppercase().as_str() {
            "TRUE"  => Ok(true),
            "FALSE" => Ok(false),
            _       => Err(Value::Error(ErrorKind::Value)),
        },
        Value::Empty => Err(Value::Error(ErrorKind::Value)),
        // A zoned instant has no truthiness.
        Value::Zoned(_) => Err(Value::Error(ErrorKind::Value)),
        // A sparkline is falsy (google.tsv: `=IF(SPARKLINE({1,2,3}),1,2)` is 2).
        Value::Sparkline(_) => Ok(false),
        // Array condition: use the top-left (anchor) element — same as the
        // unspilled-array collapse the workbook layer applies.
        Value::Array(mut elems) => {
            if elems.is_empty() {
                return Err(Value::Error(ErrorKind::Value));
            }
            let mut top = elems.swap_remove(0);
            loop {
                match top {
                    Value::Array(mut inner) => {
                        if inner.is_empty() {
                            return Err(Value::Error(ErrorKind::Value));
                        }
                        top = inner.swap_remove(0);
                    }
                    other => return to_bool(other),
                }
            }
        }
    }
}

#[cfg(test)]
mod tests;
