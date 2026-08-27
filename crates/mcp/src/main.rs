// truecalc MCP server — hand-rolled JSON-RPC over stdio (MCP protocol 2024-11-05)

use std::collections::HashMap;
use std::io::{self, BufRead, Write};

use truecalc_core::types::{SparklineChartType, SparklineSpec, SparklineValue};
use truecalc_core::{Engine, Expr, Registry, Value};
use truecalc_workbook::{
    Address, CellInput, EngineFlavor as WbEngine, RecalcContext, Value as WbValue, Workbook,
    Worksheet,
};
use serde_json::{json, Value as JsonValue};

// ─── Conformance ─────────────────────────────────────────────────────────────

struct Engines {
    google_sheets: Engine,
}

impl Engines {
    fn new() -> Self {
        Self { google_sheets: Engine::sheets() }
    }

    fn select(&self, conformance: &str) -> Option<&Engine> {
        match conformance {
            "google-sheets" => Some(&self.google_sheets),
            _ => None,
        }
    }
}

fn parse_conformance_arg(args: &[String]) -> String {
    let mut iter = args.iter();
    while let Some(arg) = iter.next() {
        if arg == "--conformance" {
            if let Some(val) = iter.next() {
                return val.clone();
            }
        }
    }
    "google-sheets".to_string()
}

// ─── Session store ────────────────────────────────────────────────────────────

const MAX_WORKBOOKS: usize = 32;
const MAX_TOTAL_BYTES: usize = 256 * 1024 * 1024;

struct SessionStore {
    workbooks: std::collections::HashMap<String, Workbook>,
    total_json_bytes: usize,
    next_id: u64,
}

impl SessionStore {
    fn new() -> Self {
        Self { workbooks: std::collections::HashMap::new(), total_json_bytes: 0, next_id: 0 }
    }

    fn allocate_id(&mut self) -> String {
        let id = format!("wb_{}", self.next_id);
        self.next_id += 1;
        id
    }

    fn create(&mut self, engine: WbEngine) -> Result<String, String> {
        if self.workbooks.len() >= MAX_WORKBOOKS {
            return Err(format!("session limit reached: max {} workbooks per process", MAX_WORKBOOKS));
        }
        let id = self.allocate_id();
        let mut wb = Workbook::new(engine);
        // Seed a default sheet so workbook_set/workbook_get/workbook_recalc
        // work immediately, matching what Google Sheets and Excel do when
        // you create a new workbook (see truecalc/core#878).
        wb.add_sheet(Worksheet::new("Sheet1"))
            .map_err(|e| format!("could not seed default sheet: {}", e))?;
        self.workbooks.insert(id.clone(), wb);
        Ok(id)
    }

    fn import(&mut self, json: &str) -> Result<String, String> {
        if self.workbooks.len() >= MAX_WORKBOOKS {
            return Err(format!("session limit reached: max {} workbooks per process", MAX_WORKBOOKS));
        }
        if self.total_json_bytes.saturating_add(json.len()) > MAX_TOTAL_BYTES {
            return Err(format!("memory limit would be exceeded: aggregate session size capped at {} MiB", MAX_TOTAL_BYTES / (1024 * 1024)));
        }
        let wb = Workbook::from_json(json.as_bytes()).map_err(|e| e.to_string())?;
        let id = self.allocate_id();
        self.total_json_bytes = self.total_json_bytes.saturating_add(json.len());
        self.workbooks.insert(id.clone(), wb);
        Ok(id)
    }

    fn get_mut(&mut self, id: &str) -> Option<&mut Workbook> {
        self.workbooks.get_mut(id)
    }
}

// ─── Entry point ─────────────────────────────────────────────────────────────

fn main() {
    let cli_args: Vec<String> = std::env::args().collect();
    let default_conformance = parse_conformance_arg(&cli_args);
    let engines = Engines::new();
    let mut session_store = SessionStore::new();

    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut out = stdout.lock();

    for line in stdin.lock().lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => break,
        };
        if line.is_empty() {
            continue;
        }

        let request: JsonValue = match serde_json::from_str(&line) {
            Ok(v) => v,
            Err(_) => {
                let err = json!({
                    "jsonrpc": "2.0",
                    "id": null,
                    "error": { "code": -32700, "message": "Parse error" }
                });
                writeln!(out, "{}", serde_json::to_string(&err).unwrap()).unwrap();
                out.flush().unwrap();
                continue;
            }
        };

        if !request.is_object() {
            let err = json!({
                "jsonrpc": "2.0",
                "id": null,
                "error": { "code": -32700, "message": "Parse error" }
            });
            writeln!(out, "{}", serde_json::to_string(&err).unwrap()).unwrap();
            out.flush().unwrap();
            continue;
        }

        if request.get("id").is_none() {
            continue;
        }

        let response = handle_request(&request, &default_conformance, &engines, &mut session_store);
        let mut response_str = serde_json::to_string(&response).expect("serialisation error");
        response_str.push('\n');
        out.write_all(response_str.as_bytes()).expect("stdout write error");
        out.flush().expect("stdout flush error");
    }
}

