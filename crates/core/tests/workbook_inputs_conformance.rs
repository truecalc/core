//! Workbook conformance — full blocking coverage of `workbook.tsv` (core#575).
//!
//! `workbook.tsv` rows fall into three kinds:
//!
//! 1. **Cross-sheet / named-range rows** — formulas like `=Data!A1` and
//!    `=SUM(PRICES)` read authored input cells and named ranges.  A
//!    [`ScenarioResolver`] is seeded from the fixtures pipeline's input-model
//!    sidecar (`workbook.inputs.json`, fixtures extension #2 / core#532) and
//!    each row is evaluated through [`Engine::evaluate_with_resolver_at`].
//!
//! 2. **Date-type / plain rows** — formulas like `=DATE(2026,6,7)` and
//!    `=TODAY()` need no external inputs; they are evaluated via
//!    [`Engine::evaluate_at`] with a pinned `now` serial so volatile functions
//!    are deterministic.
//!
//! 3. **Out-of-scope rows** — `=ROWS(PRICES)` and `=COLUMNS(GRID)` require a
//!    2-D shaped range that this conformance shim does not synthesize (its flat
//!    array resolver can't satisfy ROWS/COLUMNS); they are excluded.  Moving
//!    them to `bugs.tsv` is avoided because they are not engine bugs — they
//!    are a shim limitation.
//!
//! All rows except the out-of-scope ones are now exercised as a **blocking**
//! assertion; CI fails if any row regresses.  No expected value is hand-authored
//! here — inputs and expecteds come from the pipeline.

use serde_json::Value as Json;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use truecalc_core::{CellAddr, Engine, ErrorKind, Ref, Resolver, Value};

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/google_sheets")
        .join(name)
}

/// Volatile-row pin for `workbook.tsv` — mirrors `conformance.rs::pinned_now_serial`.
fn pinned_now_serial() -> f64 {
    46180.0 + (23.0 * 3600.0 + 50.0 * 60.0 + 56.808) / 86400.0
}

/// One authored scenario from the sidecar: sheets (A1 -> raw input) + names.
struct Scenario {
    /// sheet name -> (A1 -> raw input JSON: number/bool/string, "=..." = formula)
    sheets: HashMap<String, HashMap<String, Json>>,
    /// name -> sheet-qualified ref string, e.g. "Data!B1:B3"
    names: HashMap<String, String>,
}

/// A resolver seeded from one scenario.  Cell formulas are pre-resolved to a
/// fixpoint up front so `resolve` is a pure lookup (no reentrancy into the
/// evaluator).  Engine flavor is `sheets` (the sidecar's only engine).
struct ScenarioResolver {
    /// (lowercased sheet, col, row) -> resolved Value
    resolved: HashMap<(String, u32, u32), Value>,
    /// set of real sheet names (lowercased) so missing-sheet -> #REF!
    sheets: Vec<String>,
    /// name -> sheet-qualified ref string
    names: HashMap<String, String>,
}

impl ScenarioResolver {
    fn new(scenario: &Scenario) -> Self {
        let engine = Engine::sheets();
        let mut resolved: HashMap<(String, u32, u32), Value> = HashMap::new();
        let sheets: Vec<String> = scenario.sheets.keys().map(|s| s.to_lowercase()).collect();

        // Seed literals; collect (sheet, addr, formula) for fixpoint passes.
        let mut formulas: Vec<(String, CellAddr, String)> = Vec::new();
        for (sheet, cells) in &scenario.sheets {
            for (a1, raw) in cells {
                let addr = CellAddr::parse(a1).expect("sidecar A1 address must parse");
                match raw {
                    Json::String(s) if s.starts_with('=') => {
                        formulas.push((sheet.to_lowercase(), addr, s.clone()));
                    }
                    other => {
                        resolved.insert(
                            (sheet.to_lowercase(), addr.col, addr.row),
                            json_to_value(other),
                        );
                    }
                }
            }
        }

        let names = scenario.names.clone();

        // Fixpoint: a cell formula may read another formula cell (B2 = A1*2).
        // Each formula is evaluated *in the context of its own sheet*, so bare
        // refs (sheet = None) resolve against that sheet.
        let mut me = ScenarioResolver {
            resolved,
            sheets,
            names,
        };
        for _ in 0..(formulas.len() + 2) {
            let snapshot = me.resolved.clone();
            let mut next = snapshot.clone();
            let mut changed = false;
            for (sheet, addr, formula) in &formulas {
                let mut reader = SnapshotResolver {
                    resolved: &snapshot,
                    sheets: &me.sheets,
                    names: &me.names,
                    default_sheet: Some(sheet),
                };
                let v = engine.evaluate_with_resolver(formula, &mut reader);
                let key = (sheet.clone(), addr.col, addr.row);
                if next.get(&key) != Some(&v) {
                    next.insert(key, v);
                    changed = true;
                }
            }
            me.resolved = next;
            if !changed {
                break;
            }
        }
        me
    }
}

