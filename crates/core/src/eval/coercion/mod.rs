use crate::display::display_number;
use crate::eval::functions::date::serial::text_to_date_serial;
use crate::types::{ErrorKind, Value};

/// Coerce a [`Value`] to `f64` for arithmetic operations.
///
/// - `Number` → its value
/// - `Bool` → `1.0` (true) or `0.0` (false)
/// - `Empty` → `0.0`
/// - `Text` → parsed as f64, or `Value::Error(ErrorKind::Value)` on failure
/// - `Error` → propagated as `Err`
/// - `Array` → `Value::Error(ErrorKind::Value)`
pub fn to_number(v: Value) -> Result<f64, Value> {
    match v {
        Value::Number(n) | Value::Date(n) => Ok(n),
        Value::Bool(b)   => Ok(if b { 1.0 } else { 0.0 }),
        Value::Empty     => Ok(0.0),
        Value::Text(s)   => {
            if s.is_empty() { Ok(0.0) }
            else {
                s.parse::<f64>()
                    .or_else(|_| text_to_date_serial(&s).ok_or(Value::Error(ErrorKind::Value)))
                    .map_err(|_| Value::Error(ErrorKind::Value))
            }
        }
        Value::Error(_)  => Err(v),
        Value::Array(_)  => Err(Value::Error(ErrorKind::Value)),
        // Zoned instants have no naive numeric value; force an explicit TZSERIAL
        // downcast rather than silently mixing naive/aware time.
        Value::Zoned(_)  => Err(Value::Error(ErrorKind::Value)),
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
        Value::Error(_) => Err(v),
        Value::Array(_) => Err(Value::Error(ErrorKind::Value)),
        // Self-describing canonical RFC-9557 form so concatenation is lossless.
        Value::Zoned(z) => Ok(z.to_rfc9557()),
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
        Value::Error(_)  => Err(v),
        Value::Text(ref s) => match s.to_uppercase().as_str() {
            "TRUE"  => Ok(true),
            "FALSE" => Ok(false),
            _       => Err(Value::Error(ErrorKind::Value)),
        },
        Value::Empty => Err(Value::Error(ErrorKind::Value)),
        // A zoned instant has no truthiness.
        Value::Zoned(_) => Err(Value::Error(ErrorKind::Value)),
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
