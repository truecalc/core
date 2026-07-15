use serde::Serialize;
use tsify_next::Tsify;
use wasm_bindgen::prelude::*;

use truecalc_core::Engine;
use truecalc_workbook::{
    Address, CellInput, Change, EngineFlavor, RecalcContext, Resolved, Value, Workbook, Worksheet,
};

/// Result of [`translate_formula`]: either the rewritten `formula` text or an
/// `error` message (mutually exclusive). Mirrors the `TranslateResult` shape
/// exposed by the calc-only `@truecalc/core` WASM binding so both packages
/// present the same reference-translation surface.
#[derive(Tsify, Serialize)]
#[tsify(into_wasm_abi)]
pub struct TranslateResult {
    #[tsify(optional)]
    pub formula: Option<String>,
    #[tsify(optional)]
    pub error: Option<String>,
}

/// Shift every relative cell/range reference in `formula` by `(dRow, dCol)` —
/// the fill / copy-paste reference-adjustment transform — using the engine's
/// authoritative parser instead of a re-implemented tokenizer.
///
/// `$`-absolute axes stay fixed; range endpoints and cross-sheet refs adjust
/// while the sheet name is preserved; references inside string literals and
/// function names are never rewritten; a name bound by `LET`/`LAMBDA` is left
/// untouched. An axis that shifts out of the grid becomes a literal `#REF!`
/// for that corner.
///
/// Sheets flavor only (Excel grid bounds are a follow-up); a parse error is
/// surfaced in the `error` field.
#[wasm_bindgen(js_name = translateFormula)]
pub fn translate_formula(formula: &str, d_row: i32, d_col: i32) -> TranslateResult {
    match Engine::sheets().translate_formula(formula, d_row as i64, d_col as i64) {
        Ok(f) => TranslateResult { formula: Some(f), error: None },
        Err(e) => TranslateResult { formula: None, error: Some(e.to_string()) },
    }
}

/// Convert a `Value` to a `serde_json::Value` tagged object for WASM consumers.
fn value_to_json(v: &Value) -> serde_json::Value {
    match v {
        Value::Number(n) => serde_json::json!({"type": "number", "value": n}),
        Value::Text(s) => serde_json::json!({"type": "text", "value": s}),
        Value::Boolean(b) => serde_json::json!({"type": "bool", "value": b}),
        Value::Date(n) => serde_json::json!({"type": "date", "value": n}),
        Value::Zoned(z) => serde_json::json!({"type": "zoned", "value": z.to_rfc9557()}),
        Value::Error(code) => serde_json::json!({"type": "error", "error": code}),
        Value::Empty => serde_json::json!({"type": "empty"}),
        Value::Array(rows) => {
            let arr: Vec<Vec<serde_json::Value>> = rows
                .iter()
                .map(|row| row.iter().map(value_to_json).collect())
                .collect();
            serde_json::json!({"type": "array", "value": arr})
        }
    }
}

/// Serialize a `Change` to a `serde_json::Value`.
fn change_to_json(c: &Change) -> serde_json::Value {
    serde_json::json!({
        "sheet": c.sheet,
        "addr": c.addr.to_a1(),
        "old": value_to_json(&c.old),
        "new": value_to_json(&c.new),
    })
}

/// A spreadsheet workbook exposed to JavaScript.
///
/// Create with `new JsWorkbook("sheets")` or `new JsWorkbook("excel")`,
/// or deserialize from a JSON string with `JsWorkbook.fromJSON(s)`.
#[wasm_bindgen]
pub struct JsWorkbook {
    inner: Workbook,
}

#[wasm_bindgen]
impl JsWorkbook {
    /// Creates a new empty workbook locked to the given engine flavor.
    ///
    /// Pass `"sheets"` for Google-Sheets-compatible behavior, or any other
    /// string (e.g. `"excel"`) for Excel-compatible behavior.
    #[wasm_bindgen(constructor)]
    pub fn new(engine: &str) -> JsWorkbook {
        let flavor = if engine == "sheets" {
            EngineFlavor::Sheets
        } else {
            EngineFlavor::Excel
        };
        JsWorkbook {
            inner: Workbook::new(flavor),
        }
    }

    /// Deserializes a workbook from its canonical JSON string.
    #[wasm_bindgen(js_name = fromJSON)]
    pub fn from_json(s: &str) -> Result<JsWorkbook, JsError> {
        let wb = Workbook::from_json(s.as_bytes())
            .map_err(|e| JsError::new(&e.to_string()))?;
        Ok(JsWorkbook { inner: wb })
    }

    /// Serializes the workbook to its canonical JSON string.
    #[wasm_bindgen(js_name = toJSON)]
    pub fn to_json(&self) -> Result<String, JsError> {
        self.inner.to_json().map_err(|e| JsError::new(&e.to_string()))
    }

    /// Adds a new sheet with the given name and returns on success.
    #[wasm_bindgen(js_name = addSheet)]
    pub fn add_sheet(&mut self, name: &str) -> Result<(), JsError> {
        self.inner
            .add_sheet(Worksheet::new(name.to_string()))
            .map(|_| ())
            .map_err(|e| JsError::new(&e.to_string()))
    }