/// Read-only resolver over a fixed snapshot map.
struct SnapshotResolver<'a> {
    resolved: &'a HashMap<(String, u32, u32), Value>,
    sheets: &'a [String],
    names: &'a HashMap<String, String>,
    /// Sheet a bare (unqualified) ref resolves against — the formula's own sheet
    /// during fixpoint; `None` for top-level workbook.tsv rows (all qualified).
    default_sheet: Option<&'a str>,
}

impl<'a> SnapshotResolver<'a> {
    fn sheet_exists(&self, sheet: &str) -> bool {
        let lc = sheet.to_lowercase();
        self.sheets.contains(&lc)
    }

    fn cell(&self, sheet: &str, addr: &CellAddr) -> Value {
        self.resolved
            .get(&(sheet.to_lowercase(), addr.col, addr.row))
            .cloned()
            .unwrap_or(Value::Empty)
    }

    /// Materialize a range as a row-major flat array.  The engine's range
    /// aggregations (SUM/AVERAGE/COUNT/SUMIF) flatten one level, so a flat
    /// array is the shape they consume; shape-aware functions (ROWS/COLUMNS)
    /// need a 2-D array the current resolver shim does not synthesize, so those
    /// rows are out of scope here (see `SHAPE_DEPENDENT_ROWS`).
    fn range(&self, sheet: &str, start: &CellAddr, end: &CellAddr) -> Value {
        let mut cells = Vec::new();
        for r in start.row..=end.row {
            for c in start.col..=end.col {
                cells.push(self.cell(sheet, &CellAddr { col: c, row: r }));
            }
        }
        Value::Array(cells)
    }

    fn resolve_ref(&self, r: &Ref) -> Value {
        match r {
            Ref::Cell { sheet, addr } => {
                let s = sheet.as_deref().or(self.default_sheet);
                match s {
                    Some(s) if self.sheet_exists(s) => self.cell(s, addr),
                    Some(_) => Value::Error(ErrorKind::Ref),
                    None => Value::Empty,
                }
            }
            Ref::Range { sheet, start, end } => {
                let s = sheet.as_deref().or(self.default_sheet);
                match s {
                    Some(s) if self.sheet_exists(s) => self.range(s, start, end),
                    Some(_) => Value::Error(ErrorKind::Ref),
                    None => Value::Empty,
                }
            }
            Ref::Name(name) => match self
                .names
                .iter()
                .find(|(k, _)| k.eq_ignore_ascii_case(name))
                .map(|(_, v)| v)
            {
                Some(refstr) => match parse_qualified_ref(refstr) {
                    Some(parsed) => self.resolve_ref(&parsed),
                    None => Value::Error(ErrorKind::Ref),
                },
                None => Value::Error(ErrorKind::Name),
            },
        }
    }
}

impl<'a> Resolver for SnapshotResolver<'a> {
    fn resolve(&mut self, r: &Ref) -> Value {
        self.resolve_ref(r)
    }
}

impl Resolver for ScenarioResolver {
    fn resolve(&mut self, r: &Ref) -> Value {
        let reader = SnapshotResolver {
            resolved: &self.resolved,
            sheets: &self.sheets,
            names: &self.names,
            default_sheet: None,
        };
        reader.resolve_ref(r)
    }
}

