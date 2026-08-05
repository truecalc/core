use std::collections::HashMap;

use serde::Serialize;
use tsify_next::Tsify;
use wasm_bindgen::prelude::*;

use truecalc_core::types::zoned::parse_rfc9557;
use truecalc_core::types::{SparklineChartType, SparklineSpec, SparklineValue};
use truecalc_core::{Registry, Value};

/// One sparkline data point / option value, read back from the shape
/// [`value_to_result`] emits for it.
fn json_to_sparkline_value(v: &serde_json::Value) -> Option<SparklineValue> {
    let obj = v.as_object()?;
    match obj.get("type")?.as_str()? {
        "number" => Some(SparklineValue::number(obj.get("value")?.as_f64()?)),
        "text" => Some(SparklineValue::Text(obj.get("value")?.as_str()?.to_owned())),
        "bool" => Some(SparklineValue::Bool(obj.get("value")?.as_bool()?)),
        "empty" => Some(SparklineValue::Blank),
        _ => None,
    }
}

/// Read a `{ type: "sparkline", value: SparklineSpecResult }` object back into a
/// spec, so an emitted sparkline can be fed back in as a variable unchanged.
fn json_to_sparkline(spec: &serde_json::Value) -> Option<SparklineSpec> {
    let obj = spec.as_object()?;
    let chart_type = SparklineChartType::parse(obj.get("charttype")?.as_str()?)?;
    let raw_data = obj.get("data")?.as_array()?;
    // The evaluator answers `#N/A` for a `data` argument with fewer than two
    // points, so a shorter spec is not something it can emit — reject it here
    // too, exactly as the workbook decoder does.
    if raw_data.len() < 2 {
        return None;
    }
    let mut data = Vec::new();
    for raw in raw_data {
        data.push(json_to_sparkline_value(raw)?);
    }
    let mut options = Vec::new();
    for raw in obj.get("options")?.as_array()? {
        let pair = raw.as_array()?;
        if pair.len() != 2 {
            return None;
        }
        let key = pair[0].as_str()?.to_ascii_lowercase();
        // `charttype` is lifted into the spec's own field, never left in the
        // option list — so a payload carrying it there was not emitted by us.
        if key == "charttype" {
            return None;
        }
        options.push((key, json_to_sparkline_value(&pair[1])?));
    }
    Some(SparklineSpec {
        chart_type,
        data,
        options,
    })
}

/// Convert a JSON value (from JS) into a truecalc-core Value.
///
/// A zoned instant round-trips in via the self-describing object
/// `{ "type": "zoned", "value": "<RFC-9557>" }` (the same shape `value_to_result`
/// emits), so an emitted `Zoned` can be fed back as a variable. A sparkline
/// round-trips the same way, through `{ "type": "sparkline", "value": {...} }`:
/// without it an emitted sparkline would silently read back as `empty`, and
/// `TYPE(x)` would answer 1 instead of 128.
pub fn json_to_value(v: &serde_json::Value) -> Value {
    match v {
        serde_json::Value::Number(n) => n
            .as_f64()
            .map(Value::Number)
            .unwrap_or(Value::Error(truecalc_core::ErrorKind::Num)),
        serde_json::Value::String(s) => Value::Text(s.clone()),
        serde_json::Value::Bool(b) => Value::Bool(*b),
        serde_json::Value::Null => Value::Empty,
        serde_json::Value::Object(map) => {
            if map.get("type").and_then(|t| t.as_str()) == Some("zoned") {
                if let Some(zi) = map
                    .get("value")
                    .and_then(|val| val.as_str())
                    .and_then(parse_rfc9557)
                {
                    return Value::Zoned(Box::new(zi));
                }
            }
            if map.get("type").and_then(|t| t.as_str()) == Some("sparkline") {
                if let Some(spec) = map.get("value").and_then(json_to_sparkline) {
                    return Value::Sparkline(Box::new(spec));
                }
            }
            Value::Empty
        }
        _ => Value::Empty,
    }
}

