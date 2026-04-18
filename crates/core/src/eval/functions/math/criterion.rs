use crate::types::Value;

/// A parsed criterion used by COUNTIF, SUMIF, AVERAGEIF.
#[derive(Debug)]
pub enum Criterion {
    NumEq(f64),
    NumNe(f64),
    NumLt(f64),
    NumGt(f64),
    NumLe(f64),
    NumGe(f64),
    /// Case-insensitive exact text match.
    TextEq(String),
    /// Case-insensitive not-equal text match.
    TextNe(String),
    /// Case-insensitive wildcard pattern (`*` = any chars, `?` = any single char).
    WildcardEq(Vec<char>),
    BoolEq(bool),
}

/// Flatten a `Value` into a flat list of references.
/// `Value::Array` is expanded one level; scalars become a single-element slice.
pub fn flatten_to_vec(v: &Value) -> Vec<&Value> {
    match v {
        Value::Array(arr) => arr.iter().collect(),
        other => vec![other],
    }
}

/// Parse a `Value` criterion argument into a [`Criterion`].
pub fn parse_criterion(v: &Value) -> Criterion {
    match v {
        Value::Number(n) => Criterion::NumEq(*n),
        Value::Bool(b) => Criterion::BoolEq(*b),
        Value::Text(s) => parse_criterion_str(s),
        _ => Criterion::TextEq(String::new()), // fallback — matches nothing useful
    }
}

/// Parse a string like `">2"`, `"apple"`, `"a*"`, `"<>3"` into a [`Criterion`].
fn parse_criterion_str(s: &str) -> Criterion {
    // Strip operator prefix — longest match first.
    let (op, rest) = if let Some(r) = s.strip_prefix("<>") {
        ("<>", r)
    } else if let Some(r) = s.strip_prefix(">=") {
        (">=", r)
    } else if let Some(r) = s.strip_prefix("<=") {
        ("<=", r)
    } else if let Some(r) = s.strip_prefix('>') {
        (">", r)
    } else if let Some(r) = s.strip_prefix('<') {
        ("<", r)
    } else if let Some(r) = s.strip_prefix('=') {
        ("=", r)
    } else {
        ("", s)
    };

    // If there is an operator, or the bare string parses as a number, try numeric.
    if !op.is_empty() || rest.parse::<f64>().is_ok() {
        if let Ok(n) = rest.parse::<f64>() {
            return match op {
                "<>" => Criterion::NumNe(n),
                ">=" => Criterion::NumGe(n),
                "<=" => Criterion::NumLe(n),
                ">"  => Criterion::NumGt(n),
                "<"  => Criterion::NumLt(n),
                _    => Criterion::NumEq(n), // "=" or bare number string
            };
        }
        // Non-numeric after operator.
        if op == "<>" {
            return Criterion::TextNe(rest.to_lowercase());
        }
        // Other operators with non-numeric text: degrade to TextEq of original.
        return Criterion::TextEq(s.to_lowercase());
    }

    // No operator prefix: check for wildcards.
    if rest.contains('*') || rest.contains('?') {
        return Criterion::WildcardEq(rest.to_lowercase().chars().collect());
    }

    Criterion::TextEq(rest.to_lowercase())
}

