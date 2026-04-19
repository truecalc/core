//! Oracle conformance tests against Google Sheets reference values.
//!
//! Each `.xlsx` file under `tests/fixtures/m1/` is a Google Sheets export.
//! Each tab = one function name.  Each data row (skipping the header) has:
//!
//!   col A  description string
//!   col B  formula text stored as a plain string (e.g. "=ABS(-5)")
//!   col C  oracle value — the result Google Sheets produced
//!   col D  test category (basic / edge / coercion / error / nested)
//!
//! The test evaluates the formula in col B with `truecalc_core::evaluate`, then
//! compares against the oracle in col C.  Number comparisons allow a small
//! floating-point tolerance.  Rows whose oracle cell is empty are skipped.

use calamine::{open_workbook, CellErrorType, Data, Reader, Xlsx};
use truecalc_core::{evaluate, ErrorKind, Value};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

mod conformance_reporter;
use conformance_reporter::{collect_fixture_results, ConformanceReport, KNOWN_DEVIATIONS};

// ---------------------------------------------------------------------------
// helpers
// ---------------------------------------------------------------------------

fn fixture(milestone: &str, name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(milestone)
        .join(name)
}

/// Parse an error string like "#DIV/0!" into an ErrorKind, or return None.
fn parse_error_string(s: &str) -> Option<ErrorKind> {
    match s {
        "#DIV/0!" => Some(ErrorKind::DivByZero),
        "#VALUE!" => Some(ErrorKind::Value),
        "#REF!"   => Some(ErrorKind::Ref),
        "#NAME?"  => Some(ErrorKind::Name),
        "#NUM!"   => Some(ErrorKind::Num),
        "#N/A"    => Some(ErrorKind::NA),
        "#NULL!"  => Some(ErrorKind::Null),
        // Google Sheets generic parse/syntax error — treat as #VALUE! in our engine.
        "#ERROR!" => Some(ErrorKind::Value),
        _         => None,
    }
}

fn oracle_to_value(cell: &Data) -> Option<Value> {
    match cell {
        Data::Float(f)  => Some(Value::Number(*f)),
        Data::Int(i)    => Some(Value::Number(*i as f64)),
        // Google Sheets exports error cells as plain strings in xlsx; check both.
        Data::String(s) => {
            if let Some(kind) = parse_error_string(s.trim()) {
                Some(Value::Error(kind))
            } else {
                Some(Value::Text(s.clone()))
            }
        }
        Data::Bool(b)   => Some(Value::Bool(*b)),
        Data::Error(e)  => Some(Value::Error(match e {
            CellErrorType::Div0  => ErrorKind::DivByZero,
            CellErrorType::Value => ErrorKind::Value,
            CellErrorType::Ref   => ErrorKind::Ref,
            CellErrorType::Name  => ErrorKind::Name,
            CellErrorType::Num   => ErrorKind::Num,
            CellErrorType::NA    => ErrorKind::NA,
            CellErrorType::Null  => ErrorKind::Null,
            _                    => return None,
        })),
        Data::Empty | Data::DateTimeIso(_) | Data::DurationIso(_) | Data::DateTime(_) => None,
    }
}

/// Decode xlsx `_xNNNN_` XML escape sequences (e.g. "_x0001_" → U+0001).
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
        (Value::Number(a), Value::Number(b)) => {
            // Oracle values are stored with limited precision (≥5 significant digits).
            // Use 1e-4 relative tolerance to tolerate rounded oracle values while
            // still catching order-of-magnitude errors.
            (a - b).abs() <= b.abs() * 1e-4 + 1e-10
        }
        // Date values from our engine are typed as Value::Date, but the oracle
        // stores them as plain numbers in xlsx.
        (Value::Date(a), Value::Number(b)) => {
            (a - b).abs() <= b.abs() * 1e-4 + 1e-10
        }
        // Oracle artifact: xlsx stores numeric-looking text (e.g. "0", "123") as a float.
        // Text functions like CHAR, LOWER, CONCATENATE correctly return Text; the oracle
        // appears as Number due to calamine reading the cell as a float.
        (Value::Text(s), Value::Number(b)) => {
            if let Ok(v) = s.trim().parse::<f64>() {
                (v - b).abs() <= b.abs() * 1e-9 + 1e-10
            } else {
                false
            }
        }
        // Oracle artifact: xlsx strips non-printable control characters (U+0001..U+001F),
        // so CHAR(1) stored in the oracle cell appears as Text("").
        (Value::Text(s), Value::Text(e)) if e.is_empty() => {
            s.chars().all(|c| (c as u32) < 32)
        }
        // Oracle artifact: some xlsx writers encode control chars as `_xNNNN_` XML escapes.
        // UNICHAR(1) = U+0001 is stored as the literal string "_x0001_" by calamine.
        (Value::Text(s), Value::Text(e)) => {
            decode_xlsx_escapes(e) == *s
        }
        _ => actual == expected,
    }
}

