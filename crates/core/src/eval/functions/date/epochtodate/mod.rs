use crate::eval::coercion::to_number;
use crate::eval::functions::check_arity;
use crate::types::{ErrorKind, Value};

/// `EPOCHTODATE(timestamp, [unit])` — convert a Unix timestamp to a spreadsheet serial.
///
/// unit: 1 (default) = seconds, 2 = milliseconds, 3 = microseconds
/// Unix epoch (1970-01-01) = serial 25569.
pub fn epochtodate_fn(args: &[Value]) -> Value {
    if let Some(e) = check_arity(args, 1, 2) {
        return e;
    }
    let timestamp = match to_number(args[0].clone()) { Ok(n) => n, Err(e) => return e };
    let unit = if args.len() >= 2 {
        match to_number(args[1].clone()) { Ok(n) => n as u32, Err(e) => return e }
    } else {
        1
    };

    let seconds = match unit {
        1 => timestamp,
        2 => timestamp / 1_000.0,
        3 => timestamp / 1_000_000.0,
        _ => return Value::Error(ErrorKind::Num),
    };

    // Unix epoch = serial 25569; 86400 seconds per day.
    let serial = 25569.0 + seconds / 86400.0;
    Value::Number(serial)
}

#[cfg(test)]
mod tests;