    /// Sets the cell at `a1` on `sheet` to the given `input`.
    ///
    /// If `input` starts with `=` it is treated as a formula; otherwise the
    /// string is coerced: a numeric literal becomes a `Number`, `"true"` /
    /// `"false"` (case-insensitive) become a `Boolean`, and everything else
    /// becomes a `Text` value.
    pub fn set(&mut self, sheet: &str, a1: &str, input: &str) -> Result<(), JsError> {
        let addr = Address::from_a1(a1)
            .ok_or_else(|| JsError::new(&format!("invalid A1 address: {a1:?}")))?;

        let cell_input = if input.starts_with('=') {
            CellInput::Formula(input.to_string())
        } else if let Ok(n) = input.parse::<f64>() {
            CellInput::Literal(Value::Number(n))
        } else if input.eq_ignore_ascii_case("true") {
            CellInput::Literal(Value::Boolean(true))
        } else if input.eq_ignore_ascii_case("false") {
            CellInput::Literal(Value::Boolean(false))
        } else {
            CellInput::Literal(Value::Text(input.to_string()))
        };

        self.inner
            .set(sheet, addr, cell_input)
            .map(|_| ())
            .map_err(|e| JsError::new(&e.to_string()))
    }

    /// Sets the cell at `a1` on `sheet` to a **Date-typed** serial value.
    ///
    /// Unlike [`set`](Self::set) — which stores a numeric string as a plain
    /// `Number` — this stores `serial` as a `Date`, the type a host uses for a
    /// cell it means as a date. The engine's arithmetic type propagation then
    /// keeps it rendering as a date through offset arithmetic: `=A1+1` and
    /// `=A1-7` on a Date cell stay dates (`=A1-B1` between two Date cells is a
    /// plain day count, matching Google Sheets).
    ///
    /// The serial round-trips exactly — it is stored verbatim, never
    /// reconstructed via `DATE(y, m, d)` — so pre-1900 (negative) serials and
    /// fractional time-of-day components are preserved bit-for-bit.
    #[wasm_bindgen(js_name = setDate)]
    pub fn set_date(&mut self, sheet: &str, a1: &str, serial: f64) -> Result<(), JsError> {
        let addr = Address::from_a1(a1)
            .ok_or_else(|| JsError::new(&format!("invalid A1 address: {a1:?}")))?;

        self.inner
            .set(sheet, addr, CellInput::Literal(Value::Date(serial)))
            .map(|_| ())
            .map_err(|e| JsError::new(&e.to_string()))
    }

    /// Clears the cell at `a1` on `sheet`.
    pub fn clear(&mut self, sheet: &str, a1: &str) {
        if let Some(addr) = Address::from_a1(a1) {
            self.inner.clear(sheet, addr);
        }
    }

    /// Defines a workbook-scoped named range.
    #[wasm_bindgen(js_name = defineName)]
    pub fn define_name(&mut self, name: &str, ref_str: &str) -> Result<(), JsError> {
        self.inner
            .define_name(name, ref_str)
            .map(|_| ())
            .map_err(|e| JsError::new(&e.to_string()))
    }

    /// Redefines (renames target of) a workbook-scoped named range.
    #[wasm_bindgen(js_name = redefineName)]
    pub fn redefine_name(&mut self, name: &str, ref_str: &str) -> Result<(), JsError> {
        self.inner
            .redefine_name(name, ref_str)
            .map(|_| ())
            .map_err(|e| JsError::new(&e.to_string()))
    }

    /// Runs a full recalculation against the given context JSON.
    ///
    /// `context_json` must be a JSON object:
    /// `{"timestamp_ms": 0, "timezone": "UTC", "rng_seed": 0}`
    ///
    /// Returns a JSON array of change objects:
    /// `[{"sheet":"Sheet1","addr":"A1","old":{...},"new":{...}}, ...]`
    pub fn recalc(&mut self, context_json: &str) -> Result<String, JsError> {
        #[derive(serde::Deserialize)]
        struct CtxInput {
            timestamp_ms: i64,
            timezone: String,
            rng_seed: u64,
        }

        let input: CtxInput = serde_json::from_str(context_json)
            .map_err(|e| JsError::new(&format!("invalid context JSON: {e}")))?;

        let ctx = RecalcContext::new(input.timestamp_ms, &input.timezone, input.rng_seed)
            .ok_or_else(|| JsError::new("unknown timezone"))?;

        let changes = self.inner.recalc(&ctx);
        let json_changes: Vec<serde_json::Value> = changes.iter().map(change_to_json).collect();
        serde_json::to_string(&json_changes)
            .map_err(|e| JsError::new(&e.to_string()))
    }

    /// Returns the resolved value at `a1` on `sheet` as a JS value.
    ///
    /// Returns `null` if the cell has no value; otherwise returns a tagged
    /// JSON object: `{"type":"number","value":1.5}`, `{"type":"text","value":"hello"}`,
    /// `{"type":"bool","value":true}`, `{"type":"date","value":46180}`,
    /// `{"type":"error","error":"#REF!"}`, `{"type":"empty"}`,
    /// or `{"type":"array","value":[[...],...]}`.
    pub fn resolved(&self, sheet: &str, a1: &str) -> Result<JsValue, JsError> {
        let addr = Address::from_a1(a1)
            .ok_or_else(|| JsError::new(&format!("invalid A1 address: {a1:?}")))?;

        match self.inner.resolved(sheet, addr) {
            None => Ok(JsValue::NULL),
            Some(Resolved { value, .. }) => {
                let json = value_to_json(&value);
                let s = serde_json::to_string(&json)
                    .map_err(|e| JsError::new(&e.to_string()))?;
                Ok(JsValue::from_str(&s))
            }
        }
    }
}
