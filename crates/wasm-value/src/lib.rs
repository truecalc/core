use serde::Serialize;
use tsify_next::Tsify;

use truecalc_core::types::SparklineValue;
use truecalc_core::Value;

/// The result of evaluating a formula on the WASM surface.
///
/// A discriminated union tagged by `type`. Scalars carry their value directly;
/// the `array` variant is recursive -- each element is itself an `EvalResult`,
/// so a 2-D array result is an array of `array` rows whose elements are scalar
/// `EvalResult`s. This mirrors how `truecalc-core` represents array values
/// internally (1-D arrays are flat, 2-D arrays nest row sub-arrays).
///
/// Shared by `@truecalc/core` (`crates/wasm`) and `@truecalc/workbook`
/// (`crates/wasm-workbook`) so the two WASM packages present one identical
/// tagged-value shape rather than each hand-rolling its own copy of it.
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

/// Map a single sparkline data point / option value onto the WASM surface.
/// `pub`: reused by both `crates/wasm` (for `truecalc_core::Value`'s
/// `SparklineSpec`) and `crates/wasm-workbook` (for `truecalc_workbook::Value`'s
/// `SparklineSpec`) — the two crates' sparkline specs share this identical
/// `truecalc_core::types::SparklineSpec` type, so this mapping needs exactly
/// one implementation.
pub fn sparkline_value_to_result(v: &SparklineValue) -> EvalResult {
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