/// Parse `Data!B1:B3` / `'Quoted Name'!A1` into a [`Ref`].
fn parse_qualified_ref(s: &str) -> Option<Ref> {
    let (sheet, rest) = split_sheet(s)?;
    if let Some((a, b)) = rest.split_once(':') {
        Some(Ref::Range {
            sheet: Some(sheet),
            start: CellAddr::parse(a)?,
            end: CellAddr::parse(b)?,
        })
    } else {
        Some(Ref::Cell {
            sheet: Some(sheet),
            addr: CellAddr::parse(rest)?,
        })
    }
}

/// Split `Sheet!A1` / `'Q Name'!A1` into (unquoted sheet name, cell/range part).
fn split_sheet(s: &str) -> Option<(String, &str)> {
    if let Some(stripped) = s.strip_prefix('\'') {
        let bytes = stripped.as_bytes();
        let mut name = String::new();
        let mut i = 0;
        while i < bytes.len() {
            if bytes[i] == b'\'' {
                if i + 1 < bytes.len() && bytes[i + 1] == b'\'' {
                    name.push('\'');
                    i += 2;
                    continue;
                }
                let rest = stripped[i + 1..].strip_prefix('!')?;
                return Some((name, rest));
            }
            name.push(bytes[i] as char);
            i += 1;
        }
        None
    } else {
        let (sheet, rest) = s.split_once('!')?;
        Some((sheet.to_string(), rest))
    }
}

fn json_to_value(j: &Json) -> Value {
    match j {
        Json::Number(n) => Value::Number(n.as_f64().unwrap_or(0.0)),
        Json::Bool(b) => Value::Bool(*b),
        Json::String(s) => Value::Text(s.clone()),
        _ => Value::Empty,
    }
}

fn load_scenarios() -> HashMap<String, Scenario> {
    let raw = std::fs::read_to_string(fixture("workbook.inputs.json"))
        .expect("workbook.inputs.json must exist (fixtures extension #2)");
    let doc: Json = serde_json::from_str(&raw).expect("workbook.inputs.json must be valid JSON");
    let scenarios = doc
        .get("scenarios")
        .and_then(|s| s.as_object())
        .expect("sidecar must have a `scenarios` object");

    let mut out = HashMap::new();
    for (name, body) in scenarios {
        let sheets_json = body
            .get("sheets")
            .and_then(|s| s.as_object())
            .expect("scenario must have `sheets`");
        let mut sheets = HashMap::new();
        for (sheet, cells) in sheets_json {
            let cell_map = cells
                .as_object()
                .expect("sheet cells must be an object")
                .iter()
                .map(|(a1, v)| (a1.clone(), v.clone()))
                .collect();
            sheets.insert(sheet.clone(), cell_map);
        }
        let names = body
            .get("names")
            .and_then(|n| n.as_object())
            .map(|m| {
                m.iter()
                    .map(|(k, v)| (k.clone(), v.as_str().unwrap_or_default().to_string()))
                    .collect()
            })
            .unwrap_or_default();
        out.insert(name.clone(), Scenario { sheets, names });
    }
    out
}

fn parse_expected(value: &str, ty: &str) -> Value {
    match ty {
        "number" => Value::Number(value.parse().unwrap_or(f64::NAN)),
        "boolean" => Value::Bool(value.eq_ignore_ascii_case("true")),
        "date" => Value::Date(value.parse().unwrap_or(f64::NAN)),
        "error" => Value::Error(match value {
            "#REF!" => ErrorKind::Ref,
            "#NAME?" => ErrorKind::Name,
            "#VALUE!" => ErrorKind::Value,
            "#NUM!" => ErrorKind::Num,
            "#DIV/0!" => ErrorKind::DivByZero,
            "#N/A" => ErrorKind::NA,
            "#NULL!" => ErrorKind::Null,
            _ => ErrorKind::Value,
        }),
        _ => Value::Text(value.to_string()),
    }
}

