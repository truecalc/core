// crates/core/tests/conformance_reporter.rs
//
// Collects pass/fail counts per fixture and writes target/conformance-report.json.
// Called by the generate_conformance_report test in conformance.rs.

use calamine::{open_workbook, Data, Reader, Xlsx};
use ganit_core::{evaluate, Value};
use std::collections::HashMap;
use std::path::Path;

// Re-use the oracle helpers from the parent module via super::
// (oracle_to_value, values_match, is_volatile_formula are defined in conformance.rs
//  and visible here through the test module tree)

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

/// Known deviations: cases where ganit intentionally differs from Google Sheets.
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
            let expected = match super::oracle_to_value(&row[2]) {
                Some(v) => v,
                None => continue,
            };
            if super::is_volatile_formula(formula) {
                continue;
            }

            entry.total += 1;
            let actual = evaluate(formula, &vars);
            if super::values_match(&actual, &expected) {
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
