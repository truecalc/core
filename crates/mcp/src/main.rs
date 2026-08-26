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
            .expect("fresh workbook accepts its first sheet");
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
            let result = dispatch_tool(name, args, default_conformance, engines, store);
            let is_error = result.get("error").is_some();
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

fn dispatch_tool(name: &str, args: &JsonValue, default_conformance: &str, engines: &Engines, store: &mut SessionStore) -> JsonValue {
    match name {
        "evaluate" => tool_evaluate(args, default_conformance, engines),
        "validate" => tool_validate(args, engines),
        "explain" => tool_explain(args, engines),
        "batch_evaluate" => tool_batch_evaluate(args, default_conformance, engines),
        "list_functions" => tool_list_functions(),
        "get_stats" => tool_get_stats(),
        "workbook_create" => tool_workbook_create(args, store),
        "workbook_set" => tool_workbook_set(args, store),
        "workbook_get" => tool_workbook_get(args, store),
        "workbook_recalc" => tool_workbook_recalc(args, store),
        "workbook_export" => tool_workbook_export(args, store),
        "workbook_import" => tool_workbook_import(args, store),
        _ => json!({ "error": format!("Unknown tool: {}", name) }),
    }
}

// ─── Individual tools ─────────────────────────────────────────────────────────

fn tool_evaluate(args: &JsonValue, default_conformance: &str, engines: &Engines) -> JsonValue {
    let formula = match args["formula"].as_str() {
        Some(f) => f,
        None => return json!({ "error": "missing formula" }),
    };
    let conformance = args["conformance"].as_str().unwrap_or(default_conformance);
    let engine = match engines.select(conformance) {
        Some(e) => e,
        None => return json!({ "error": format!("Unknown conformance target: '{}'", conformance) }),
    };
    let vars = parse_variables(&args["variables"]);
    let value = engine.evaluate(formula, &vars);
    value_to_json(&value)
}

fn tool_validate(args: &JsonValue, engines: &Engines) -> JsonValue {
    let formula = match args["formula"].as_str() {
        Some(f) => f,
        None => return json!({ "error": "missing formula" }),
    };
    match engines.google_sheets.validate(formula) {
        Ok(_) => json!({ "valid": true }),
        Err(e) => json!({ "valid": false, "error": e.to_string() }),
    }
}

fn tool_explain(args: &JsonValue, engines: &Engines) -> JsonValue {
    let formula = match args["formula"].as_str() {
        Some(f) => f,
        None => return json!({ "error": "missing formula" }),
    };
    match engines.google_sheets.parse(formula) {
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
    }
}

fn tool_batch_evaluate(args: &JsonValue, default_conformance: &str, engines: &Engines) -> JsonValue {
    let formulas = match args["formulas"].as_array() {
        Some(a) => a,
        None => return json!({ "error": "missing formulas array" }),
    };
    let conformance = args["conformance"].as_str().unwrap_or(default_conformance);
    let engine = match engines.select(conformance) {
        Some(e) => e,
        None => return json!({ "error": format!("Unknown conformance target: '{}'", conformance) }),
    };
    let vars = parse_variables(&args["variables"]);
    let results: Vec<JsonValue> = formulas
        .iter()
        .map(|f| {
            let formula = f.as_str().unwrap_or("");
            let value = engine.evaluate(formula, &vars);
            value_to_json(&value)
        })
        .collect();
    json!(results)
}

fn tool_list_functions() -> JsonValue {
    let registry = Registry::new();
    let mut entries: Vec<JsonValue> = registry
        .list_functions()
        .map(|(name, meta)| json!({
            "name": name,
            "category": meta.category,
            "syntax": meta.signature,
            "description": meta.description,
        }))
        .collect();
    entries.sort_by_key(|e| e["name"].as_str().unwrap_or("").to_owned());
    json!({ "functions": entries })
}

fn tool_get_stats() -> JsonValue {
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
    json!({
        "version": env!("CARGO_PKG_VERSION"),
        "total_functions": total,
        "by_category": categories
    })
}

// ─── Workbook tools ───────────────────────────────────────────────────────────

fn tool_workbook_create(args: &JsonValue, store: &mut SessionStore) -> JsonValue {
    let engine_str = match args["engine"].as_str() {
        Some(e) => e,
        None => return json!({ "error": "missing engine" }),
    };
    let engine = match engine_str {
        "sheets" => WbEngine::Sheets,
        "excel" => WbEngine::Excel,
        other => return json!({ "error": format!("unknown engine: {}", other) }),
    };
    match store.create(engine) {
        Ok(id) => json!({ "workbook_id": id }),
        Err(e) => json!({ "error": e }),
    }
}

fn tool_workbook_set(args: &JsonValue, store: &mut SessionStore) -> JsonValue {
    let workbook_id = match args["workbook_id"].as_str() {
        Some(id) => id,
        None => return json!({ "error": "missing workbook_id" }),
    };
    let sheet = match args["sheet"].as_str() {
        Some(s) => s,
        None => return json!({ "error": "missing sheet" }),
    };
    let cell = match args["cell"].as_str() {
        Some(c) => c,
        None => return json!({ "error": "missing cell" }),
    };
    let value = match args["value"].as_str() {
        Some(v) => v,
        None => return json!({ "error": "missing value" }),
    };

    let wb = match store.get_mut(workbook_id) {
        Some(wb) => wb,
        None => return json!({ "error": format!("workbook not found: {}", workbook_id) }),
    };

    let addr = match Address::from_a1(&cell.to_uppercase()) {
        Some(a) => a,
        None => return json!({ "error": format!("invalid cell address: {}", cell) }),
    };

    let input = if value.starts_with('=') {
        CellInput::Formula(value.to_owned())
    } else if value == "TRUE" || value == "true" {
        CellInput::Literal(WbValue::Boolean(true))
    } else if value == "FALSE" || value == "false" {
        CellInput::Literal(WbValue::Boolean(false))
    } else if let Ok(n) = value.parse::<f64>() {
        CellInput::Literal(WbValue::Number(n))
    } else if let Some(zi) = truecalc_core::types::zoned::parse_rfc9557(value) {
        CellInput::Literal(WbValue::Zoned(Box::new(zi)))
    } else {
        CellInput::Literal(WbValue::Text(value.to_owned()))
    };

    match wb.set(sheet, addr, input) {
        Ok(_) => json!({ "ok": true }),
        Err(e) => json!({ "error": e.to_string() }),
    }
}