fn values_match(actual: &Value, expected: &Value) -> bool {
    match (actual, expected) {
        (Value::Number(a), Value::Number(b)) | (Value::Date(a), Value::Number(b)) => {
            (a - b).abs() <= 1e-4 * b.abs().max(1.0)
        }
        (Value::Date(a), Value::Date(b)) => (a - b).abs() <= 1e-4 * b.abs().max(1.0),
        (Value::Bool(a), Value::Bool(b)) => a == b,
        (Value::Text(a), Value::Text(b)) => a == b,
        (Value::Error(a), Value::Error(b)) => a == b,
        (Value::Empty, Value::Text(s)) => s.is_empty(),
        (Value::Array(items), exp) if !items.is_empty() => values_match(&items[0], exp),
        _ => false,
    }
}

/// Rows out of scope for this seeded runner. Two reasons:
///
/// - shape-dependent: `ROWS`/`COLUMNS` need a 2-D range the flat resolver shim
///   does not synthesize (the real P1.3/P3.x resolver returns shaped ranges;
///   this conformance shim deliberately stays minimal).
///
/// Excluding these keeps the blocking assertion honest: it covers only rows the
/// engine + sidecar fully support, and never masks a regression in them.
const OUT_OF_SCOPE_FORMULAS: &[&str] = &["=ROWS(PRICES)", "=COLUMNS(GRID)"];

/// Route a workbook.tsv row to the scenario whose inputs it reads, or None
/// (out of scope — e.g. date-type rows need no inputs).
fn scenario_for_row(formula: &str, scenarios: &HashMap<String, Scenario>) -> Option<&'static str> {
    if let Some(n) = scenarios.get("named_range") {
        if n.names.keys().any(|name| formula.contains(name))
            || formula.contains("NOT_A_DEFINED_NAME")
            || formula.contains("tax_rate")
        {
            return Some("named_range");
        }
    }
    if scenarios.contains_key("cross_sheet_ref") && formula.contains('!') {
        return Some("cross_sheet_ref");
    }
    None
}

#[test]
fn workbook_conformance() {
    let scenarios = load_scenarios();
    let engine = Engine::sheets();
    let now = pinned_now_serial();
    let empty_vars: HashMap<String, Value> = HashMap::new();

    let mut resolvers: HashMap<String, ScenarioResolver> = HashMap::new();
    for (name, sc) in &scenarios {
        resolvers.insert(name.clone(), ScenarioResolver::new(sc));
    }

    let tsv = std::fs::read_to_string(fixture("workbook.tsv")).expect("workbook.tsv must exist");
    let mut lines = tsv.lines();
    lines.next(); // header

    let mut checked = 0usize;
    let mut failures: Vec<String> = Vec::new();

    for line in lines {
        if line.trim().is_empty() {
            continue;
        }
        let cols: Vec<&str> = line.split('\t').collect();
        if cols.len() < 5 {
            continue;
        }
        let (desc, formula, expected_s, _cat, ty) = (cols[0], cols[1], cols[2], cols[3], cols[4]);

        if OUT_OF_SCOPE_FORMULAS.contains(&formula) {
            continue;
        }

        let actual = match scenario_for_row(formula, &scenarios) {
            Some(scenario_name) => {
                let resolver = resolvers.get_mut(scenario_name).unwrap();
                engine.evaluate_with_resolver_at(formula, resolver, Some(now))
            }
            None => {
                // Date-type and plain rows need no external inputs — evaluate
                // with pinned time so volatile functions (TODAY, NOW) are deterministic.
                engine.evaluate_at(formula, &empty_vars, now)
            }
        };

        let expected = parse_expected(expected_s, ty);
        checked += 1;
        if !values_match(&actual, &expected) {
            let tag = scenario_for_row(formula, &scenarios).unwrap_or("standalone");
            failures.push(format!(
                "[{tag}] {desc}: {formula} => {actual:?}, expected {expected:?} ({ty})"
            ));
        }
    }

    assert!(
        checked > 0,
        "no workbook rows were exercised — fixture/routing broken"
    );
    assert!(
        failures.is_empty(),
        "{} of {} workbook rows failed:\n{}",
        failures.len(),
        checked,
        failures.join("\n")
    );
    eprintln!("workbook_conformance: {checked} rows passed");
}