/// Returns true if a formula contains volatile functions whose results are
/// non-deterministic and therefore cannot be verified against a fixed oracle.
fn is_volatile_formula(formula: &str) -> bool {
    let upper = formula.to_uppercase();
    upper.contains("RAND()") || upper.contains("RANDBETWEEN(") || upper.contains("RANDARRAY(")
}

fn run_fixture(path: &Path) {
    assert!(path.exists(), "fixture not found: {:?}", path);

    let mut workbook: Xlsx<_> = open_workbook(path)
        .unwrap_or_else(|e| panic!("failed to open {:?}: {}", path, e));

    let sheet_names: Vec<String> = workbook.sheet_names().to_vec();
    let vars: HashMap<String, Value> = HashMap::new();
    let mut failures: Vec<String> = Vec::new();
    let mut total = 0usize;

    for sheet_name in &sheet_names {
        let range = workbook
            .worksheet_range(sheet_name)
            .unwrap_or_else(|e| panic!("failed to read sheet {sheet_name}: {e}"));

        for (row_idx, row) in range.rows().enumerate().skip(1) {
            if row.len() < 3 {
                continue;
            }

            let desc = match &row[0] {
                Data::String(s) => s.as_str(),
                _ => continue,
            };
            let formula = match &row[1] {
                Data::String(s) => s.as_str(),
                _ => continue,
            };
            let expected = match oracle_to_value(&row[2]) {
                Some(v) => v,
                None => continue,
            };

            // Skip volatile functions whose oracle values are non-deterministic.
            if is_volatile_formula(formula) {
                continue;
            }

            total += 1;
            let actual = evaluate(formula, &vars);

            if !values_match(&actual, &expected) {
                failures.push(format!(
                    "  FAIL  [{sheet_name}] row {}  {desc}\n        formula:  {formula}\n        expected: {expected:?}\n        actual:   {actual:?}",
                    row_idx + 2,
                ));
            }
        }
    }

    if !failures.is_empty() {
        panic!(
            "\n{}/{} conformance failures in {}:\n\n{}\n",
            failures.len(),
            total,
            path.file_name().unwrap().to_string_lossy(),
            failures.join("\n\n"),
        );
    }
}

// ---------------------------------------------------------------------------
// one test per fixture file
// ---------------------------------------------------------------------------

macro_rules! conformance_test {
    ($fn_name:ident, $milestone:literal, $file:literal) => {
        #[test]
        fn $fn_name() {
            run_fixture(&fixture($milestone, $file));
        }
    };
    // Pending variant: fixture exists and defines the target, but the functions
    // are not yet implemented.  Run with `cargo test -- --include-ignored` to
    // see the full failure list and use it to drive implementation.
    (pending, $fn_name:ident, $milestone:literal, $file:literal) => {
        #[test]
        #[ignore = "pending implementation — run with --include-ignored to see failures"]
        fn $fn_name() {
            run_fixture(&fixture($milestone, $file));
        }
    };
}

conformance_test!(m1_math_conformance,        "m1", "Math.xlsx");
conformance_test!(m1_logical_conformance,     "m1", "Logical.xlsx");
conformance_test!(m1_info_conformance,        "m1", "Info.xlsx");
conformance_test!(m1_statistical_conformance, "m1", "Statistical.xlsx");
conformance_test!(m1_operator_conformance,    "m1", "Operator.xlsx");
conformance_test!(m1_text_conformance,        "m1", "Text.xlsx");

conformance_test!(m2_date_conformance,        "m2", "Date.xlsx");
conformance_test!(m2_engineering_conformance, "m2", "Engineering.xlsx");
conformance_test!(m2_info_conformance,        "m2", "Info.xlsx");
conformance_test!(m2_logical_conformance,     "m2", "Logical.xlsx");
conformance_test!(m2_lookup_conformance,               "m2", "Lookup.xlsx");
conformance_test!(m2_math_conformance,                 "m2", "Math.xlsx");
conformance_test!(m2_parser_conformance,               "m2", "Parser.xlsx");
conformance_test!(m2_statistical_conformance,          "m2", "Statistical.xlsx");
conformance_test!(m2_text_conformance,                 "m2", "Text.xlsx");

conformance_test!(m3_database_conformance,    "m3", "Database.xlsx");
conformance_test!(m3_engineering_conformance, "m3", "Engineering.xlsx");
conformance_test!(pending, m3_financial_conformance,   "m3", "Financial.xlsx");
conformance_test!(m3_info_conformance,        "m3", "Info.xlsx");
conformance_test!(pending, m3_lookup_conformance,      "m3", "Lookup.xlsx");
conformance_test!(m3_math_conformance,        "m3", "Math.xlsx");
conformance_test!(m3_statistical_conformance, "m3", "Statistical.xlsx");

