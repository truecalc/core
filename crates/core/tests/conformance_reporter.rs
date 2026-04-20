// crates/core/tests/conformance_reporter.rs
//
// Collects pass/fail counts per TSV fixture and writes target/conformance-report.json.
// Called by the generate_conformance_report test in conformance.rs.

use truecalc_core::{evaluate, Value};
use std::collections::HashMap;
use std::path::Path;

// ---------------------------------------------------------------------------
// Oracle helpers — shared logic duplicated from conformance.rs because
// conformance_reporter.rs compiles as a standalone integration-test module
// ---------------------------------------------------------------------------

fn parse_expected(value: &str, expected_type: &str) -> Option<Value> {
    use truecalc_core::ErrorKind;
    match expected_type {
        "number" => value.parse::<f64>().ok().map(Value::Number),
        "boolean" => match value.to_uppercase().as_str() {
            "TRUE"  => Some(Value::Bool(true)),
            "FALSE" => Some(Value::Bool(false)),
            _       => None,
        },
        "error" => parse_error_string(value).map(Value::Error),
        "string" | "array" => Some(Value::Text(value.to_string())),
        _ => Some(Value::Text(value.to_string())),
    }
}

fn parse_error_string(s: &str) -> Option<truecalc_core::ErrorKind> {
    use truecalc_core::ErrorKind;
    match s {
        "#DIV/0!" => Some(ErrorKind::DivByZero),
        "#VALUE!" => Some(ErrorKind::Value),
        "#REF!"   => Some(ErrorKind::Ref),
        "#NAME?"  => Some(ErrorKind::Name),
        "#NUM!"   => Some(ErrorKind::Num),
        "#N/A"    => Some(ErrorKind::NA),
        "#NULL!"  => Some(ErrorKind::Null),
        "#ERROR!" => Some(ErrorKind::Value),
        _         => None,
    }
}

fn parse_array_literal(s: &str) -> Option<Vec<Value>> {
    let s = s.trim();
    if !s.starts_with('{') || !s.ends_with('}') {
        return None;
    }
    let inner = &s[1..s.len() - 1];
    let items: Vec<&str> = inner.split(|c| c == ',' || c == ';').collect();
    let mut result = Vec::new();
    for item in items {
        let item = item.trim().trim_matches('"');
        if let Some(kind) = parse_error_string(item) {
            result.push(Value::Error(kind));
        } else if item.eq_ignore_ascii_case("true") {
            result.push(Value::Bool(true));
        } else if item.eq_ignore_ascii_case("false") {
            result.push(Value::Bool(false));
        } else if let Ok(f) = item.parse::<f64>() {
            result.push(Value::Number(f));
        } else {
            result.push(Value::Text(item.to_string()));
        }
    }
    Some(result)
}

fn flatten_array(v: &Value) -> Vec<Value> {
    match v {
        Value::Array(items) => {
            let mut flat = Vec::new();
            for item in items {
                match item {
                    Value::Array(inner) => flat.extend(inner.iter().cloned()),
                    other => flat.push(other.clone()),
                }
            }
            flat
        }
        other => vec![other.clone()],
    }
}

fn infer_type(v: &Value) -> &'static str {
    match v {
        Value::Number(_) | Value::Date(_) => "number",
        Value::Text(_) => "string",
        Value::Bool(_) => "boolean",
        Value::Error(_) => "error",
        Value::Array(_) => "array",
        Value::Empty => "string",
    }
}

fn values_match(actual: &Value, expected: &Value, expected_type: &str) -> bool {
    if expected_type == "array" {
        let literal = match expected {
            Value::Text(s) => s.as_str(),
            _ => return false,
        };
        let expected_items = match parse_array_literal(literal) {
            Some(items) => items,
            None => return false,
        };
        let actual_items = flatten_array(actual);
        if actual_items.len() != expected_items.len() {
            return false;
        }
        return actual_items.iter().zip(expected_items.iter()).all(|(a, e)| {
            values_match(a, e, infer_type(e))
        });
    }

    match (actual, expected) {
        (Value::Number(a), Value::Number(b)) => (a - b).abs() <= b.abs() * 1e-4 + 1e-10,
        (Value::Date(a), Value::Number(b))   => (a - b).abs() <= b.abs() * 1e-4 + 1e-10,
        (Value::Text(s), Value::Number(b)) => {
            if let Ok(v) = s.trim().parse::<f64>() {
                (v - b).abs() <= b.abs() * 1e-9 + 1e-10
            } else {
                false
            }
        }
        (Value::Text(s), Value::Text(e)) if e.is_empty() => s.chars().all(|c| (c as u32) < 32),
        (Value::Text(s), Value::Text(e)) => s == e,
        (Value::Error(a), Value::Error(b)) => a == b,
        _ => actual == expected,
    }
}

