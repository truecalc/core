use serde::Serialize;
use tsify_next::Tsify;
use wasm_bindgen::prelude::*;

pub mod depgraph;

use depgraph::{DependentsResult, PrecedentsResult};

use truecalc_core::types::SparklineValue;
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
        Ok(f) => TranslateResult {
            formula: Some(f),
            error: None,
        },
        Err(e) => TranslateResult {
            formula: None,
            error: Some(e.to_string()),
        },
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
        // Additive `message` (Google Sheets parity, e.g. the arity diagnostic);
        // absent for bare errors, so existing consumers are unaffected.
        Value::ErrorMsg(code, msg) => {
            serde_json::json!({"type": "error", "error": code, "message": msg})
        }
        Value::Empty => serde_json::json!({"type": "empty"}),
        Value::Array(rows) => {
            let arr: Vec<Vec<serde_json::Value>> = rows
                .iter()
                .map(|row| row.iter().map(value_to_json).collect())
                .collect();
            serde_json::json!({"type": "array", "value": arr})
        }
        // The sparkline's parsed spec, carried in full: it is the value's
        // identity, and every text projection of it is empty.
        Value::Sparkline(spec) => {
            let cell = |v: &SparklineValue| match v {
                SparklineValue::Number(n) => value_to_json(&Value::Number(*n)),
                SparklineValue::Text(s) => value_to_json(&Value::Text(s.clone())),
                SparklineValue::Bool(b) => value_to_json(&Value::Boolean(*b)),
                SparklineValue::Blank => value_to_json(&Value::Empty),
            };
            let data: Vec<serde_json::Value> = spec.data.iter().map(&cell).collect();
            let options: Vec<serde_json::Value> = spec
                .options
                .iter()
                .map(|(k, v)| serde_json::json!([k, cell(v)]))
                .collect();
            serde_json::json!({
                "type": "sparkline",
                "value": {
                    "charttype": spec.chart_type.as_str(),
                    "data": data,
                    "options": options,
                }
            })
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
        let wb = Workbook::from_json(s.as_bytes()).map_err(|e| JsError::new(&e.to_string()))?;
        Ok(JsWorkbook { inner: wb })
    }

    /// Serializes the workbook to its canonical JSON string.
    #[wasm_bindgen(js_name = toJSON)]
    pub fn to_json(&self) -> Result<String, JsError> {
        self.inner
            .to_json()
            .map_err(|e| JsError::new(&e.to_string()))
    }

    /// Adds a new sheet with the given name and returns on success.
    #[wasm_bindgen(js_name = addSheet)]
    pub fn add_sheet(&mut self, name: &str) -> Result<(), JsError> {
        self.inner
            .add_sheet(Worksheet::new(name.to_string()))
            .map(|_| ())
            .map_err(|e| JsError::new(&e.to_string()))
    }

    /// Inserts a new sheet named `name` at 0-based tab position `index`,
    /// shifting later tabs right. `index` equal to the current sheet count
    /// appends, the same as [`addSheet`](Self::add_sheet).
    ///
    /// Errors on a duplicate name (case-insensitive), an empty/too-long name,
    /// the per-workbook sheet cap, or `index` beyond the valid range.
    #[wasm_bindgen(js_name = insertSheet)]
    pub fn insert_sheet(&mut self, index: u32, name: &str) -> Result<(), JsError> {
        self.inner
            .insert_sheet(index as usize, Worksheet::new(name.to_string()))
            .map_err(|e| JsError::new(&e.to_string()))
    }

    /// Removes the sheet named `name`, if one exists. An unknown `name` is a
    /// silent no-op, matching [`removeName`](Self::remove_name).
    ///
    /// A workbook-scoped named range or table may now dangle to the removed
    /// sheet — removal does not re-check that invariant. It is re-verified
    /// only at the next [`toJSON`](Self::to_json) call, so a dangling
    /// reference surfaces there, not here.
    #[wasm_bindgen(js_name = removeSheet)]
    pub fn remove_sheet(&mut self, name: &str) {
        self.inner.remove_sheet(name);
    }

    /// Renames the sheet currently named `from` to `to`, repointing every
    /// named-range/table `ref` and every formula reference that qualified a
    /// cell with the old name.
    ///
    /// Errors if `from` does not exist, `to` is invalid or collides with
    /// another sheet, or the rewrite itself would violate an invariant (a
    /// rewritten formula exceeding the formula-length cap, or a repointed
    /// table landing on another table's range). A rejected rename leaves the
    /// workbook untouched.
    #[wasm_bindgen(js_name = renameSheet)]
    pub fn rename_sheet(&mut self, from: &str, to: &str) -> Result<(), JsError> {
        self.inner
            .rename_sheet(from, to)
            .map_err(|e| JsError::new(&e.to_string()))
    }

    /// Moves the sheet at 0-based tab position `from` to position `to`,
    /// shifting the sheets in between.
    ///
    /// Errors if either position is out of range (the valid range is named
    /// in the error message).
    #[wasm_bindgen(js_name = moveSheet)]
    pub fn move_sheet(&mut self, from: u32, to: u32) -> Result<(), JsError> {
        self.inner
            .move_sheet(from as usize, to as usize)
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

    /// The total number of populated cells across every sheet — the
    /// quantity the per-workbook cell cap bounds.
    #[wasm_bindgen(js_name = totalCells)]
    pub fn total_cells(&self) -> u32 {
        self.inner.total_cells() as u32
    }

    /// Returns the **authored** cell at `a1` on `sheet` as a JS value, or
    /// `null` if no cell is authored there.
    ///
    /// This is the authored cell only — a literal or a formula physically
    /// present at that address. A *spilled* (non-anchor) cell returns `null`
    /// here even though [`resolved`](Self::resolved) would return its
    /// reconstructed value at that address; use `resolved` to read the
    /// effective value at any address, authored or spilled.
    ///
    /// Returns a tagged JSON object: `{"formula": "=A1+1"|null, "value": <a
    /// tagged value object, the same shape resolved()/get() share>}`.
    pub fn get(&self, sheet: &str, a1: &str) -> Result<JsValue, JsError> {
        let addr = Address::from_a1(a1)
            .ok_or_else(|| JsError::new(&format!("invalid A1 address: {a1:?}")))?;

        match self.inner.get(sheet, addr) {
            None => Ok(JsValue::NULL),
            Some(cell) => {
                let json = serde_json::json!({
                    "formula": cell.formula(),
                    "value": value_to_json(cell.value()),
                });
                let s = serde_json::to_string(&json).map_err(|e| JsError::new(&e.to_string()))?;
                Ok(JsValue::from_str(&s))
            }
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

    /// Removes a workbook-scoped named range, if one exists. A `name` that
    /// does not exist is a silent no-op, matching [`clear`](Self::clear).
    #[wasm_bindgen(js_name = removeName)]
    pub fn remove_name(&mut self, name: &str) {
        self.inner.remove_name(name);
    }

    /// Defines a workbook-scoped table (issue #868): `ref_str`'s first row
    /// becomes the table's header row, so formulas can use `Table[Column]`
    /// (whole-column) and `Table[@Column]` / unqualified `[@Column]`
    /// (current-row) structured references against it.
    #[wasm_bindgen(js_name = defineTable)]
    pub fn define_table(&mut self, name: &str, ref_str: &str) -> Result<(), JsError> {
        self.inner
            .define_table(name, ref_str)
            .map(|_| ())
            .map_err(|e| JsError::new(&e.to_string()))
    }

    /// Redefines (retargets) a workbook-scoped table.
    #[wasm_bindgen(js_name = redefineTable)]
    pub fn redefine_table(&mut self, name: &str, ref_str: &str) -> Result<(), JsError> {
        self.inner
            .redefine_table(name, ref_str)
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
        serde_json::to_string(&json_changes).map_err(|e| JsError::new(&e.to_string()))
    }

    /// Recomputes only the formula cells affected by an edit and returns the
    /// ordered changes — the incremental counterpart of [`recalc`](Self::recalc).
    ///
    /// `context_json` is the same context object `recalc` takes. `edited_json`
    /// is a JSON array of the cells a mutation touched:
    /// `[{"sheet":"Sheet1","addr":"A1"}, ...]`. An empty array is valid input,
    /// not an error — always-dirty volatile cells (`NOW`, `TODAY`, ...) still
    /// recompute.
    ///
    /// Returns the identical `Change[]` JSON array shape [`recalc`](Self::recalc)
    /// returns: `[{"sheet":...,"addr":...,"old":{...},"new":{...}}, ...]`.
    #[wasm_bindgen(js_name = recalcIncremental)]
    pub fn recalc_incremental(
        &mut self,
        context_json: &str,
        edited_json: &str,
    ) -> Result<String, JsError> {
        #[derive(serde::Deserialize)]
        struct CtxInput {
            timestamp_ms: i64,
            timezone: String,
            rng_seed: u64,
        }

        #[derive(serde::Deserialize)]
        struct EditedInput {
            sheet: String,
            addr: String,
        }

        let input: CtxInput = serde_json::from_str(context_json)
            .map_err(|e| JsError::new(&format!("invalid context JSON: {e}")))?;

        let ctx = RecalcContext::new(input.timestamp_ms, &input.timezone, input.rng_seed)
            .ok_or_else(|| JsError::new("unknown timezone"))?;

        let edited_input: Vec<EditedInput> = serde_json::from_str(edited_json)
            .map_err(|e| JsError::new(&format!("invalid edited JSON: {e}")))?;
        let edited: Vec<(String, Address)> = edited_input
            .into_iter()
            .map(|e| {
                let addr = Address::from_a1(&e.addr)
                    .ok_or_else(|| JsError::new(&format!("invalid A1 address: {:?}", e.addr)))?;
                Ok((e.sheet, addr))
            })
            .collect::<Result<_, JsError>>()?;

        let changes = self.inner.recalc_incremental(&ctx, &edited);
        let json_changes: Vec<serde_json::Value> = changes.iter().map(change_to_json).collect();
        serde_json::to_string(&json_changes).map_err(|e| JsError::new(&e.to_string()))
    }

    /// Returns the resolved value at `a1` on `sheet` as a JS value.
    ///
    /// Returns `null` if the cell has no value; otherwise returns a tagged
    /// JSON object: `{"type":"number","value":1.5}`, `{"type":"text","value":"hello"}`,
    /// `{"type":"bool","value":true}`, `{"type":"date","value":46180}`,
    /// `{"type":"error","error":"#REF!"}`, `{"type":"empty"}`,
    /// or `{"type":"array","value":[[...],...]}`.
    ///
    /// This is the **effective** value — it resolves through array spills.
    /// When the queried address is a *spilled* (non-anchor) cell, the object
    /// additionally carries `"anchor":"B2"`, the A1 address of the spilling
    /// formula on the same sheet; the key is absent for an authored cell.
    /// Use [`get`](Self::get) to read only what is physically authored at
    /// `a1`.
    pub fn resolved(&self, sheet: &str, a1: &str) -> Result<JsValue, JsError> {
        let addr = Address::from_a1(a1)
            .ok_or_else(|| JsError::new(&format!("invalid A1 address: {a1:?}")))?;

        match self.inner.resolved(sheet, addr) {
            None => Ok(JsValue::NULL),
            Some(Resolved { value, anchor }) => {
                let mut json = value_to_json(&value);
                if let (Some(anchor), serde_json::Value::Object(map)) = (anchor, &mut json) {
                    map.insert(
                        "anchor".to_string(),
                        serde_json::Value::String(anchor.to_a1()),
                    );
                }
                let s = serde_json::to_string(&json).map_err(|e| JsError::new(&e.to_string()))?;
                Ok(JsValue::from_str(&s))
            }
        }
    }

    /// What the cell at `a1` on `sheet` **reads** — its precedents.
    ///
    /// Answers: where does this number come from? Returns
    /// `{ cell, precedents: [{ depth, reference }], truncated, truncatedBy? }`.
    /// Each `reference` is a tagged union: `cell`, `range` (never expanded into
    /// its members), `name` (carrying what the name currently targets), or
    /// `unresolved` (a dangling sheet or name, the reason the cell will show
    /// `#REF!` / `#NAME?`). Every `cell` / `range` carries its own `sheet`, so
    /// a cross-sheet precedent stays cross-sheet.
    ///
    /// `precedents` is always an array: a literal, an empty cell and a
    /// constant formula all return `[]`.
    ///
    /// `maxDepth` defaults to `1` (direct precedents only) and is clamped to
    /// `1..=64`; `maxNodes` defaults to `1000` and is clamped to `10000`.
    /// **If the walk stops at either bound, `truncated` is `true` and
    /// `truncatedBy` says which one** — the returned array is then a prefix of
    /// the answer, not the answer. `truncated` is always present; branch on
    /// it, not on `truncatedBy`.
    ///
    /// Reuses the workbook's cached dependency graph when it is warm (see
    /// `truecalc_workbook`'s `graph_cache` module docs) and builds fresh
    /// otherwise, so the result is never stale either way: it reflects every
    /// `set` / `clear` / `defineName` since the last `recalc`, and is
    /// meaningful before any `recalc` at all. A cold build costs
    /// `O(formula cells)` per call — an upper bound expected to improve, not
    /// a fixed contract.
    ///
    /// Throws on an unknown sheet or a malformed A1 address.
    #[wasm_bindgen(js_name = precedentsOf)]
    pub fn precedents_of(
        &self,
        sheet: &str,
        a1: &str,
        max_depth: Option<u32>,
        max_nodes: Option<u32>,
    ) -> Result<PrecedentsResult, JsError> {
        depgraph::precedents_of(&self.inner, sheet, a1, max_depth, max_nodes)
            .map_err(|e| JsError::new(&e))
    }

    /// What **reads** the cell at `a1` on `sheet` — its dependents, i.e. what
    /// would have to recalculate (and could visibly change) if you edited it.
    ///
    /// Returns `{ cell, dependents: [{ depth, sheet, a1 }], truncated,
    /// truncatedBy? }`. Dependents are always concrete formula cells and
    /// include cells that reach this one through a range that contains it or
    /// through a named range whose target contains it — the caller never has
    /// to reason about range compression or name indirection.
    ///
    /// `dependents` is always an array; `[]` means nothing reads the cell.
    ///
    /// Bounds, truncation reporting and freshness are exactly as for
    /// [`precedentsOf`](Self::precedents_of), with one addition: finding what
    /// reads a cell means testing every distinct range node and every name
    /// against it, so each emitted (or expanded) node costs `O(distinct
    /// ranges + names)` on top of the `O(formula cells)` graph build — a
    /// workbook with many named ranges or range formulas makes this call
    /// noticeably more expensive than `precedentsOf` at the same size.
    ///
    /// Throws on an unknown sheet or a malformed A1 address.
    #[wasm_bindgen(js_name = dependentsOf)]
    pub fn dependents_of(
        &self,
        sheet: &str,
        a1: &str,
        max_depth: Option<u32>,
        max_nodes: Option<u32>,
    ) -> Result<DependentsResult, JsError> {
        depgraph::dependents_of(&self.inner, sheet, a1, max_depth, max_nodes)
            .map_err(|e| JsError::new(&e))
    }
}