fn handle_request(req: &JsonValue, default_conformance: &str, engines: &Engines, store: &mut SessionStore) -> JsonValue {
    let id = &req["id"];
    let method = req["method"].as_str().unwrap_or("");
    let params = &req["params"];

    match method {
        "initialize" => json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": {
                "protocolVersion": "2024-11-05",
                "capabilities": { "tools": {} },
                "serverInfo": { "name": "truecalc-mcp", "version": "0.1.0" }
            }
        }),

        "tools/list" => json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": { "tools": tools_list() }
        }),

        "tools/call" => {
            let name = params["name"].as_str().unwrap_or("");
            let args = &params["arguments"];
            // The flag comes from the outcome type, never from probing the
            // payload for an "error" key: `validate` answering {"valid": false}
            // is a successful answer to a question, not a failed call.
            let (result, is_error) = match dispatch_tool(name, args, default_conformance, engines, store) {
                Ok(v) => (v, false),
                Err(e) => (json!({ "error": e }), true),
            };
            let mut tool_result = json!({
                "content": [{ "type": "text", "text": serde_json::to_string(&result).expect("result serialisation is infallible") }]
            });
            if is_error {
                tool_result["isError"] = json!(true);
            }
            json!({
                "jsonrpc": "2.0",
                "id": id,
                "result": tool_result
            })
        }

        _ => json!({
            "jsonrpc": "2.0",
            "id": id,
            "error": { "code": -32601, "message": "Method not found" }
        }),
    }
}

// ─── Tool dispatch ────────────────────────────────────────────────────────────

fn dispatch_tool(name: &str, args: &JsonValue, default_conformance: &str, engines: &Engines, store: &mut SessionStore) -> Result<JsonValue, String> {
    match name {
        "evaluate" => tool_evaluate(args, default_conformance, engines),
        "validate" => tool_validate(args, engines),
        "explain" => tool_explain(args, engines),
        "batch_evaluate" => tool_batch_evaluate(args, default_conformance, engines),
        "list_functions" => tool_list_functions(args),
        "get_stats" => tool_get_stats(),
        "workbook_create" => tool_workbook_create(args, store),
        "workbook_set" => tool_workbook_set(args, store),
        "workbook_get" => tool_workbook_get(args, store),
        "workbook_recalc" => tool_workbook_recalc(args, store),
        "workbook_export" => tool_workbook_export(args, store),
        "workbook_import" => tool_workbook_import(args, store),
        _ => Err(format!("Unknown tool: {}", name)),
    }
}

// ─── Individual tools ─────────────────────────────────────────────────────────

fn tool_evaluate(args: &JsonValue, default_conformance: &str, engines: &Engines) -> Result<JsonValue, String> {
    let formula = args["formula"].as_str().ok_or("missing formula")?;
    let conformance = optional_str(args, "conformance")?.unwrap_or(default_conformance);
    let engine = engines
        .select(conformance)
        .ok_or_else(|| format!("Unknown conformance target: '{}'", conformance))?;
    let vars = parse_variables(&args["variables"])?;
    let value = engine.evaluate(formula, &vars);
    let mut out = value_to_json(&value);
    out["accepted"] = accepted_bindings(&vars, conformance);
    Ok(out)
}

fn tool_validate(args: &JsonValue, engines: &Engines) -> Result<JsonValue, String> {
    let formula = args["formula"].as_str().ok_or("missing formula")?;
    // Both arms are Ok: the tool was asked whether the formula parses and it
    // answered. Only a call it could not carry out is an error.
    match engines.google_sheets.validate(formula) {
        Ok(_) => Ok(json!({ "valid": true })),
        Err(e) => Ok(json!({ "valid": false, "error": e.to_string() })),
    }
}