fn is_volatile_formula(formula: &str) -> bool {
    let upper = formula.to_uppercase();
    upper.contains("RAND()") || upper.contains("RANDBETWEEN(") || upper.contains("RANDARRAY(")
}

// ---------------------------------------------------------------------------
// Report types
// ---------------------------------------------------------------------------

#[derive(Default, Debug)]
pub struct CategoryResult {
    pub passed: usize,
    pub total: usize,
    pub failures: Vec<String>,
}

#[derive(Default, Debug)]
pub struct ConformanceReport {
    pub by_category: HashMap<String, CategoryResult>,
    pub known_deviations: Vec<KnownDeviation>,
}

#[derive(Debug, Clone)]
pub struct KnownDeviation {
    pub formula: &'static str,
    pub reason: &'static str,
}

impl ConformanceReport {
    pub fn total_passed(&self) -> usize {
        self.by_category.values().map(|r| r.passed).sum()
    }
    pub fn total_tests(&self) -> usize {
        self.by_category.values().map(|r| r.total).sum()
    }
    pub fn total_failed(&self) -> usize {
        self.total_tests() - self.total_passed()
    }

    pub fn to_json(&self) -> String {
        let mut s = String::new();
        s.push_str("{\n");
        s.push_str(&format!("  \"total\": {},\n", self.total_tests()));
        s.push_str(&format!("  \"passed\": {},\n", self.total_passed()));
        s.push_str(&format!("  \"failed\": {},\n", self.total_failed()));
        s.push_str("  \"by_category\": {\n");
        let mut cats: Vec<(&String, &CategoryResult)> = self.by_category.iter().collect();
        cats.sort_by_key(|(k, _)| k.as_str());
        for (i, (cat, result)) in cats.iter().enumerate() {
            let comma = if i + 1 < cats.len() { "," } else { "" };
            s.push_str(&format!(
                "    \"{}\": {{ \"passed\": {}, \"total\": {} }}{}\n",
                cat, result.passed, result.total, comma
            ));
        }
        s.push_str("  },\n");
        s.push_str("  \"known_deviations\": [\n");
        for (i, dev) in self.known_deviations.iter().enumerate() {
            let comma = if i + 1 < self.known_deviations.len() { "," } else { "" };
            s.push_str(&format!(
                "    {{ \"formula\": \"{}\", \"reason\": \"{}\" }}{}\n",
                dev.formula.replace('"', "\\\""),
                dev.reason.replace('"', "\\\""),
                comma
            ));
        }
        s.push_str("  ]\n");
        s.push_str("}\n");
        s
    }
}

/// Known deviations: cases where truecalc intentionally differs from Google Sheets.
pub const KNOWN_DEVIATIONS: &[KnownDeviation] = &[];

/// Collect pass/fail for a single TSV fixture file. Returns results without panicking.
pub fn collect_tsv_fixture_results(path: &Path, category: &str, report: &mut ConformanceReport) {
    if !path.exists() {
        return;
    }

    let mut rdr = match csv::ReaderBuilder::new()
        .delimiter(b'\t')
        .has_headers(true)
        .from_path(path)
    {
        Ok(r) => r,
        Err(_) => return,
    };

    let vars: HashMap<String, Value> = HashMap::new();
    let entry = report.by_category.entry(category.to_string()).or_default();

    for (row_idx, result) in rdr.records().enumerate() {
        let record = match result {
            Ok(r) => r,
            Err(_) => continue,
        };
        if record.len() < 5 {
            continue;
        }

        let desc          = record[0].trim().to_string();
        let formula       = record[1].trim().to_string();
        let expected_str  = record[2].trim().to_string();
        let expected_type = record[4].trim().to_string();

        if formula.is_empty() || expected_str.is_empty() {
            continue;
        }
        if is_volatile_formula(&formula) {
            continue;
        }

        let expected = match parse_expected(&expected_str, &expected_type) {
            Some(v) => v,
            None => continue,
        };

        entry.total += 1;
        let actual = evaluate(&formula, &vars);
        if values_match(&actual, &expected, &expected_type) {
            entry.passed += 1;
        } else {
            entry.failures.push(format!(
                "row {} {}: formula={} expected={:?} got={:?}",
                row_idx + 2,
                desc,
                formula,
                expected,
                actual
            ));
        }
    }
}