/// The result of evaluating a formula on the WASM surface.
///
/// A discriminated union tagged by `type`. Scalars carry their value directly;
/// the `array` variant is recursive -- each element is itself an `EvalResult`,
/// so a 2-D array result is an array of `array` rows whose elements are scalar
/// `EvalResult`s. This mirrors how `truecalc-core` represents array values
/// internally (1-D arrays are flat, 2-D arrays nest row sub-arrays) and matches
/// the recursive shape `truecalc-mcp` already emits.
///
/// # Surface shape (npm `@truecalc/core` >= 0.7.0)
///
/// - `{ type: "number", value: 1.5 }`
/// - `{ type: "text", value: "yes" }`
/// - `{ type: "bool", value: true }`
/// - `{ type: "empty" }`
/// - `{ type: "error", error: "#REF!" }` -- and, when a diagnostic is available,
///   `{ type: "error", error: "#N/A", message: "Wrong number of arguments to
///   DATE. Expected 3 arguments, but got 0 arguments." }`. `message` is additive and
///   omitted for errors without a diagnostic, so existing consumers are unaffected.
/// - `{ type: "date", value: 46180 }` -- spreadsheet serial number (epoch implied
///   by the engine flavor; `sheets` day 0 = 1899-12-30)
/// - `{ type: "array", value: [ EvalResult, ... ] }` -- recursive; a 2-D result is
///   `{ type: "array", value: [ { type: "array", value: [ <cells> ] }, ... ] }`
#[derive(Tsify, Serialize, Debug)]
#[tsify(into_wasm_abi)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum EvalResult {
    Number { value: f64 },
    Text { value: String },
    Bool { value: bool },
    /// A spreadsheet serial date number. Distinct from `Number` so consumers can
    /// format it as a date; the epoch is implied by the engine flavor.
    Date { value: f64 },
    /// A zone-aware instant, carried as its canonical, self-describing RFC-9557
    /// string, e.g. `2026-07-14T11:00:00+02:00[Europe/Berlin]`. Derived fields
    /// (offset/abbrev/is_dst) are intentionally NOT emitted — fetch them via the
    /// `TZ*` functions so consumers never persist a stale offset.
    Zoned { value: String },
    Error {
        error: String,
        /// Optional human-readable diagnostic (Google Sheets parity), e.g. the
        /// arity message for `DATE()`. Absent for errors without a diagnostic.
        #[tsify(optional)]
        #[serde(skip_serializing_if = "Option::is_none")]
        message: Option<String>,
    },
    Empty,
    /// An (unspilled) array result. Recursive: 2-D arrays are arrays of `array`
    /// rows. Cells carry their own type, including nested `date`/`error`/`empty`.
    Array { value: Vec<EvalResult> },
    /// A sparkline: the parsed, validated render spec produced by `SPARKLINE`.
    /// Google Sheets models this as a value kind of its own (`TYPE()` reports
    /// the undocumented code `128`), and the spec is the value's identity, so
    /// it is carried in full rather than projected to text.
    Sparkline { value: SparklineSpecResult },
}

/// A parsed sparkline render spec on the WASM surface. `data` points and
/// option values are ordinary [`EvalResult`] cells (a blank cell inside the
/// source range is `empty`).
#[derive(Tsify, Serialize, Debug)]
pub struct SparklineSpecResult {
    /// `line` (the default), `bar`, `column` or `winloss`.
    pub charttype: String,
    /// The points to plot, row-major.
    pub data: Vec<EvalResult>,
    /// The remaining option key/value pairs, in the order given, keys
    /// lower-cased. Keys the engine does not recognise are kept, not rejected:
    /// Sheets ignores an unknown option key rather than erroring.
    pub options: Vec<(String, EvalResult)>,
}