fn tool_explain(args: &JsonValue, engines: &Engines) -> Result<JsonValue, String> {
    let formula = args["formula"].as_str().ok_or("missing formula")?;
    Ok(match engines.google_sheets.parse(formula) {
        Ok(expr) => {
            let mut functions = Vec::new();
            collect_functions(&expr, &mut functions);
            functions.sort_unstable();
            functions.dedup();
            let description = if functions.is_empty() {
                "Formula with no function calls".to_string()
            } else {
                format!("Formula using: {}", functions.join(", "))
            };
            json!({ "description": description, "functions_used": functions })
        }
        Err(e) => json!({
            "description": format!("Invalid formula: {}", e),
            "functions_used": []
        }),
    })
}

fn tool_batch_evaluate(args: &JsonValue, default_conformance: &str, engines: &Engines) -> Result<JsonValue, String> {
    let formulas = args["formulas"].as_array().ok_or("missing formulas array")?;
    let conformance = optional_str(args, "conformance")?.unwrap_or(default_conformance);
    let engine = engines
        .select(conformance)
        .ok_or_else(|| format!("Unknown conformance target: '{}'", conformance))?;
    let vars = parse_variables(&args["variables"])?;
    let mut results: Vec<JsonValue> = Vec::with_capacity(formulas.len());
    for (i, f) in formulas.iter().enumerate() {
        // Coerced to "", a non-string entry evaluates to an empty value in the
        // middle of an otherwise correct batch — a hole the caller cannot see.
        let formula = f
            .as_str()
            .ok_or_else(|| format!("\"formulas\"[{i}] must be a string, got {f}"))?;
        results.push(value_to_json(&engine.evaluate(formula, &vars)));
    }
    Ok(json!(results))
}

/// How many entries a *filtered* `list_functions` call returns unless the
/// caller says otherwise. The unfiltered call is the pre-existing "dump the
/// catalogue" request and stays uncapped, so nothing that relies on it changes.
const LIST_FUNCTIONS_DEFAULT_LIMIT: usize = 100;

const LIST_FUNCTIONS_ARGS: [&str; 4] = ["category", "name_contains", "names", "limit"];

/// Read an optional string argument, rejecting a value of the wrong type.
///
/// `unwrap_or(default)` would take `{"category": 7}` as "no category given"
/// and answer the narrow question with the whole catalogue.
fn optional_str<'a>(args: &'a JsonValue, key: &str) -> Result<Option<&'a str>, String> {
    match &args[key] {
        JsonValue::Null => Ok(None),
        JsonValue::String(s) => Ok(Some(s.as_str())),
        other => Err(format!("\"{key}\" must be a string, got {other}")),
    }
}

/// Read an optional integer argument, rejecting a value of the wrong type.
/// `kind` names what would have been accepted, since "integer" alone does not
/// explain why a negative seed or a fractional timestamp was refused.
fn optional_int<T>(args: &JsonValue, key: &str, kind: &str, read: fn(&JsonValue) -> Option<T>) -> Result<Option<T>, String> {
    match &args[key] {
        JsonValue::Null => Ok(None),
        v => read(v).map(Some).ok_or_else(|| format!("\"{key}\" must be {kind}, got {v}")),
    }
}

/// A one-word description of the shape a value bound at — enough to see that a
/// list was read as a list, without restating the request.
fn value_shape(v: &Value) -> String {
    match v {
        Value::Number(_) | Value::Date(_) => "number".to_owned(),
        Value::Text(_) => "text".to_owned(),
        Value::Bool(_) => "bool".to_owned(),
        Value::Empty => "empty".to_owned(),
        Value::Error(_) | Value::ErrorMsg(..) => "error".to_owned(),
        Value::Zoned(_) => "zoned".to_owned(),
        Value::Sparkline(_) => "sparkline".to_owned(),
        Value::Array(items) => format!("array[{}]", items.len()),
    }
}

/// What the server resolved an evaluate-shaped request to, in its own terms:
/// which names bound at which shape, and the conformance target actually used.
/// A caller that has to pay a round trip to check this will not check.
fn accepted_bindings(vars: &HashMap<String, Value>, conformance: &str) -> JsonValue {
    let bound: serde_json::Map<String, JsonValue> = vars
        .iter()
        .map(|(k, v)| (k.clone(), json!(value_shape(v))))
        .collect();
    json!({ "bound": bound, "conformance": conformance })
}

