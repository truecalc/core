use crate::eval::functions::check_arity;
use crate::types::Value;

/// `ISURL(value)` — returns TRUE if the value is a string that looks like a URL.
///
/// Returns TRUE for strings with a recognized scheme (e.g. `https://`) or for
/// domain-like strings that contain a dot and no spaces.
/// Non-string values return FALSE.
pub fn isurl_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 1, 1) {
        return err;
    }
    let result = match &args[0] {
        Value::Text(s) => looks_like_url(s),
        _ => false,
    };
    Value::Bool(result)
}

fn looks_like_url(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }
    // Has a valid scheme like http://, https://, ftp://
    if let Some(pos) = s.find("://") {
        if pos > 0 {
            let scheme = &s[..pos];
            let valid_scheme = scheme.chars().enumerate().all(|(i, c)| {
                if i == 0 { c.is_ascii_alphabetic() }
                else { c.is_ascii_alphanumeric() || c == '+' || c == '-' || c == '.' }
            });
            if valid_scheme && s.len() > pos + 3 {
                return true;
            }
        }
    }
    // Also accept domain-like strings: contain a dot and no spaces
    !s.contains(' ') && s.contains('.')
}

#[cfg(test)]
mod tests;
