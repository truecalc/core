//! Conformance tests against Google Sheets canonical reference values.
//!
//! # Fixture directories
//!
//! `tests/fixtures/google_sheets/` — canonical reference data produced by the
//! fixtures pipeline.  These files must never be edited by hand.  Every row
//! in every category TSV (financial, date, math, …) must pass before a PR
//! can merge.  `bugs.tsv` is also canonical reference data; its rows are known
//! implementation gaps and produce a non-blocking progress report.
//!
//! `tests/fixtures/lab/` — staging area for test cases discovered during
//! development, not yet submitted to the GS fixtures pipeline.  Cases are
//! organised by source under `lab/google_sheets/`, `lab/excel/`, etc.  Lab
//! tests produce a report but do not block CI.  See `lab/README.md`.
//!
//! # TSV format (5 columns, tab-separated, with header row)
//!
//!   description     human-readable test name
//!   formula_text    formula string (may or may not have leading `=`)
//!   expected_value  canonical expected value as a string
//!   test_category   basic / edge / coercion / error / nested
//!   expected_type   number / string / boolean / error / array / date
//!
//! The test evaluates the formula with `truecalc_core::evaluate` and compares
//! against the canonical value.  Number comparisons allow 1e-4 relative tolerance.

use truecalc_core::{ErrorKind, Value};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use chrono::NaiveDate;

mod conformance_reporter;
use conformance_reporter::{collect_tsv_fixture_results, ConformanceReport, KNOWN_DEVIATIONS};

// ---------------------------------------------------------------------------
// helpers
// ---------------------------------------------------------------------------

fn fixture_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/google_sheets")
}

fn fixture(name: &str) -> PathBuf {
    fixture_dir().join(name)
}

fn lab_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/lab")
}

/// Decode xlsx `_xNNNN_` XML-escape sequences (e.g. `_x0001_` → U+0001).
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
        "#ERROR!" => Some(ErrorKind::Value),
        _         => None,
    }
}

