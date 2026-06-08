//! Workbook-level recalc conformance (P3.3 / plan P3.6) against the fixtures
//! pipeline's ground truth.
//!
//! This drives a **real** [`Workbook`] — real sparse grid, real dependency
//! graph, real named ranges, real grid-backed resolver — through
//! [`Workbook::recalc`] and compares the recomputed probe cell to the
//! pipeline-recorded expected value of each `workbook.tsv` row whose authored
//! inputs the `workbook.inputs.json` sidecar models (cross-sheet refs and
//! named ranges). It is the workbook-layer counterpart of core's
//! `workbook_inputs_conformance` (which exercises only the bare
//! `Resolver`/`evaluate_with_resolver` seam); here the entire P3.3 stack is in
//! the loop.
//!
//! Both the inputs and the expected values come from the pipeline — nothing is
//! hand-authored here. The fixtures live in the `truecalc-core` crate (the
//! pipeline commits them there, fixtures extension #2 / #532); this test reads
//! them from there rather than duplicating immutable ground truth.
//!
//! Volatile rows are pinned via a [`RecalcContext`] whose `now_serial` matches
//! the fixture's recorded `evaluatedAt` under the pinned `Etc/GMT` timezone
//! (the same pin core's conformance runner uses).

use serde_json::Value as Json;
use std::path::{Path, PathBuf};
use truecalc_workbook::{
    Address, CellInput, EngineFlavor, RecalcContext, Value, Workbook, Worksheet,
};

/// The pipeline committed `workbook.tsv` / `workbook.inputs.json` into the core
/// crate's fixture tree; read them from there (no duplicate ground truth).
fn core_fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../core/tests/fixtures/google_sheets")
        .join(name)
}

/// The fixture's evaluation instant: `workbook.tsv` records
/// `evaluatedAt = 2026-06-07T23:50:56.808Z` under `Etc/GMT`. The same pin core's
/// `pinned_now_serial` derives (serial 46180.993713…).
fn pinned_ctx() -> RecalcContext {
    RecalcContext::new(1_780_876_256_808, "Etc/GMT", 0).expect("Etc/GMT is valid")
}

fn json_to_input(j: &Json) -> CellInput {
    match j {
        Json::String(s) if s.starts_with('=') => CellInput::Formula(s.clone()),
        Json::String(s) => CellInput::Literal(Value::Text(s.clone())),
        Json::Number(n) => CellInput::Literal(Value::Number(n.as_f64().unwrap_or(0.0))),
        Json::Bool(b) => CellInput::Literal(Value::Boolean(*b)),
        _ => CellInput::Literal(Value::Text(String::new())),
    }
}

/// Build a real workbook from one sidecar scenario (sheets + named ranges).
fn build_scenario(scenario: &Json) -> Workbook {
    let mut wb = Workbook::new(EngineFlavor::Sheets);
    let sheets = scenario.get("sheets").and_then(Json::as_object).unwrap();
    for (sheet_name, cells) in sheets {
        wb.add_sheet(Worksheet::new(sheet_name.clone())).unwrap();
        for (a1, raw) in cells.as_object().unwrap() {
            let addr = Address::from_a1(a1).unwrap();
            wb.set(sheet_name, addr, json_to_input(raw)).unwrap();
        }
    }
    if let Some(names) = scenario.get("names").and_then(Json::as_object) {
        for (name, refstr) in names {
            wb.define_name(name, refstr.as_str().unwrap()).unwrap();
        }
    }
    wb
}

fn value_matches(actual: &Value, expected_s: &str, ty: &str) -> bool {
    match ty {
        "number" => matches!(actual,
            Value::Number(n) | Value::Date(n)
            if (n - expected_s.parse::<f64>().unwrap_or(f64::NAN)).abs()
                <= 1e-4 * expected_s.parse::<f64>().unwrap_or(1.0).abs().max(1.0)),
        "date" => matches!(actual,
            Value::Date(n) | Value::Number(n)
            if (n - expected_s.parse::<f64>().unwrap_or(f64::NAN)).abs() <= 1e-4),
        "boolean" => {
            matches!(actual, Value::Boolean(b) if *b == expected_s.eq_ignore_ascii_case("true"))
        }
        "error" => matches!(actual, Value::Error(code) if code == expected_s),
        _ => match actual {
            Value::Text(s) => s == expected_s,
            Value::Empty => expected_s.is_empty(),
            _ => false,
        },
    }
}