fn sparkline_value_to_result(v: &SparklineValue) -> EvalResult {
    match v {
        SparklineValue::Number(n) => EvalResult::Number { value: *n },
        SparklineValue::Text(s) => EvalResult::Text { value: s.clone() },
        SparklineValue::Bool(b) => EvalResult::Bool { value: *b },
        SparklineValue::Blank => EvalResult::Empty,
    }
}

/// Map a `truecalc-core` `Value` onto the WASM `EvalResult` surface shape.
///
/// Recurses into arrays so every cell carries its own type; `Date` is preserved
/// as a distinct `date` result rather than collapsed to `number`.
pub fn value_to_result(value: Value) -> EvalResult {
    match value {
        Value::Number(n) => EvalResult::Number { value: n },
        Value::Date(n) => EvalResult::Date { value: n },
        Value::Zoned(z) => EvalResult::Zoned { value: z.to_rfc9557() },
        Value::Text(s) => EvalResult::Text { value: s },
        Value::Bool(b) => EvalResult::Bool { value: b },
        Value::Error(e) => EvalResult::Error { error: e.to_string(), message: None },
        Value::ErrorMsg(e, m) => EvalResult::Error { error: e.to_string(), message: Some(m) },
        Value::Empty => EvalResult::Empty,
        Value::Array(items) => EvalResult::Array {
            value: items.into_iter().map(value_to_result).collect(),
        },
        Value::Sparkline(spec) => EvalResult::Sparkline {
            value: SparklineSpecResult {
                charttype: spec.chart_type.as_str().to_string(),
                data: spec.data.iter().map(sparkline_value_to_result).collect(),
                options: spec
                    .options
                    .iter()
                    .map(|(k, v)| (k.clone(), sparkline_value_to_result(v)))
                    .collect(),
            },
        },
    }
}

#[derive(Tsify, Serialize)]
#[tsify(into_wasm_abi)]
pub struct ValidateResult {
    pub valid: bool,
    #[tsify(optional)]
    pub error: Option<String>,
}

#[derive(Tsify, Serialize)]
#[tsify(into_wasm_abi)]
pub struct FunctionInfo {
    pub name: String,
    pub category: String,
    pub syntax: String,
    pub description: String,
}

/// Evaluate a formula with named variables supplied as a JS object.
///
/// `variables` must be a plain JS object mapping string keys to number/string/bool/null.
/// Passing `undefined` or `null` is safe and is treated as no variables.
#[wasm_bindgen]
pub fn evaluate(formula: &str, variables: JsValue) -> EvalResult {
    let vars_json: serde_json::Value = serde_wasm_bindgen::from_value(variables)
        .unwrap_or(serde_json::Value::Object(Default::default()));

    let vars: HashMap<String, Value> = match vars_json.as_object() {
        Some(map) => map
            .iter()
            .map(|(k, v)| (k.clone(), json_to_value(v)))
            .collect(),
        None => HashMap::new(),
    };

    value_to_result(truecalc_core::Engine::sheets().evaluate(formula, &vars))
}

/// Validate a formula string without evaluating it.
///
/// Returns `{ valid: true }` on success or `{ valid: false, error: "..." }` on failure.
#[wasm_bindgen]
pub fn validate(formula: &str) -> ValidateResult {
    match truecalc_core::Engine::sheets().validate(formula) {
        Ok(_) => ValidateResult { valid: true, error: None },
        Err(e) => ValidateResult { valid: false, error: Some(e.to_string()) },
    }
}

#[derive(Tsify, Serialize)]
#[tsify(into_wasm_abi)]
pub struct TranslateResult {
    #[tsify(optional)]
    pub formula: Option<String>,
    #[tsify(optional)]
    pub error: Option<String>,
}

