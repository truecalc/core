// crates/core/tests/conformance_reporter.rs
//
// Collects pass/fail counts per fixture and writes target/conformance-report.json.
// Called by the generate_conformance_report test in conformance.rs.

use calamine::{open_workbook, CellErrorType, Data, Reader, Xlsx};
use truecalc_core::{evaluate, ErrorKind, Value};
use std::collections::HashMap;
use std::path::Path;

// ---------------------------------------------------------------------------
// Oracle helpers (mirrors of the same functions in conformance.rs)
// Duplicated here so that conformance_reporter.rs compiles as a standalone
// integration test target (where super:: would be out of scope).
// ---------------------------------------------------------------------------

fn parse_error_string(s: &str) -> Option<ErrorKind> {
    match s {
        "#DIV/0!" => Some(ErrorKind::DivByZero),
        "#VALUE!" => Some(ErrorKind::Value),
        "#REF!" => Some(ErrorKind::Ref),
        "#NAME?" => Some(ErrorKind::Name),
        "#NUM!" => Some(ErrorKind::Num),
        "#N/A" => Some(ErrorKind::NA),
        "#NULL!" => Some(ErrorKind::Null),
        "#ERROR!" => Some(ErrorKind::Value),
        _ => None,
    }
}

fn oracle_to_value(cell: &Data) -> Option<Value> {
    match cell {
        Data::Float(f) => Some(Value::Number(*f)),
        Data::Int(i) => Some(Value::Number(*i as f64)),
        Data::String(s) => {
            if let Some(kind) = parse_error_string(s.trim()) {
                Some(Value::Error(kind))
            } else {
                Some(Value::Text(s.clone()))
            }
        }
        Data::Bool(b) => Some(Value::Bool(*b)),
        Data::Error(e) => Some(Value::Error(match e {
            CellErrorType::Div0 => ErrorKind::DivByZero,
            CellErrorType::Value => ErrorKind::Value,
            CellErrorType::Ref => ErrorKind::Ref,
            CellErrorType::Name => ErrorKind::Name,
            CellErrorType::Num => ErrorKind::Num,
            CellErrorType::NA => ErrorKind::NA,
            CellErrorType::Null => ErrorKind::Null,
            _ => return None,
        })),
        Data::Empty | Data::DateTimeIso(_) | Data::DurationIso(_) | Data::DateTime(_) => None,
    }
}

fn decode_xlsx_escapes(s: &str) -> String {
    let mut result = String::new();
    let mut rest = s;
    while let Some(start) = rest.find("_x") {
        result.push_str(&rest[..start]);
        let after = &rest[start + 2..];
        if let Some(end) = after.find('_') {
            let hex = &after[..end];
            if hex.len() == 4 && hex.chars().all(|c| c.is_ascii_hexdigit()) {
                if let Ok(n) = u32::from_str_radix(hex, 16) {
                    if let Some(c) = char::from_u32(n) {
                        result.push(c);
                        rest = &after[end + 1..];
                        continue;
                    }
                }
            }
        }
        result.push_str("_x");
        rest = after;
    }
    result.push_str(rest);
    result
}

fn values_match(actual: &Value, expected: &Value) -> bool {
    match (actual, expected) {
        (Value::Number(a), Value::Number(b)) => (a - b).abs() <= b.abs() * 1e-4 + 1e-10,
        (Value::Date(a), Value::Number(b)) => (a - b).abs() <= b.abs() * 1e-4 + 1e-10,
        (Value::Text(s), Value::Number(b)) => {
            if let Ok(v) = s.trim().parse::<f64>() {
                (v - b).abs() <= b.abs() * 1e-9 + 1e-10
            } else {
                false
            }
        }
        (Value::Text(s), Value::Text(e)) if e.is_empty() => {
            s.chars().all(|c| (c as u32) < 32)
        }
        (Value::Text(s), Value::Text(e)) => decode_xlsx_escapes(e) == *s,
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
            let comma = if i + 1 < self.known_deviations.len() {
                ","
            } else {
                ""
            };
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
/// These are excluded from failure counts but documented in the report.
pub const KNOWN_DEVIATIONS: &[KnownDeviation] = &[
    // Add entries here as they are discovered. Format:
    // KnownDeviation { formula: "=RATE(4*12,-200,8000)", reason: "floating-point iteration limit differs" },
];

/// Collect pass/fail for a single fixture file. Returns results without panicking.
/// `category` is the logical name used in the JSON (e.g. "math", "text").
pub fn collect_fixture_results(path: &Path, category: &str, report: &mut ConformanceReport) {
    if !path.exists() {
        return;
    }

    let mut workbook: Xlsx<_> = match open_workbook(path) {
        Ok(w) => w,
        Err(_) => return,
    };

    let sheet_names: Vec<String> = workbook.sheet_names().to_vec();
    let vars: HashMap<String, Value> = HashMap::new();
    let entry = report.by_category.entry(category.to_string()).or_default();

    for sheet_name in &sheet_names {
        let range = match workbook.worksheet_range(sheet_name) {
            Ok(r) => r,
            Err(_) => continue,
        };

        for (row_idx, row) in range.rows().enumerate().skip(1) {
            if row.len() < 3 {
                continue;
            }
            let formula = match &row[1] {
                Data::String(s) => s.as_str(),
                _ => continue,
            };
            let expected = match oracle_to_value(&row[2]) {
                Some(v) => v,
                None => continue,
            };
            if is_volatile_formula(formula) {
                continue;
            }

            entry.total += 1;
            let actual = evaluate(formula, &vars);
            if values_match(&actual, &expected) {
                entry.passed += 1;
            } else {
                let desc = match &row[0] {
                    Data::String(s) => s.clone(),
                    _ => String::new(),
                };
                entry.failures.push(format!(
                    "[{}] row {} {}: formula={} expected={:?} got={:?}",
                    sheet_name,
                    row_idx + 2,
                    desc,
                    formula,
                    expected,
                    actual
                ));
            }
        }
    }
}
