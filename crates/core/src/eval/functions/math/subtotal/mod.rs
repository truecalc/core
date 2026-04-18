use crate::eval::functions::check_arity;
use crate::types::{ErrorKind, Value};

/// `SUBTOTAL(function_code, ref1, ...)` — applies a function to a list.
///
/// When any argument (other than the function code) is an array, returns #VALUE!.
/// This mirrors the Google Sheets behavior for in-formula array ranges.
pub fn subtotal_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 1, 255) {
        return err;
    }
    // If any non-code argument is an array, return #VALUE!
    for arg in args.iter().skip(1) {
        if matches!(arg, Value::Array(_)) {
            return Value::Error(ErrorKind::Value);
        }
    }
    // Stub: basic implementation returns #VALUE! for now
    Value::Error(ErrorKind::Value)
}
