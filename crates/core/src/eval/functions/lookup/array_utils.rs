use crate::types::Value;

/// Glob-style wildcard match for spreadsheet patterns.
/// `*` matches any sequence of characters; `?` matches any single character.
/// Match is case-insensitive. The pattern must match the entire text.
pub fn wildcard_match_value(pattern: &Value, text: &Value) -> bool {
    let (pat_str, txt_str) = match (pattern, text) {
        (Value::Text(p), Value::Text(t)) => (p.to_lowercase(), t.to_lowercase()),
        _ => return false,
    };
    let pat: Vec<char> = pat_str.chars().collect();
    let txt: Vec<char> = txt_str.chars().collect();
    wildcard_match_chars(&pat, &txt)
}

fn wildcard_match_chars(pattern: &[char], text: &[char]) -> bool {
    match (pattern.first(), text.first()) {
        (None, None) => true,
        (None, _) => false,
        (Some('*'), _) => {
            for i in 0..=text.len() {
                if wildcard_match_chars(&pattern[1..], &text[i..]) {
                    return true;
                }
            }
            false
        }
        (Some(_), None) => false,
        (Some(p), Some(t)) => {
            if *p == '?' || *p == *t {
                wildcard_match_chars(&pattern[1..], &text[1..])
            } else {
                false
            }
        }
    }
}

/// Returns true if the pattern string contains wildcard characters (* or ?).
pub fn has_wildcards(v: &Value) -> bool {
    match v {
        Value::Text(s) => s.contains('*') || s.contains('?'),
        _ => false,
    }
}

/// Flatten a Value into a list of rows (Vec of Vec<Value>).
/// A 2D array (Array of Arrays) → each inner Array is a row.
/// A 1D array (Array of scalars) → one row containing all elements.
/// A scalar → one row with one element.
pub fn flatten_to_rows(v: &Value) -> Vec<Vec<Value>> {
    match v {
        Value::Array(outer) => {
            // Check if 2D: any element is itself an Array
            let is_2d = outer.iter().any(|e| matches!(e, Value::Array(_)));
            if is_2d {
                outer.iter().map(|row| match row {
                    Value::Array(cols) => cols.clone(),
                    other => vec![other.clone()],
                }).collect()
            } else {
                // 1D: one row
                vec![outer.clone()]
            }
        }
        other => vec![vec![other.clone()]],
    }
}

/// Flatten all values in a Value (including nested Arrays) into a single Vec<Value>.
pub fn flatten_to_flat(v: &Value) -> Vec<Value> {
    match v {
        Value::Array(elems) => elems.iter().flat_map(flatten_to_flat).collect(),
        other => vec![other.clone()],
    }
}

/// Compare two values for equality (case-insensitive for text).
pub fn values_equal(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Number(x), Value::Number(y)) => x == y,
        (Value::Bool(x), Value::Bool(y)) => x == y,
        (Value::Text(x), Value::Text(y)) => x.to_uppercase() == y.to_uppercase(),
        (Value::Empty, Value::Empty) => true,
        _ => false,
    }
}

/// Map a Value to a numeric ordering key for cross-type comparison.
/// In spreadsheets: numbers < text < booleans (for LOOKUP/MATCH sorting purposes).
/// Within booleans: FALSE (0) < TRUE (1).
fn type_rank(v: &Value) -> u8 {
    match v {
        Value::Number(_) => 0,
        Value::Text(_)   => 1,
        Value::Bool(_)   => 2,
        _                => 3,
    }
}

/// Compare two values for ordering (for approximate match / LOOKUP / MATCH).
/// Returns None only if both values are of incomparable types (e.g. error vs text).
/// Same-type comparisons always return Some.
/// Cross-type: numbers < text < booleans (spreadsheet convention).
pub fn value_compare(a: &Value, b: &Value) -> Option<std::cmp::Ordering> {
    match (a, b) {
        (Value::Number(x), Value::Number(y)) => x.partial_cmp(y),
        (Value::Text(x), Value::Text(y)) => Some(x.to_uppercase().cmp(&y.to_uppercase())),
        (Value::Bool(x), Value::Bool(y)) => Some(x.cmp(y)),
        // Cross-type: use type rank
        _ => {
            let ra = type_rank(a);
            let rb = type_rank(b);
            if ra != rb {
                Some(ra.cmp(&rb))
            } else {
                None
            }
        }
    }
}
