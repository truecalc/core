use crate::eval::coercion::to_number;
use crate::eval::functions::check_arity;
use crate::types::Value;

/// `DAYS(end_date, start_date)` — number of days between two serial dates.
/// Returns end_serial - start_serial as an integer.
pub fn days_fn(args: &[Value]) -> Value {
    if let Some(e) = check_arity(args, 2, 2) {
        return e;
    }
    let end   = match to_number(args[0].clone()) { Ok(n) => n, Err(e) => return e };
    let start = match to_number(args[1].clone()) { Ok(n) => n, Err(e) => return e };
    Value::Number(end.floor() - start.floor())
}

#[cfg(test)]
mod tests;