/// Route a `workbook.tsv` row to the scenario whose inputs it reads, mirroring
/// core's `scenario_for_row`. Rows needing no sidecar inputs (date-type) are
/// skipped here (covered by other suites).
fn scenario_for(formula: &str, names: &[&str]) -> Option<&'static str> {
    if names.iter().any(|n| formula.contains(n))
        || formula.contains("NOT_A_DEFINED_NAME")
        || formula.contains("tax_rate")
    {
        return Some("named_range");
    }
    if formula.contains('!') {
        return Some("cross_sheet_ref");
    }
    None
}

/// Rows excluded for known pre-existing engine gaps unrelated to recalc, kept in
/// lockstep with core's `workbook_inputs_conformance::OUT_OF_SCOPE_FORMULAS`:
///
/// - `=COUNT(Data!A1:D3)` — COUNT's range path counts a boolean cell the
///   pipeline observed Sheets skipping (engine bug #584).
/// - `=ROWS(PRICES)` / `=COLUMNS(GRID)` — shape functions need a 2-D range; the
///   P1.3 [`Resolver`] contract materializes ranges **flat** (reading order),
///   which is the shape the aggregations consume and is all core's evaluator
///   accepts, so dimensions are not recoverable. Shaped-range support is a core
///   concern, not a recalc one; excluded here exactly as core excludes them.
const ENGINE_GAP_FORMULAS: &[&str] = &["=COUNT(Data!A1:D3)", "=ROWS(PRICES)", "=COLUMNS(GRID)"];

#[test]
fn workbook_recalc_matches_pipeline_for_seeded_rows() {
    let inputs_raw = std::fs::read_to_string(core_fixture("workbook.inputs.json"))
        .expect("workbook.inputs.json (committed by the fixtures pipeline) must exist");
    let inputs: Json = serde_json::from_str(&inputs_raw).unwrap();
    let scenarios = inputs.get("scenarios").and_then(Json::as_object).unwrap();

    let names: Vec<&str> = scenarios
        .get("named_range")
        .and_then(|s| s.get("names"))
        .and_then(Json::as_object)
        .map(|m| m.keys().map(String::as_str).collect())
        .unwrap_or_default();

    let tsv =
        std::fs::read_to_string(core_fixture("workbook.tsv")).expect("workbook.tsv must exist");
    let ctx = pinned_ctx();

    let mut checked = 0usize;
    let mut failures: Vec<String> = Vec::new();

    for line in tsv.lines().skip(1) {
        if line.trim().is_empty() {
            continue;
        }
        let cols: Vec<&str> = line.split('\t').collect();
        if cols.len() < 5 {
            continue;
        }
        let (desc, formula, expected_s, _cat, ty) = (cols[0], cols[1], cols[2], cols[3], cols[4]);
        if ENGINE_GAP_FORMULAS.contains(&formula) {
            continue;
        }
        let Some(scenario_name) = scenario_for(formula, &names) else {
            continue;
        };

        // Build a fresh workbook for the scenario and add the probe formula in
        // a free cell of a probe sheet, so a bare-ref row resolves against its
        // own (empty) sheet exactly as Sheets would, and qualified refs reach
        // the seeded sheets.
        let mut wb = build_scenario(&scenarios[scenario_name]);
        wb.add_sheet(Worksheet::new("Probe")).unwrap();
        let probe = Address::from_a1("Z99").unwrap();
        wb.set("Probe", probe, CellInput::Formula(formula.to_string()))
            .unwrap();

        wb.recalc(&ctx);
        let actual = wb.get("Probe", probe).unwrap().value().clone();

        checked += 1;
        if !value_matches(&actual, expected_s, ty) {
            failures.push(format!(
                "[{scenario_name}] {desc}: {formula} => {actual:?}, expected {expected_s:?} ({ty})"
            ));
        }
    }

    assert!(
        checked > 0,
        "no seeded workbook rows exercised — routing broken"
    );
    assert!(
        failures.is_empty(),
        "{} of {} recalc conformance rows failed:\n{}",
        failures.len(),
        checked,
        failures.join("\n")
    );
    eprintln!("workbook_recalc_matches_pipeline_for_seeded_rows: {checked} rows passed");
}