conformance_test!(m4_array_conformance,       "m4", "Array.xlsx");
conformance_test!(m4_filter_conformance,               "m4", "Filter.xlsx");
conformance_test!(m4_info_conformance,                 "m4", "Info.xlsx");
conformance_test!(m4_logical_conformance,              "m4", "Logical.xlsx");
conformance_test!(m4_lookup_conformance,      "m4", "Lookup.xlsx");
conformance_test!(m4_math_conformance,        "m4", "Math.xlsx");
conformance_test!(m4_operator_conformance,    "m4", "Operator.xlsx");
conformance_test!(m4_web_conformance,                  "m4", "Web.xlsx");

// ---------------------------------------------------------------------------
// Conformance report generator — writes target/conformance-report.json
// ---------------------------------------------------------------------------

#[test]
fn generate_conformance_report() {
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));

    let mut report = ConformanceReport::default();
    report.known_deviations = KNOWN_DEVIATIONS.to_vec();

    // m1 fixtures
    collect_fixture_results(&fixture("m1", "Math.xlsx"),        "math",        &mut report);
    collect_fixture_results(&fixture("m1", "Logical.xlsx"),     "logical",     &mut report);
    collect_fixture_results(&fixture("m1", "Info.xlsx"),        "info",        &mut report);
    collect_fixture_results(&fixture("m1", "Statistical.xlsx"), "statistical", &mut report);
    collect_fixture_results(&fixture("m1", "Operator.xlsx"),    "operator",    &mut report);
    collect_fixture_results(&fixture("m1", "Text.xlsx"),        "text",        &mut report);

    // m2 fixtures
    collect_fixture_results(&fixture("m2", "Date.xlsx"),        "date",        &mut report);
    collect_fixture_results(&fixture("m2", "Engineering.xlsx"), "engineering", &mut report);
    collect_fixture_results(&fixture("m2", "Info.xlsx"),        "info",        &mut report);
    collect_fixture_results(&fixture("m2", "Logical.xlsx"),     "logical",     &mut report);
    collect_fixture_results(&fixture("m2", "Lookup.xlsx"),      "lookup",      &mut report);
    collect_fixture_results(&fixture("m2", "Math.xlsx"),        "math",        &mut report);
    collect_fixture_results(&fixture("m2", "Parser.xlsx"),      "parser",      &mut report);
    collect_fixture_results(&fixture("m2", "Statistical.xlsx"), "statistical", &mut report);
    collect_fixture_results(&fixture("m2", "Text.xlsx"),        "text",        &mut report);

    // m3 fixtures
    collect_fixture_results(&fixture("m3", "Database.xlsx"),    "database",    &mut report);
    collect_fixture_results(&fixture("m3", "Engineering.xlsx"), "engineering", &mut report);
    collect_fixture_results(&fixture("m3", "Financial.xlsx"),   "financial",   &mut report);
    collect_fixture_results(&fixture("m3", "Info.xlsx"),        "info",        &mut report);
    collect_fixture_results(&fixture("m3", "Lookup.xlsx"),      "lookup",      &mut report);
    collect_fixture_results(&fixture("m3", "Math.xlsx"),        "math",        &mut report);
    collect_fixture_results(&fixture("m3", "Statistical.xlsx"), "statistical", &mut report);

    // m4 fixtures
    collect_fixture_results(&fixture("m4", "Array.xlsx"),       "array",       &mut report);
    collect_fixture_results(&fixture("m4", "Filter.xlsx"),      "filter",      &mut report);
    collect_fixture_results(&fixture("m4", "Info.xlsx"),        "info",        &mut report);
    collect_fixture_results(&fixture("m4", "Logical.xlsx"),     "logical",     &mut report);
    collect_fixture_results(&fixture("m4", "Lookup.xlsx"),      "lookup",      &mut report);
    collect_fixture_results(&fixture("m4", "Math.xlsx"),        "math",        &mut report);
    collect_fixture_results(&fixture("m4", "Operator.xlsx"),    "operator",    &mut report);
    collect_fixture_results(&fixture("m4", "Web.xlsx"),         "web",         &mut report);

    // Write JSON to target/
    let out_dir = manifest.join("../../target");
    std::fs::create_dir_all(&out_dir).ok();
    let out_path = out_dir.join("conformance-report.json");
    std::fs::write(&out_path, report.to_json())
        .expect("failed to write conformance-report.json");

    println!("conformance-report.json written to {}", out_path.display());
    println!(
        "Total: {}/{} passed ({} failed)",
        report.total_passed(),
        report.total_tests(),
        report.total_failed(),
    );
}