fn tool_list_functions(args: &JsonValue) -> Result<JsonValue, String> {
    if let Some(obj) = args.as_object() {
        for key in obj.keys() {
            if !LIST_FUNCTIONS_ARGS.contains(&key.as_str()) {
                return Err(format!(
                    "unsupported argument \"{key}\"; supported: {}",
                    LIST_FUNCTIONS_ARGS.join(", ")
                ));
            }
        }
    }

    let category = optional_str(args, "category")?.map(|c| c.to_ascii_lowercase());
    let name_contains = optional_str(args, "name_contains")?.map(|n| n.to_ascii_uppercase());
    let names: Option<Vec<String>> = match &args["names"] {
        JsonValue::Null => None,
        JsonValue::Array(items) => Some(
            items
                .iter()
                .map(|i| {
                    i.as_str()
                        .map(|s| s.to_ascii_uppercase())
                        .ok_or_else(|| format!("\"names\" must contain strings, got {i}"))
                })
                .collect::<Result<Vec<_>, _>>()?,
        ),
        other => return Err(format!("\"names\" must be an array of strings, got {other}")),
    };
    let limit = match &args["limit"] {
        JsonValue::Null => {
            if category.is_none() && name_contains.is_none() && names.is_none() {
                usize::MAX
            } else {
                LIST_FUNCTIONS_DEFAULT_LIMIT
            }
        }
        v => match v.as_u64().filter(|n| *n > 0) {
            Some(n) => n as usize,
            None => return Err(format!("\"limit\" must be a positive integer, got {v}")),
        },
    };

    let registry = Registry::new();
    if let Some(cat) = &category {
        let mut known: Vec<&str> = registry.list_functions().map(|(_, m)| m.category).collect();
        known.sort_unstable();
        known.dedup();
        if !known.contains(&cat.as_str()) {
            return Err(format!("unknown category \"{cat}\"; known: {}", known.join(", ")));
        }
    }

    let mut entries: Vec<JsonValue> = registry
        .list_functions()
        .filter(|(name, meta)| {
            category.as_ref().is_none_or(|c| meta.category == c)
                && name_contains.as_ref().is_none_or(|n| name.contains(n.as_str()))
                && names.as_ref().is_none_or(|wanted| wanted.iter().any(|w| w == name))
        })
        .map(|(name, meta)| json!({
            "name": name,
            "category": meta.category,
            "syntax": meta.signature,
            "description": meta.description,
        }))
        .collect();
    entries.sort_by_key(|e| e["name"].as_str().unwrap_or("").to_owned());

    // A requested name that matched nothing is reported: the caller asked
    // about it by name, and an entry missing from the list is otherwise
    // indistinguishable from one that was simply capped away.
    let not_found: Vec<&String> = names
        .iter()
        .flatten()
        .filter(|w| !entries.iter().any(|e| e["name"].as_str() == Some(w.as_str())))
        .collect();

    let total_matched = entries.len();
    entries.truncate(limit);
    let returned = entries.len();
    let mut out = json!({
        "functions": entries,
        "total_matched": total_matched,
        "returned": returned,
    });
    if !not_found.is_empty() {
        out["not_found"] = json!(not_found);
    }
    Ok(out)
}

fn tool_get_stats() -> Result<JsonValue, String> {
    let registry = Registry::new();
    let mut by_category: std::collections::BTreeMap<&str, u32> = std::collections::BTreeMap::new();
    let mut total: u32 = 0;
    for (_name, meta) in registry.list_functions() {
        *by_category.entry(meta.category).or_insert(0) += 1;
        total += 1;
    }
    let categories: Vec<JsonValue> = by_category
        .iter()
        .map(|(cat, count)| json!({ "category": cat, "count": count }))
        .collect();
    Ok(json!({
        "version": env!("CARGO_PKG_VERSION"),
        "total_functions": total,
        "by_category": categories
    }))
}

// ─── Workbook tools ───────────────────────────────────────────────────────────

fn tool_workbook_create(args: &JsonValue, store: &mut SessionStore) -> Result<JsonValue, String> {
    let engine_str = args["engine"].as_str().ok_or("missing engine")?;
    let engine = match engine_str {
        "sheets" => WbEngine::Sheets,
        "excel" => WbEngine::Excel,
        other => return Err(format!("unknown engine: {}", other)),
    };
    store.create(engine).map(|id| json!({ "workbook_id": id }))
}

