use crate::eval::coercion::to_number;
use crate::eval::functions::check_arity;
use crate::types::{ErrorKind, Value};

const ROMAN_TABLE: &[(i32, &str)] = &[
    (1000, "M"),
    (900,  "CM"),
    (500,  "D"),
    (400,  "CD"),
    (100,  "C"),
    (90,   "XC"),
    (50,   "L"),
    (40,   "XL"),
    (10,   "X"),
    (9,    "IX"),
    (5,    "V"),
    (4,    "IV"),
    (1,    "I"),
];

/// `ROMAN(number)` — converts an integer (1–3999) to a Roman numeral string.
/// Returns `#VALUE!` for 0, negative, or out-of-range values.
pub fn roman_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 1, 1) {
        return err;
    }
    let n = match to_number(args[0].clone()) {
        Ok(v) => v,
        Err(e) => return e,
    };
    let n = n as i32;
    if n <= 0 || n > 3999 {
        return Value::Error(ErrorKind::Value);
    }
    Value::Text(to_roman(n))
}

fn to_roman(mut n: i32) -> String {
    let mut result = String::new();
    for &(val, sym) in ROMAN_TABLE {
        while n >= val {
            result.push_str(sym);
            n -= val;
        }
    }
    result
}

#[cfg(test)]
mod tests;
