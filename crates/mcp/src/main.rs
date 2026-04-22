// truecalc MCP server — hand-rolled JSON-RPC over stdio (MCP protocol 2024-11-05)

use std::collections::HashMap;
use std::io::{self, BufRead, Write};

use truecalc_core::{parse, validate, Engine, Expr, Registry, Value};
use serde_json::{json, Value as JsonValue};

// ─── Conformance ─────────────────────────────────────────────────────────────

struct Engines {
    google_sheets: Engine,
}

impl Engines {
    fn new() -> Self {
        Self { google_sheets: Engine::google_sheets() }
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

// ─── Entry point ─────────────────────────────────────────────────────────────

fn main() {
    let cli_args: Vec<String> = std::env::args().collect();
    let default_conformance = parse_conformance_arg(&cli_args);
    let engines = Engines::new();

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

        let response = handle_request(&request, &default_conformance, &engines);
        let mut response_str = serde_json::to_string(&response).expect("serialisation error");
        response_str.push('\n');
        out.write_all(response_str.as_bytes()).expect("stdout write error");
        out.flush().expect("stdout flush error");
    }
}

fn handle_request(req: &JsonValue, default_conformance: &str, engines: &Engines) -> JsonValue {
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
            let result = dispatch_tool(name, args, default_conformance, engines);
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

fn dispatch_tool(name: &str, args: &JsonValue, default_conformance: &str, engines: &Engines) -> JsonValue {
    match name {
        "evaluate" => tool_evaluate(args, default_conformance, engines),
        "validate" => tool_validate(args),
        "explain" => tool_explain(args),
        "batch_evaluate" => tool_batch_evaluate(args, default_conformance, engines),
        "list_functions" => tool_list_functions(),
        "get_stats" => tool_get_stats(),
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

fn tool_validate(args: &JsonValue) -> JsonValue {
    let formula = match args["formula"].as_str() {
        Some(f) => f,
        None => return json!({ "error": "missing formula" }),
    };
    match validate(formula) {
        Ok(_) => json!({ "valid": true }),
        Err(e) => json!({ "valid": false, "error": e.to_string() }),
    }
}

fn tool_explain(args: &JsonValue) -> JsonValue {
    let formula = match args["formula"].as_str() {
        Some(f) => f,
        None => return json!({ "error": "missing formula" }),
    };
    match parse(formula) {
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

// ─── Helpers ──────────────────────────────────────────────────────────────────

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
                _ => continue,
            };
            map.insert(k.clone(), val);
        }
    }
    map
}

fn value_to_json(v: &Value) -> JsonValue {
    match v {
        Value::Number(n) | Value::Date(n) => json!({ "value": n, "type": "number" }),
        Value::Text(s) => json!({ "value": s, "type": "text" }),
        Value::Bool(b) => json!({ "value": b, "type": "bool" }),
        Value::Empty => json!({ "value": null, "type": "empty" }),
        Value::Error(e) => json!({ "value": e.to_string(), "type": "error" }),
        Value::Array(arr) => {
            let items: Vec<JsonValue> = arr.iter().map(value_to_json).collect();
            json!({ "value": items, "type": "array" })
        }
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
        }
    ])
}

