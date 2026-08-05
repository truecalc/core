//! Holiday argument extraction for NETWORKDAYS / WORKDAY family.
//!
//! Google Sheets rules:
//! - Omitted: no holidays (empty set).
//! - Empty array `{}`: returns `#REF!`.
//! - Scalar text string (not coercible to a date serial): returns `#VALUE!`.
//! - Single number / Date: one holiday.
//! - Array of numbers / Dates: those days are holidays.

use std::collections::HashSet;
use crate::eval::coercion::to_number;
use crate::types::{ErrorKind, Value};

/// Extract a set of integer day-serials from the holidays argument.
///
/// Returns `Ok(set)` on success (including `Ok(empty)` when the arg is absent),
/// or `Err(Value::Error(...))` on bad input.
pub fn extract_holidays(holiday_arg: Option<&Value>) -> Result<HashSet<i64>, Value> {
    let arg = match holiday_arg {
        None => return Ok(HashSet::new()),
        Some(v) => v,
    };

    match arg {
        // Empty array -> #REF!
        Value::Array(items) if items.is_empty() => {
            Err(Value::Error(ErrorKind::Ref))
        }
        // Non-empty array -> collect numeric serials, ignore text/empty/error elements.
        Value::Array(items) => {
            let mut set = HashSet::new();
            for item in items {
                match item {
                    Value::Array(inner) => {
                        for v in inner {
                            if let Ok(n) = to_number(v.clone()) {
                                set.insert(n.floor() as i64);
                            }
                        }
                    }
                    Value::Text(_) => {
                        // Text inside an array: skip silently (GS ignores non-numeric holiday cells)
                    }
                    Value::Empty => {}
                    _ => {
                        if let Ok(n) = to_number(item.clone()) {
                            set.insert(n.floor() as i64);
                        }
                    }
                }
            }
            Ok(set)
        }
        // Text scalar -> #VALUE! (can't be a date serial)
        Value::Text(_) => Err(Value::Error(ErrorKind::Value)),
        // Single numeric value (including Value::Date) -> one holiday
        _ => match to_number(arg.clone()) {
            Ok(n) => {
                let mut set = HashSet::new();
                set.insert(n.floor() as i64);
                Ok(set)
            }
            Err(_) => Err(Value::Error(ErrorKind::Value)),
        },
    }
}