/// Test whether a `Value` satisfies a `Criterion`.
pub fn matches_criterion(value: &Value, crit: &Criterion) -> bool {
    match crit {
        Criterion::NumEq(n) => match value {
            Value::Number(v) => (v - n).abs() < 1e-10,
            _ => false,
        },
        Criterion::NumNe(n) => match value {
            Value::Number(v) => (v - n).abs() >= 1e-10,
            _ => true, // non-numbers are "not equal" to a number
        },
        Criterion::NumLt(n) => matches!(value, Value::Number(v) if v < n),
        Criterion::NumGt(n) => matches!(value, Value::Number(v) if v > n),
        Criterion::NumLe(n) => matches!(value, Value::Number(v) if v <= n),
        Criterion::NumGe(n) => matches!(value, Value::Number(v) if v >= n),
        Criterion::TextEq(pat) => match value {
            Value::Text(s) => s.to_lowercase() == *pat,
            Value::Bool(b) => {
                let s = if *b { "true" } else { "false" };
                s == pat.as_str()
            }
            _ => false,
        },
        Criterion::TextNe(pat) => match value {
            Value::Text(s) => s.to_lowercase() != *pat,
            _ => true,
        },
        Criterion::WildcardEq(pattern) => match value {
            Value::Text(s) => {
                let text: Vec<char> = s.to_lowercase().chars().collect();
                wildcard_match(pattern, &text)
            }
            _ => false,
        },
        Criterion::BoolEq(b) => matches!(value, Value::Bool(v) if v == b),
    }
}

/// Full wildcard match: `pattern` must match the entire `text`.
/// `*` matches any sequence of characters (including empty); `?` matches any single character.
fn wildcard_match(pattern: &[char], text: &[char]) -> bool {
    match (pattern.first(), text.first()) {
        (None, None) => true,
        (None, _) => false,
        (Some('*'), _) => {
            // Try consuming 0, 1, 2, … characters from text.
            for i in 0..=text.len() {
                if wildcard_match(&pattern[1..], &text[i..]) {
                    return true;
                }
            }
            false
        }
        (Some(_), None) => false,
        (Some(p), Some(t)) => {
            if *p == '?' || *p == *t {
                wildcard_match(&pattern[1..], &text[1..])
            } else {
                false
            }
        }
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Value;

    fn num(n: f64) -> Value { Value::Number(n) }
    fn text(s: &str) -> Value { Value::Text(s.to_string()) }

    #[test]
    fn numeric_eq() {
        let c = parse_criterion(&num(3.0));
        assert!(matches_criterion(&num(3.0), &c));
        assert!(!matches_criterion(&num(4.0), &c));
    }

    #[test]
    fn text_criterion_gt() {
        let c = parse_criterion(&text(">2"));
        assert!(matches_criterion(&num(3.0), &c));
        assert!(!matches_criterion(&num(1.0), &c));
    }

    #[test]
    fn text_criterion_ne_num() {
        let c = parse_criterion(&text("<>2"));
        assert!(matches_criterion(&num(3.0), &c));
        assert!(!matches_criterion(&num(2.0), &c));
    }

    #[test]
    fn text_criterion_exact() {
        let c = parse_criterion(&text("apple"));
        assert!(matches_criterion(&text("Apple"), &c)); // case-insensitive
        assert!(!matches_criterion(&text("banana"), &c));
    }

    #[test]
    fn text_criterion_wildcard_star() {
        let c = parse_criterion(&text("a*"));
        assert!(matches_criterion(&text("apple"), &c));
        assert!(matches_criterion(&text("a"), &c));
        assert!(!matches_criterion(&text("banana"), &c));
    }

    #[test]
    fn text_criterion_wildcard_question() {
        let c = parse_criterion(&text("ap?"));
        assert!(matches_criterion(&text("apt"), &c));
        assert!(matches_criterion(&text("ape"), &c));
        assert!(!matches_criterion(&text("apple"), &c));
    }

    #[test]
    fn bool_criterion() {
        let c = parse_criterion(&Value::Bool(true));
        assert!(matches_criterion(&Value::Bool(true), &c));
        assert!(!matches_criterion(&Value::Bool(false), &c));
    }

    #[test]
    fn flatten_array() {
        let arr = Value::Array(vec![num(1.0), num(2.0), num(3.0)]);
        let flat = flatten_to_vec(&arr);
        assert_eq!(flat.len(), 3);
    }

    #[test]
    fn flatten_scalar() {
        let v = num(5.0);
        let flat = flatten_to_vec(&v);
        assert_eq!(flat.len(), 1);
    }
}