fn tool_workbook_set(args: &JsonValue, store: &mut SessionStore) -> Result<JsonValue, String> {
    let workbook_id = args["workbook_id"].as_str().ok_or("missing workbook_id")?;
    let sheet = args["sheet"].as_str().ok_or("missing sheet")?;
    let cell = args["cell"].as_str().ok_or("missing cell")?;
    let value = args["value"].as_str().ok_or("missing value")?;

    let wb = store
        .get_mut(workbook_id)
        .ok_or_else(|| format!("workbook not found: {}", workbook_id))?;

    let addr = Address::from_a1(&cell.to_uppercase())
        .ok_or_else(|| format!("invalid cell address: {}", cell))?;

    // The kind the value was read as travels back with the answer: "007" is a
    // number and "1/2" is text, and which one happened is not otherwise
    // visible without reading the cell back.
    let (input, read_as) = if value.starts_with('=') {
        (CellInput::Formula(value.to_owned()), "formula")
    } else if value == "TRUE" || value == "true" {
        (CellInput::Literal(WbValue::Boolean(true)), "boolean")
    } else if value == "FALSE" || value == "false" {
        (CellInput::Literal(WbValue::Boolean(false)), "boolean")
    } else if let Ok(n) = value.parse::<f64>() {
        (CellInput::Literal(WbValue::Number(n)), "number")
    } else if let Some(zi) = truecalc_core::types::zoned::parse_rfc9557(value) {
        (CellInput::Literal(WbValue::Zoned(Box::new(zi))), "zoned")
    } else {
        (CellInput::Literal(WbValue::Text(value.to_owned())), "text")
    };

    wb.set(sheet, addr, input).map_err(|e| e.to_string())?;
    Ok(json!({
        "ok": true,
        "accepted": { "sheet": sheet, "cell": addr.to_a1(), "as": read_as }
    }))
}

fn tool_workbook_get(args: &JsonValue, store: &mut SessionStore) -> Result<JsonValue, String> {
    let workbook_id = args["workbook_id"].as_str().ok_or("missing workbook_id")?;
    let sheet = args["sheet"].as_str().ok_or("missing sheet")?;
    let cell = args["cell"].as_str().ok_or("missing cell")?;

    let wb = store
        .get_mut(workbook_id)
        .ok_or_else(|| format!("workbook not found: {}", workbook_id))?;

    let addr = Address::from_a1(&cell.to_uppercase())
        .ok_or_else(|| format!("invalid cell address: {}", cell))?;

    if wb.sheet(sheet).is_none() {
        return Err(format!("sheet not found: {}", sheet));
    }

    // `resolved` returns `None` for a genuinely empty address (never authored,
    // not covered by a spill) once the sheet itself is known to exist — that is
    // a normal empty read, not an invalid one, and must return the same shape
    // recalc/formula-indirection already return for an empty value.
    let mut out = match wb.resolved(sheet, addr) {
        Some(resolved) => wb_value_to_json(&resolved.value),
        None => wb_value_to_json(&WbValue::Empty),
    };
    out["accepted"] = json!({ "sheet": sheet, "cell": addr.to_a1() });
    Ok(out)
}

fn tool_workbook_recalc(args: &JsonValue, store: &mut SessionStore) -> Result<JsonValue, String> {
    let workbook_id = args["workbook_id"].as_str().ok_or("missing workbook_id")?;

    // Defaulted, a value we cannot read answers for a context the caller never
    // asked for: NOW() at the epoch, RAND() from seed 0.
    let timestamp_ms = optional_int(args, "timestamp_ms", "a whole number of milliseconds", JsonValue::as_i64)?.unwrap_or(0);
    let timezone = optional_str(args, "timezone")?.unwrap_or("UTC");
    let rng_seed = optional_int(args, "rng_seed", "a non-negative integer", JsonValue::as_u64)?.unwrap_or(0);

    let ctx = RecalcContext::new(timestamp_ms, timezone, rng_seed)
        .ok_or_else(|| format!("unknown timezone: {}", timezone))?;

    let wb = store
        .get_mut(workbook_id)
        .ok_or_else(|| format!("workbook not found: {}", workbook_id))?;

    let changes = wb.recalc(&ctx);
    let change_list: Vec<JsonValue> = changes
        .iter()
        .map(|c| json!({
            "sheet": c.sheet,
            "cell": c.addr.to_a1(),
            "before": wb_value_to_json(&c.old),
            "after": wb_value_to_json(&c.new),
        }))
        .collect();
    Ok(json!({ "changes": change_list }))
}