fn tool_workbook_get(args: &JsonValue, store: &mut SessionStore) -> JsonValue {
    let workbook_id = match args["workbook_id"].as_str() {
        Some(id) => id,
        None => return json!({ "error": "missing workbook_id" }),
    };
    let sheet = match args["sheet"].as_str() {
        Some(s) => s,
        None => return json!({ "error": "missing sheet" }),
    };
    let cell = match args["cell"].as_str() {
        Some(c) => c,
        None => return json!({ "error": "missing cell" }),
    };

    let wb = match store.get_mut(workbook_id) {
        Some(wb) => wb,
        None => return json!({ "error": format!("workbook not found: {}", workbook_id) }),
    };

    let addr = match Address::from_a1(&cell.to_uppercase()) {
        Some(a) => a,
        None => return json!({ "error": format!("invalid cell address: {}", cell) }),
    };

    if wb.sheet(sheet).is_none() {
        return json!({ "error": format!("sheet not found: {}", sheet) });
    }

    // `resolved` returns `None` for a genuinely empty address (never authored,
    // not covered by a spill) once the sheet itself is known to exist — that is
    // a normal empty read, not an invalid one, and must return the same shape
    // recalc/formula-indirection already return for an empty value.
    match wb.resolved(sheet, addr) {
        Some(resolved) => wb_value_to_json(&resolved.value),
        None => wb_value_to_json(&WbValue::Empty),
    }
}

fn tool_workbook_recalc(args: &JsonValue, store: &mut SessionStore) -> JsonValue {
    let workbook_id = match args["workbook_id"].as_str() {
        Some(id) => id,
        None => return json!({ "error": "missing workbook_id" }),
    };

    let timestamp_ms = args["timestamp_ms"].as_i64().unwrap_or(0);
    let timezone = args["timezone"].as_str().unwrap_or("UTC");
    let rng_seed = args["rng_seed"].as_u64().unwrap_or(0);

    let ctx = match RecalcContext::new(timestamp_ms, timezone, rng_seed) {
        Some(c) => c,
        None => return json!({ "error": format!("unknown timezone: {}", timezone) }),
    };

    let wb = match store.get_mut(workbook_id) {
        Some(wb) => wb,
        None => return json!({ "error": format!("workbook not found: {}", workbook_id) }),
    };

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
    json!({ "changes": change_list })
}

fn tool_workbook_export(args: &JsonValue, store: &mut SessionStore) -> JsonValue {
    let workbook_id = match args["workbook_id"].as_str() {
        Some(id) => id,
        None => return json!({ "error": "missing workbook_id" }),
    };

    let wb = match store.get_mut(workbook_id) {
        Some(wb) => wb,
        None => return json!({ "error": format!("workbook not found: {}", workbook_id) }),
    };

    wb.to_json()
        .map(|s| json!({ "json": s }))
        .unwrap_or_else(|e| json!({ "error": e.to_string() }))
}

fn tool_workbook_import(args: &JsonValue, store: &mut SessionStore) -> JsonValue {
    let json_str = match args["json"].as_str() {
        Some(s) => s,
        None => return json!({ "error": "missing json" }),
    };

    match store.import(json_str) {
        Ok(id) => json!({ "workbook_id": id }),
        Err(e) => json!({ "error": e }),
    }
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

fn parse_variables(vars_json: &JsonValue) -> HashMap<String, Value> {
    let mut map = HashMap::new();
    if let Some(obj) = vars_json.as_object() {
        for (k, v) in obj {
            let val = match v {
                JsonValue::Number(n) => {
                    if let Some(f) = n.as_f64() {
                        Value::Number(f)
                    } else {
                        continue;
                    }
                }
                JsonValue::String(s) => Value::Text(s.clone()),
                JsonValue::Bool(b) => Value::Bool(*b),
                // Self-describing zoned instant: { "type": "zoned", "value": "<RFC-9557>" }.
                JsonValue::Object(o)
                    if o.get("type").and_then(|t| t.as_str()) == Some("zoned") =>
                {
                    match o
                        .get("value")
                        .and_then(|x| x.as_str())
                        .and_then(truecalc_core::types::zoned::parse_rfc9557)
                    {
                        Some(zi) => Value::Zoned(Box::new(zi)),
                        None => continue,
                    }
                }
                // Self-describing sparkline: { "type": "sparkline", "value": {...} }.
                JsonValue::Object(o)
                    if o.get("type").and_then(|t| t.as_str()) == Some("sparkline") =>
                {
                    match o.get("value").and_then(json_to_sparkline) {
                        Some(spec) => Value::Sparkline(Box::new(spec)),
                        None => continue,
                    }
                }
                _ => continue,
            };
            map.insert(k.clone(), val);
        }
    }
    map
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
                    "variables": { "type": "object", "description": "Variable bindings (name → number/string/bool)" },
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
            "description": "Return the catalogue of supported spreadsheet functions.",
            "inputSchema": { "type": "object", "properties": {} }
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