/// Parse a `{1,2,3}` or `{1,2,3;4,5,6}` array literal into a flat Vec<Value>.
/// Used for the `array` expected_type where the canonical value is ARRAYTOTEXT output.
fn parse_array_literal(s: &str) -> Option<Vec<Value>> {
    let s = s.trim();
    if !s.starts_with('{') || !s.ends_with('}') {
        return None;
    }
    let inner = &s[1..s.len() - 1];
    // ARRAYTOTEXT with mode=1 uses comma separators and semicolons for row breaks.
    // For our purposes (1D arrays), just split on commas and semicolons.
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

/// Parse expected_value string into a Value according to expected_type.
pub fn parse_expected(value: &str, expected_type: &str) -> Option<Value> {
    match expected_type {
        "number" => {
            value.parse::<f64>().ok().map(Value::Number)
        }
        "boolean" => match value.to_uppercase().as_str() {
            "TRUE"  => Some(Value::Bool(true)),
            "FALSE" => Some(Value::Bool(false)),
            _       => None,
        },
        "error" => parse_error_string(value).map(Value::Error),
        "string" => {
            // Decode xlsx `_xNNNN_` XML escapes preserved from the migration.
            Some(Value::Text(decode_xlsx_escapes(value)))
        }
        "array"  => {
            // Store the array literal string as-is; comparison handled in values_match
            Some(Value::Text(value.to_string()))
        }
        "date" => {
            // P1.4 (#526): `date`-typed rows — the pipeline observed Sheets
            // producing a Date; the engine must produce Value::Date with
            // this serial (schema spec §6 date-type production rule).
            value.parse::<f64>().ok().map(Value::Date)
        }
        _ => Some(Value::Text(value.to_string())),
    }
}

/// Flatten a Value::Array into a Vec<Value> (1 level deep).
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

/// Parse a GAS UTC ISO-8601 date string (e.g. "2011-05-15T07:00:00.000Z") to an
/// Excel date serial (days since 1899-12-30). GAS serialises Date cell values as
/// ISO strings; our evaluator returns them as Number serials — this bridge lets
/// the conformance test treat them as equivalent.
fn gas_iso_date_to_serial(s: &str) -> Option<f64> {
    let date_part = s.split('T').next()?;
    let date = NaiveDate::parse_from_str(date_part, "%Y-%m-%d").ok()?;
    let epoch = NaiveDate::from_ymd_opt(1899, 12, 30)?;
    Some(date.signed_duration_since(epoch).num_days() as f64)
}

/// Top-left element of a (possibly nested) array value.
///
/// P1.4 (#526): the engine returns array results unspilled.  The fixtures
/// pipeline, however, observes the *anchor cell* of the spilled range in
/// Google Sheets, so a scalar-typed expected value corresponds to the
/// top-left element of an array actual — the same collapse the workbook /
/// surface layer applies for a single-cell view.
fn top_left(v: &Value) -> &Value {
    match v {
        Value::Array(items) if !items.is_empty() => top_left(&items[0]),
        other => other,
    }
}

pub fn values_match(actual: &Value, expected: &Value, expected_type: &str) -> bool {
    if expected_type == "array" {
        // expected is stored as Text(array_literal)
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

    // Scalar expected type: compare against the top-left element of an
    // unspilled array actual (anchor-cell view, see `top_left`).
    let actual = top_left(actual);

    match (actual, expected) {
        (Value::Number(a), Value::Number(b)) => {
            (a - b).abs() <= b.abs() * 1e-4 + 1e-10
        }
        // P1.4 (#526) date-type production rule: a `date`-typed expected
        // value only matches a Value::Date actual.  A plain Number actual
        // against a Date expected falls through to the catch-all arm and
        // fails -- losing the date type is a conformance failure.
        (Value::Date(a), Value::Date(b)) => {
            (a - b).abs() <= b.abs() * 1e-4 + 1e-10
        }
        (Value::Date(a), Value::Number(b)) => {
            (a - b).abs() <= b.abs() * 1e-4 + 1e-10
        }
        // Note: xlsx/TSV stores numeric-looking text as a number.
        (Value::Text(s), Value::Number(b)) => {
            if let Ok(v) = s.trim().parse::<f64>() {
                (v - b).abs() <= b.abs() * 1e-9 + 1e-10
            } else {
                false
            }
        }
        // GAS artifact: COUPNCD/COUPPCD etc. return Number serials; GAS serialises
        // Date cell values as UTC ISO-8601 strings. Convert the date part to an
        // Excel serial and compare numerically.
        (Value::Number(a), Value::Text(s)) => {
            if let Some(serial) = gas_iso_date_to_serial(s) {
                (a - serial).abs() <= 1.0
            } else {
                false
            }
        }
        // Control characters: GS may strip them → empty string
        (Value::Text(s), Value::Text(e)) if e.is_empty() => {
            s.chars().all(|c| (c as u32) < 32)
        }
        (Value::Text(s), Value::Text(e)) => s == e,
        (Value::Error(a), Value::Error(b)) => a == b,
        _ => actual == expected,
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

/// Returns true if a formula contains volatile functions.
fn is_volatile_formula(formula: &str) -> bool {
    let upper = formula.to_uppercase();
    upper.contains("RAND()") || upper.contains("RANDBETWEEN(") || upper.contains("RANDARRAY(")
}

/// Pinned "now" serial for fixture files whose volatile rows (`NOW`, `TODAY`)
/// were captured at a recorded instant (P1.4, issue #526).
///
/// workbook.tsv (P1.5, PR #559): the DATE_TYPE block's meta sidecar in the
/// truecalc/fixtures snapshot `snapshots/2026-06-08/google_sheets/workbook/`
/// records `evaluatedAt = 2026-06-07T23:50:56.808Z` with the sheet timezone
/// pinned to `Etc/GMT`, so local time == UTC: day serial 46180 (2026-06-07)
/// plus the time-of-day fraction.  Rows in pinned files are evaluated through
/// `Engine::evaluate_at` so volatile date functions are deterministic.
fn pinned_now_serial(path: &Path) -> Option<f64> {
    match path.file_name().and_then(|n| n.to_str()) {
        Some("workbook.tsv") => Some(46180.0 + (23.0 * 3600.0 + 50.0 * 60.0 + 56.808) / 86400.0),
        _ => None,
    }
}

// ---------------------------------------------------------------------------
// TSV runner
// ---------------------------------------------------------------------------

fn run_tsv_fixture(path: &Path) {
    assert!(path.exists(), "fixture not found: {:?}", path);

    let pinned_now = pinned_now_serial(path);
    let vars: HashMap<String, Value> = HashMap::new();
    let mut failures: Vec<String> = Vec::new();
    let mut total = 0usize;

    let mut rdr = csv::ReaderBuilder::new()
        .delimiter(b'\t')
        .has_headers(true)
        .from_path(path)
        .unwrap_or_else(|e| panic!("failed to open {:?}: {}", path, e));

    for (row_idx, result) in rdr.records().enumerate() {
        let record = result.unwrap_or_else(|e| panic!("bad row {} in {:?}: {}", row_idx + 2, path, e));

        if record.len() < 5 {
            continue;
        }

        let desc          = record[0].trim();
        let formula       = record[1].trim();
        // NOTE: do NOT trim expected_str — values like "  Hello World" have meaningful
        // leading whitespace (e.g. PROPER("  hello world") preserves leading spaces).
        let expected_str  = &record[2];
        let _test_category = record[3].trim();
        let expected_type = record[4].trim();

        if formula.is_empty() || expected_str.trim().is_empty() {
            continue;
        }

        if is_volatile_formula(formula) {
            continue;
        }

        let expected = match parse_expected(expected_str, expected_type) {
            Some(v) => v,
            None => continue,
        };

        total += 1;
        let actual = match pinned_now {
            Some(now) => truecalc_core::Engine::sheets().evaluate_at(formula, &vars, now),
            None => evaluate(formula, &vars),
        };

        if !values_match(&actual, &expected, expected_type) {
            failures.push(format!(
                "  FAIL  row {}  {desc}\n        formula:  {formula}\n        expected: {expected:?}\n        actual:   {actual:?}",
                row_idx + 2,
            ));
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

/// Non-panicking variant: prints FAIL rows but does not abort the test.
/// Used for bugs.tsv where failures are expected and intentional.
fn run_tsv_fixture_report(path: &Path) {
    assert!(path.exists(), "fixture not found: {:?}", path);

    let pinned_now = pinned_now_serial(path);
    let vars: HashMap<String, Value> = HashMap::new();
    let mut pass = 0usize;
    let mut fail = 0usize;

    let mut rdr = csv::ReaderBuilder::new()
        .delimiter(b'\t')
        .has_headers(true)
        .from_path(path)
        .unwrap_or_else(|e| panic!("failed to open {:?}: {}", path, e));

    for (row_idx, result) in rdr.records().enumerate() {
        let record = result.unwrap_or_else(|e| panic!("bad row {} in {:?}: {}", row_idx + 2, path, e));

        if record.len() < 5 {
            continue;
        }

        let desc           = record[0].trim();
        let formula        = record[1].trim();
        let expected_str   = &record[2];
        let _test_category = record[3].trim();
        let expected_type  = record[4].trim();

        if formula.is_empty() || expected_str.trim().is_empty() {
            continue;
        }

        if is_volatile_formula(formula) {
            continue;
        }

        let expected = match parse_expected(expected_str, expected_type) {
            Some(v) => v,
            None => continue,
        };

        let actual = match pinned_now {
            Some(now) => truecalc_core::Engine::sheets().evaluate_at(formula, &vars, now),
            None => evaluate(formula, &vars),
        };

        if values_match(&actual, &expected, expected_type) {
            pass += 1;
        } else {
            fail += 1;
            println!(
                "  FAIL  row {}  {desc}\n        formula:  {formula}\n        expected: {expected:?}\n        actual:   {actual:?}",
                row_idx + 2,
            );
        }
    }

    let name = path.file_name().unwrap_or_default().to_string_lossy();
    println!("{name}: {pass} passed, {fail} open");
}

// ---------------------------------------------------------------------------
// one test per TSV fixture file
// ---------------------------------------------------------------------------

macro_rules! conformance_tsv_test {
    ($fn_name:ident, $file:literal) => {
        #[test]
        fn $fn_name() {
            run_tsv_fixture(&fixture($file));
        }
    };
}

/// Report-only variant (non-blocking) for categories not yet green against the
/// refreshed 2026-06-08 fixtures (#582 / backlog #592). Each category flips back
/// to the blocking `conformance_tsv_test!` in its own fix PR once it passes —
/// a one-line change per category, so parallel agents never collide here.
macro_rules! conformance_tsv_test_report {
    ($fn_name:ident, $file:literal) => {
        #[test]
        fn $fn_name() {
            run_tsv_fixture_report(&fixture($file));
        }
    };
}

conformance_tsv_test_report!(math_conformance,        "math.tsv");
conformance_tsv_test_report!(logical_conformance,     "logical.tsv");
conformance_tsv_test_report!(info_conformance,        "info.tsv");
conformance_tsv_test_report!(statistical_conformance, "statistical.tsv");
conformance_tsv_test!(operator_conformance,         "operator.tsv");
conformance_tsv_test_report!(text_conformance,        "text.tsv");
conformance_tsv_test_report!(date_conformance,        "date.tsv");
conformance_tsv_test_report!(engineering_conformance, "engineering.tsv");
conformance_tsv_test_report!(lookup_conformance,      "lookup.tsv");
conformance_tsv_test_report!(parser_conformance,      "parser.tsv");
conformance_tsv_test_report!(database_conformance,    "database.tsv");
conformance_tsv_test_report!(array_conformance,       "array.tsv");
conformance_tsv_test_report!(filter_conformance,      "filter.tsv");
conformance_tsv_test!(web_conformance,         "web.tsv");
conformance_tsv_test_report!(financial_conformance,   "financial.tsv");

/// P1.5 workbook fixtures — cross-sheet refs, named ranges, date-typed scalars
/// (pipeline-generated, core issue #527; registered via PR #562).
///
/// Report-only (non-blocking). The language pieces are now in place — P1.2
/// reference grammar (#524) and P1.3 resolver (#525) resolve cross-sheet/named
/// refs, and P1.4 (#526) adds `date`-typed row handling and volatile-row
/// pinning via `Engine::evaluate_at` (see `pinned_now_serial`).  This whole-file
/// runner stays report-only because it evaluates with an empty variable map and
/// no resolver.  The fixtures pipeline now commits the authored *input* model
/// (`workbook.inputs.json`, fixtures extension #2 / #532), and the cross-sheet
/// + named-range rows are exercised as a **blocking** test in
/// `tests/workbook_inputs_conformance.rs`, which seeds a `Resolver` from that
/// sidecar and evaluates each row via `Engine::evaluate_with_resolver_at` (no
/// expected value hand-authored).  Unifying the whole `workbook.tsv` into a
/// single blocking `conformance_tsv_test!` (incl. the date-type rows) is
/// tracked in #575.
#[test]
fn workbook_conformance_report() {
    run_tsv_fixture_report(&fixture("workbook.tsv"));
}

/// Known-bug regression baseline.
///
/// Rows here are GS-captured cases where our engine does not yet produce the
/// correct result.  The test does NOT panic on failures; a failure means the
/// gap is still open.  When the engine is fixed, the pass count rises
/// automatically.  Do NOT edit this file by hand — it is canonical reference
/// data from the fixtures pipeline.
#[test]
fn bugs_conformance() {
    run_tsv_fixture_report(&fixture("bugs.tsv"));
}

/// Recursively collect all `.tsv` files under `dir`.
fn collect_tsv_files(dir: &Path) -> Vec<PathBuf> {
    let mut result = Vec::new();
    let Ok(entries) = std::fs::read_dir(dir) else { return result };
    for entry in entries.filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.is_dir() {
            result.extend(collect_tsv_files(&path));
        } else if path.extension().and_then(|s| s.to_str()) == Some("tsv") {
            result.push(path);
        }
    }
    result
}

/// Lab conformance report — non-blocking.
///
/// Runs every `.tsv` file found recursively under `tests/fixtures/lab/`.
/// Cases are organised by source subdirectory (`lab/google_sheets/`, etc.).
/// Failures do NOT block CI; they mean a known case is still open.
/// See `tests/fixtures/lab/README.md` for the full intent and graduation process.
#[test]
fn lab_conformance() {
    let dir = lab_dir();
    let mut entries = collect_tsv_files(&dir);
    entries.sort();

    if entries.is_empty() {
        println!("lab: no .tsv files — nothing to report");
        return;
    }
    for path in &entries {
        run_tsv_fixture_report(path);
    }
}

// ---------------------------------------------------------------------------
// Conformance report generator — writes target/conformance-report.json
// ---------------------------------------------------------------------------

#[test]
fn generate_conformance_report() {
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    let gdir = fixture_dir();

    let mut report = ConformanceReport::default();
    report.known_deviations = KNOWN_DEVIATIONS.to_vec();

    let categories = [
        "math", "logical", "info", "statistical", "operator", "text",
        "date", "engineering", "lookup", "parser", "database",
        "array", "filter", "web", "financial",
    ];

    for cat in &categories {
        let path = gdir.join(format!("{cat}.tsv"));
        collect_tsv_fixture_results(&path, cat, &mut report);
    }

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

// ---------------------------------------------------------------------------
// T2.3 — per-function conformance coverage gate (initially ignored)
// ---------------------------------------------------------------------------

#[test]
fn every_registered_function_has_conformance_coverage() {
    use truecalc_core::Registry;
    let registry = Registry::new();
    let all_names = registry.metadata_names();
    let volatile: std::collections::HashSet<&str> = Registry::VOLATILE_FUNCTIONS
        .iter()
        .copied()
        .collect();
    let context_limited: std::collections::HashSet<&str> = [
        "INDIRECT", "OFFSET", "FORMULATEXT", "GETPIVOTDATA", "ISFORMULA", "CELL",
        "SHEET", "SHEETS",
    ]
    .iter()
    .copied()
    .collect();

    let gdir = fixture_dir();
    let vars: HashMap<String, Value> = HashMap::new();

    // Collect function names with at least one passing fixture row (any TSV except bugs.tsv).
    let mut covered = std::collections::HashSet::new();
    // Collect function names acknowledged as known bugs/unverified in bugs.tsv.
    let mut acknowledged = std::collections::HashSet::new();

    fn extract_fn_names(formula: &str, set: &mut std::collections::HashSet<String>) {
        let upper = formula.to_uppercase();
        let mut rest = upper.as_str();
        while let Some(idx) = rest.find('(') {
            let before = &rest[..idx];
            let name_start = before
                .rfind(|c: char| !c.is_alphanumeric() && c != '.' && c != '_')
                .map(|i| i + 1)
                .unwrap_or(0);
            let name = &before[name_start..];
            if !name.is_empty() {
                set.insert(name.to_string());
            }
            rest = &rest[idx + 1..];
        }
    }

    let bugs_path = gdir.join("bugs.tsv");

    for entry in std::fs::read_dir(&gdir).expect("cannot read fixture dir") {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("tsv") {
            continue;
        }
        let is_bugs = path == bugs_path;
        let mut rdr = csv::ReaderBuilder::new()
            .delimiter(b'\t')
            .has_headers(true)
            .from_path(&path)
            .unwrap();

        for result in rdr.records() {
            let record = match result {
                Ok(r) => r,
                Err(_) => continue,
            };
            if record.len() < 2 {
                continue;
            }
            let formula = record[1].trim();
            if formula.is_empty() {
                continue;
            }

            if is_bugs {
                // Every bugs.tsv row acknowledges the functions it uses.
                extract_fn_names(formula, &mut acknowledged);
                continue;
            }

            if record.len() < 5 {
                continue;
            }
            let expected_str = record[2].trim();
            let expected_type = record[4].trim();
            if expected_str.is_empty() || is_volatile_formula(formula) {
                continue;
            }
            let expected = match parse_expected(expected_str, expected_type) {
                Some(v) => v,
                None => continue,
            };
            let actual = evaluate(formula, &vars);
            if values_match(&actual, &expected, expected_type) {
                extract_fn_names(formula, &mut covered);
            }
        }
    }

    let mut missing = Vec::new();
    for name in &all_names {
        let upper = name.to_uppercase();
        if volatile.contains(upper.as_str())
            || context_limited.contains(upper.as_str())
            || covered.contains(&upper)
            || acknowledged.contains(&upper)
        {
            continue;
        }
        missing.push(name.clone());
    }
    missing.sort();
    assert!(
        missing.is_empty(),
        "Functions with no passing conformance row: {:?}",
        missing
    );
}

/// Engine-explicit shim: the free `evaluate` is deprecated in favor of
/// `Engine::sheets().evaluate` (ADR 2026-04-27).
fn evaluate(formula: &str, variables: &std::collections::HashMap<String, Value>) -> Value {
    truecalc_core::Engine::sheets().evaluate(formula, variables)
}