fn tool_workbook_export(args: &JsonValue, store: &mut SessionStore) -> Result<JsonValue, String> {
    let workbook_id = args["workbook_id"].as_str().ok_or("missing workbook_id")?;

    let wb = store
        .get_mut(workbook_id)
        .ok_or_else(|| format!("workbook not found: {}", workbook_id))?;

    wb.to_json().map(|s| json!({ "json": s })).map_err(|e| e.to_string())
}

fn tool_workbook_import(args: &JsonValue, store: &mut SessionStore) -> Result<JsonValue, String> {
    let json_str = args["json"].as_str().ok_or("missing json")?;

    store.import(json_str).map(|id| json!({ "workbook_id": id }))
}

// ─── Helpers ──────────────────────────────────────────────────────────────────

fn wb_value_to_json(v: &WbValue) -> JsonValue {
    match v {
        WbValue::Number(n) => json!({ "type": "number", "value": n }),
        WbValue::Text(s) => json!({ "type": "text", "value": s }),
        WbValue::Boolean(b) => json!({ "type": "boolean", "value": b }),
        WbValue::Error(e) => json!({ "type": "error", "value": e }),
        WbValue::ErrorMsg(e, m) => json!({ "type": "error", "value": e, "message": m }),
        WbValue::Empty => json!({ "type": "empty", "value": null }),
        WbValue::Date(d) => json!({ "type": "date", "value": d }),
        WbValue::Zoned(z) => json!({ "type": "zoned", "value": z.to_rfc9557() }),
        WbValue::Sparkline(spec) => json!({ "type": "sparkline", "value": sparkline_to_json(spec) }),
        WbValue::Array(rows) => {
            let arr: Vec<Vec<JsonValue>> = rows.iter()
                .map(|r| r.iter().map(wb_value_to_json).collect())
                .collect();
            json!({ "type": "array", "value": arr })
        }
    }
}

/// One sparkline data point / option value, read back from the shape
/// [`sparkline_to_json`] emits for it.
fn json_to_sparkline_value(v: &JsonValue) -> Option<SparklineValue> {
    let obj = v.as_object()?;
    match obj.get("type")?.as_str()? {
        "number" => Some(SparklineValue::number(obj.get("value")?.as_f64()?)),
        "text" => Some(SparklineValue::Text(obj.get("value")?.as_str()?.to_owned())),
        "bool" => Some(SparklineValue::Bool(obj.get("value")?.as_bool()?)),
        "empty" => Some(SparklineValue::Blank),
        _ => None,
    }
}