/// Shift every relative cell/range reference in `formula` by `(d_row, d_col)`
/// — the fill / copy-paste reference-adjustment transform. `$`-absolute axes
/// are left unchanged; an out-of-bounds corner becomes literal `#REF!`.
///
/// Sheets flavor only (issue #709 v1); Excel support is a follow-up.
#[wasm_bindgen]
pub fn translate_formula(formula: &str, d_row: i32, d_col: i32) -> TranslateResult {
    match truecalc_core::Engine::sheets().translate_formula(formula, d_row as i64, d_col as i64) {
        Ok(f) => TranslateResult { formula: Some(f), error: None },
        Err(e) => TranslateResult { formula: None, error: Some(e.to_string()) },
    }
}

#[derive(Tsify, Serialize)]
#[tsify(into_wasm_abi)]
pub struct RenameSheetRefsResult {
    #[tsify(optional)]
    pub formula: Option<String>,
    #[tsify(optional)]
    pub error: Option<String>,
}

/// Rewrite the sheet qualifier of every cell/range reference in `formula`
/// that points at `old` to point at `new` instead — the sheet-rename
/// reference-rewrite transform. Sheet-name matching is case-insensitive.
/// Requoting is handled automatically. Unqualified refs, refs to other
/// sheets, string literals, function names, and defined names are left
/// untouched; no-op if `formula` has no `old`-qualified refs.
#[wasm_bindgen]
pub fn rename_sheet_refs(formula: &str, old: &str, new: &str) -> RenameSheetRefsResult {
    match truecalc_core::Engine::sheets().rename_sheet_refs(formula, old, new) {
        Ok(f) => RenameSheetRefsResult { formula: Some(f), error: None },
        Err(e) => RenameSheetRefsResult { formula: None, error: Some(e.to_string()) },
    }
}

/// Return metadata for all built-in functions as a JS array.
///
/// Each entry: `{ name, category, syntax, description }`.
#[wasm_bindgen]
pub fn list_functions() -> Vec<FunctionInfo> {
    let registry = Registry::new();
    let mut out: Vec<FunctionInfo> = registry
        .get_metadata()
        .into_iter()
        .map(|entry| FunctionInfo {
            name: entry.name.to_string(),
            category: entry.meta.category.to_string(),
            // The registry calls this `signature`; the JS surface has always
            // called it `syntax`. Kept as `syntax` deliberately — renaming it
            // is a breaking change for every existing npm consumer, so it is
            // left for a major version rather than smuggled in with a bug fix.
            syntax: entry.meta.signature.to_string(),
            description: entry.meta.description.to_string(),
        })
        .collect();
    // Stable order: the registry is a HashMap, so iteration order varies run to
    // run. Without this the same build returns a different array order each
    // call, which breaks snapshot tests and any consumer rendering a list.
    out.sort_by(|a, b| a.name.cmp(&b.name));
    out
}

/// A stateful engine bound to a conformance target.
///
/// Obtained via `createEngine('google-sheets')`.
#[wasm_bindgen(js_name = "Engine")]
pub struct WasmEngine {
    inner: truecalc_core::Engine,
}

#[wasm_bindgen]
impl WasmEngine {
    /// Evaluate a formula using this engine's conformance target.
    pub fn evaluate(&self, formula: &str, variables: JsValue) -> EvalResult {
        let vars_json: serde_json::Value = serde_wasm_bindgen::from_value(variables)
            .unwrap_or(serde_json::Value::Object(Default::default()));

        let vars: HashMap<String, Value> = match vars_json.as_object() {
            Some(map) => map
                .iter()
                .map(|(k, v)| (k.clone(), json_to_value(v)))
                .collect(),
            None => HashMap::new(),
        };

        value_to_result(self.inner.evaluate(formula, &vars))
    }
}

/// Create an engine for a specific conformance target.
///
/// Supported targets: `"google-sheets"`.
/// Returns an error for unknown targets.
#[wasm_bindgen(js_name = "createEngine")]
pub fn create_engine(target: &str) -> Result<WasmEngine, JsValue> {
    match target {
        "google-sheets" => Ok(WasmEngine { inner: truecalc_core::Engine::sheets() }),
        _ => Err(JsValue::from_str(&format!("Unknown conformance target: '{}'", target))),
    }
}