/// Read the payload of a `{ "type": "sparkline", "value": {...} }` object back
/// into a spec, so a sparkline this server emitted can be handed back as a
/// variable unchanged. Without it the object would silently arrive as `empty`.
fn json_to_sparkline(spec: &JsonValue) -> Option<SparklineSpec> {
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

/// Decode one variable binding, or say why it is not one.
///
/// Every rejection must reach the caller. Dropping a binding leaves its name
/// unbound, and an unbound name evaluates to empty rather than raising, so the
/// caller is handed a plausible wrong number instead of an error.
fn parse_variable(v: &JsonValue) -> Result<Value, String> {
    match v {
        JsonValue::Number(n) => n
            .as_f64()
            .map(Value::Number)
            .ok_or_else(|| "number is not representable".to_owned()),
        JsonValue::String(s) => Ok(Value::Text(s.clone())),
        JsonValue::Bool(b) => Ok(Value::Bool(*b)),
        // Self-describing zoned instant: { "type": "zoned", "value": "<RFC-9557>" }.
        JsonValue::Object(o) if o.get("type").and_then(|t| t.as_str()) == Some("zoned") => o
            .get("value")
            .and_then(|x| x.as_str())
            .and_then(truecalc_core::types::zoned::parse_rfc9557)
            .map(|zi| Value::Zoned(Box::new(zi)))
            .ok_or_else(|| "not a valid RFC-9557 zoned instant".to_owned()),
        // Self-describing sparkline: { "type": "sparkline", "value": {...} }.
        JsonValue::Object(o) if o.get("type").and_then(|t| t.as_str()) == Some("sparkline") => o
            .get("value")
            .and_then(json_to_sparkline)
            .map(|spec| Value::Sparkline(Box::new(spec)))
            .ok_or_else(|| "not a sparkline this server can emit".to_owned()),
        // Elements decode by these same rules, so an array is not limited to
        // numbers and a bad element is reported rather than dropped.
        JsonValue::Array(items) => items
            .iter()
            .map(parse_variable)
            .collect::<Result<Vec<_>, _>>()
            .map(Value::Array),
        _ => Err("expected a number, string, boolean, or array".to_owned()),
    }
}

fn parse_variables(vars_json: &JsonValue) -> Result<HashMap<String, Value>, String> {
    let obj = match vars_json {
        JsonValue::Null => return Ok(HashMap::new()),
        JsonValue::Object(obj) => obj,
        // Ignored, this leaves every name unbound, and an unbound name
        // evaluates to empty rather than raising.
        other => return Err(format!("\"variables\" must be an object of name → value, got {other}")),
    };
    let mut map = HashMap::new();
    for (k, v) in obj {
        let val = parse_variable(v)
            .map_err(|why| format!("unsupported variable binding for \"{k}\": {why}"))?;
        map.insert(k.clone(), val);
    }
    Ok(map)
}

/// The plain-value projection of a sparkline data point / option value, so a
/// spec is emitted in the same vocabulary as any other value.
fn sparkline_cell(v: &SparklineValue) -> Value {
    match v {
        SparklineValue::Number(n) => Value::Number(*n),
        SparklineValue::Text(s) => Value::Text(s.clone()),
        SparklineValue::Bool(b) => Value::Bool(*b),
        SparklineValue::Blank => Value::Empty,
    }
}

/// A sparkline's parsed spec, carried in full: it is the value's identity, so
/// no surface projects it to text (every text projection of it is empty).
fn sparkline_to_json(spec: &SparklineSpec) -> JsonValue {
    let data: Vec<JsonValue> = spec.data.iter().map(|d| value_to_json(&sparkline_cell(d))).collect();
    let options: Vec<JsonValue> = spec
        .options
        .iter()
        .map(|(k, v)| json!([k, value_to_json(&sparkline_cell(v))]))
        .collect();
    json!({ "charttype": spec.chart_type.as_str(), "data": data, "options": options })
}

fn value_to_json(v: &Value) -> JsonValue {
    match v {
        Value::Number(n) | Value::Date(n) => json!({ "value": n, "type": "number" }),
        Value::Text(s) => json!({ "value": s, "type": "text" }),
        Value::Bool(b) => json!({ "value": b, "type": "bool" }),
        Value::Empty => json!({ "value": null, "type": "empty" }),
        Value::Error(e) => json!({ "value": e.to_string(), "type": "error" }),
        Value::ErrorMsg(e, m) => json!({ "value": e.to_string(), "type": "error", "message": m }),
        // Self-describing RFC-9557; deliberately NOT collapsed to "number".
        Value::Zoned(z) => json!({ "value": z.to_rfc9557(), "type": "zoned" }),
        Value::Array(arr) => {
            let items: Vec<JsonValue> = arr.iter().map(value_to_json).collect();
            json!({ "value": items, "type": "array" })
        }
        Value::Sparkline(spec) => json!({ "value": sparkline_to_json(spec), "type": "sparkline" }),
    }
}

fn collect_functions(expr: &Expr, out: &mut Vec<String>) {
    match expr {
        Expr::FunctionCall { name, args, .. } => {
            out.push(name.clone());
            for arg in args {
                collect_functions(arg, out);
            }
        }
        Expr::UnaryOp { operand, .. } => collect_functions(operand, out),
        Expr::BinaryOp { left, right, .. } => {
            collect_functions(left, out);
            collect_functions(right, out);
        }
        _ => {}
    }
}

// ─── tools/list metadata ─────────────────────────────────────────────────────

fn tools_list() -> JsonValue {
    json!([
        {
            "name": "evaluate",
            "description": "Evaluate a spreadsheet formula with optional variable bindings.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "formula": { "type": "string", "description": "Formula string, e.g. \"SUM(A,B)\"" },
                    "variables": { "type": "object", "description": "Variable bindings (name → number/string/bool, or an array of those)" },
                    "conformance": { "type": "string", "description": "Conformance target (default: server default). Supported: \"google-sheets\"" }
                },
                "required": ["formula"]
            }
        },
        {
            "name": "validate",
            "description": "Check whether a formula parses without errors.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "formula": { "type": "string" }
                },
                "required": ["formula"]
            }
        },
        {
            "name": "explain",
            "description": "Describe a formula and list the functions it uses.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "formula": { "type": "string" }
                },
                "required": ["formula"]
            }
        },
        {
            "name": "batch_evaluate",
            "description": "Evaluate multiple formulas sharing the same variable bindings.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "formulas": { "type": "array", "items": { "type": "string" } },
                    "variables": { "type": "object" },
                    "conformance": { "type": "string", "description": "Conformance target (default: server default). Supported: \"google-sheets\"" }
                },
                "required": ["formulas"]
            }
        },
        {
            "name": "list_functions",
            "description": "Return supported spreadsheet functions. Prefer a filter: the unfiltered catalogue is ~70 KB (~17.5k tokens). Filters combine with AND; a filtered call returns at most 100 entries unless \"limit\" says otherwise, and \"total_matched\" always reports how many matched.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "category": { "type": "string", "description": "Exact category, e.g. \"lookup\" (see get_stats for the categories and their sizes)" },
                    "name_contains": { "type": "string", "description": "Case-insensitive substring of the function name, e.g. \"lookup\"" },
                    "names": { "type": "array", "items": { "type": "string" }, "description": "Exact function names to look up in one call; any that do not exist come back in \"not_found\"" },
                    "limit": { "type": "integer", "description": "Maximum entries to return (default: 100 for a filtered call, uncapped otherwise)" }
                }
            }
        },
        {
            "name": "get_stats",
            "description": "Return the total number of supported functions, the library version, and a per-category breakdown.",
            "inputSchema": { "type": "object", "properties": {} }
        },
        {
            "name": "workbook_create",
            "description": "Create a new in-memory workbook with one default sheet named 'Sheet1', ready for workbook_set. Engine is locked at creation.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "engine": { "type": "string", "enum": ["sheets", "excel"], "description": "Spreadsheet dialect" }
                },
                "required": ["engine"]
            }
        },
        {
            "name": "workbook_set",
            "description": "Write a value or formula to a cell in an existing workbook sheet.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "workbook_id": { "type": "string", "description": "Workbook session ID" },
                    "sheet": { "type": "string", "description": "Sheet name" },
                    "cell": { "type": "string", "description": "Cell address in A1 notation" },
                    "value": { "type": "string", "description": "Cell value; prefix with '=' for a formula" }
                },
                "required": ["workbook_id", "sheet", "cell", "value"]
            }
        },
        {
            "name": "workbook_get",
            "description": "Read the effective value of a cell (resolves spills).",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "workbook_id": { "type": "string" },
                    "sheet": { "type": "string" },
                    "cell": { "type": "string", "description": "Cell address in A1 notation" }
                },
                "required": ["workbook_id", "sheet", "cell"]
            }
        },
        {
            "name": "workbook_recalc",
            "description": "Recalculate all formula cells and return the list of changed cells.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "workbook_id": { "type": "string" },
                    "timestamp_ms": { "type": "integer", "description": "UTC epoch milliseconds for NOW()/TODAY() (default: 0)" },
                    "timezone": { "type": "string", "description": "IANA timezone name (default: UTC)" },
                    "rng_seed": { "type": "integer", "description": "RNG seed for RAND() etc. (default: 0)" }
                },
                "required": ["workbook_id"]
            }
        },
        {
            "name": "workbook_export",
            "description": "Export a workbook session as canonical JSON.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "workbook_id": { "type": "string" }
                },
                "required": ["workbook_id"]
            }
        },
        {
            "name": "workbook_import",
            "description": "Import a workbook from canonical JSON and return a new session ID. Prefer workbook_create for a fresh workbook (it already seeds a 'Sheet1'); use workbook_import only to load pre-existing workbook JSON, e.g. from workbook_export. Canonical shape: a JSON object with required fields \"version\" (string, e.g. \"1\" — NOT an integer), \"engine\" (\"sheets\" or \"excel\"), \"names\" (array, required — use [] if there are no named ranges) and \"sheets\" (array of {\"name\": string, \"cells\": object}). Each entry in a sheet's \"cells\" object maps an A1 address to a cell object with a required \"value\" field shaped {\"type\": ..., \"value\": ...} (type is one of \"number\", \"text\", \"boolean\", \"empty\", \"error\", etc.) and an optional \"formula\" string. Minimal known-good example: {\"version\":\"1\",\"engine\":\"sheets\",\"names\":[],\"sheets\":[{\"name\":\"Sheet1\",\"cells\":{\"A1\":{\"value\":{\"type\":\"number\",\"value\":42}}}}]}",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "json": { "type": "string", "description": "Canonical workbook JSON" }
                },
                "required": ["json"]
            }
        }
    ])
}
