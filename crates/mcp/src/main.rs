// ganit MCP server — hand-rolled JSON-RPC over stdio (MCP protocol 2024-11-05)

use std::collections::HashMap;
use std::io::{self, BufRead, Write};

use ganit_core::{evaluate, parse, validate, Expr, Value};
use serde_json::{json, Value as JsonValue};

fn main() {
    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut out = stdout.lock();

    for line in stdin.lock().lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => break, // EOF or broken pipe — exit cleanly
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

        // Notifications have no "id"; respond only to requests.
        if request.get("id").is_none() {
            continue;
        }

        let response = handle_request(&request);
        let mut response_str = serde_json::to_string(&response).expect("serialisation error");
        response_str.push('\n');
        out.write_all(response_str.as_bytes()).expect("stdout write error");
        out.flush().expect("stdout flush error");
    }
}

fn handle_request(req: &JsonValue) -> JsonValue {
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
                "serverInfo": { "name": "ganit-mcp", "version": "0.1.0" }
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
            let result = dispatch_tool(name, args);
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

fn dispatch_tool(name: &str, args: &JsonValue) -> JsonValue {
    match name {
        "evaluate" => tool_evaluate(args),
        "validate" => tool_validate(args),
        "explain" => tool_explain(args),
        "batch_evaluate" => tool_batch_evaluate(args),
        "list_functions" => tool_list_functions(),
        _ => json!({ "error": format!("Unknown tool: {}", name) }),
    }
}

// ─── Individual tools ─────────────────────────────────────────────────────────

fn tool_evaluate(args: &JsonValue) -> JsonValue {
    let formula = match args["formula"].as_str() {
        Some(f) => f,
        None => return json!({ "error": "missing formula" }),
    };
    let vars = parse_variables(&args["variables"]);
    let value = evaluate(formula, vars);
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

fn tool_batch_evaluate(args: &JsonValue) -> JsonValue {
    let formulas = match args["formulas"].as_array() {
        Some(a) => a,
        None => return json!({ "error": "missing formulas array" }),
    };
    let vars = parse_variables(&args["variables"]);
    let results: Vec<JsonValue> = formulas
        .iter()
        .map(|f| {
            let formula = f.as_str().unwrap_or("");
            let value = evaluate(formula, vars.clone());
            value_to_json(&value)
        })
        .collect();
    json!(results)
}

fn tool_list_functions() -> JsonValue {
    let entries: Vec<JsonValue> = FUNCTIONS
        .iter()
        .map(|(name, category, syntax, description)| {
            json!({
                "name": name,
                "category": category,
                "syntax": syntax,
                "description": description
            })
        })
        .collect();
    json!({ "functions": entries })
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
        Value::Number(n) => json!({ "value": n, "type": "number" }),
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
                    "variables": { "type": "object", "description": "Variable bindings (name → number/string/bool)" }
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
                    "variables": { "type": "object" }
                },
                "required": ["formulas"]
            }
        },
        {
            "name": "list_functions",
            "description": "Return the catalogue of supported spreadsheet functions.",
            "inputSchema": { "type": "object", "properties": {} }
        }
    ])
}

// ─── Function catalogue (58 functions) ───────────────────────────────────────

static FUNCTIONS: &[(&str, &str, &str, &str)] = &[
    // math (18)
    ("ABS",        "math",        "ABS(number)",                          "Absolute value of a number"),
    ("CEILING",    "math",        "CEILING(number, significance)",         "Round up to nearest multiple of significance"),
    ("FLOOR",      "math",        "FLOOR(number, significance)",           "Round down to nearest multiple of significance"),
    ("INT",        "math",        "INT(number)",                           "Round down to nearest integer"),
    ("LN",         "math",        "LN(number)",                            "Natural logarithm"),
    ("LOG",        "math",        "LOG(number, base)",                     "Logarithm to specified base"),
    ("LOG10",      "math",        "LOG10(number)",                         "Base-10 logarithm"),
    ("MOD",        "math",        "MOD(number, divisor)",                  "Remainder after division"),
    ("POWER",      "math",        "POWER(number, power)",                  "Number raised to a power"),
    ("PRODUCT",    "math",        "PRODUCT(value1,...)",                   "Product of arguments"),
    ("ROUND",      "math",        "ROUND(number, digits)",                 "Round to specified decimal places"),
    ("ROUNDDOWN",  "math",        "ROUNDDOWN(number, digits)",             "Round toward zero"),
    ("ROUNDUP",    "math",        "ROUNDUP(number, digits)",               "Round away from zero"),
    ("SIGN",       "math",        "SIGN(number)",                          "Sign of a number: -1, 0, or 1"),
    ("SQRT",       "math",        "SQRT(number)",                          "Square root"),
    ("SUM",        "math",        "SUM(value1,...)",                       "Sum of arguments"),
    ("SUMIF",      "math",        "SUMIF(range, criteria, sum_range)",     "Sum cells matching a condition"),
    ("TRUNC",      "math",        "TRUNC(number, digits)",                 "Truncate to integer or decimal places"),
    // logical (11)
    ("AND",        "logical",     "AND(value1,...)",                       "True if all arguments are true"),
    ("FALSE",      "logical",     "FALSE()",                               "Logical false value"),
    ("IF",         "logical",     "IF(condition, true_val, false_val)",    "Conditional value"),
    ("IFERROR",    "logical",     "IFERROR(value, value_if_error)",        "Return alternate value on error"),
    ("IFNA",       "logical",     "IFNA(value, value_if_na)",              "Return alternate value on #N/A"),
    ("IFS",        "logical",     "IFS(cond1, val1,...)",                  "First value whose condition is true"),
    ("NOT",        "logical",     "NOT(value)",                            "Logical negation"),
    ("OR",         "logical",     "OR(value1,...)",                        "True if any argument is true"),
    ("SWITCH",     "logical",     "SWITCH(expr, val1, result1,...)",       "Match expression against values"),
    ("TRUE",       "logical",     "TRUE()",                                "Logical true value"),
    ("XOR",        "logical",     "XOR(value1,...)",                       "True if an odd number of arguments are true"),
    // text (21)
    ("CHAR",       "text",        "CHAR(number)",                          "Character from ASCII/Unicode code"),
    ("CLEAN",      "text",        "CLEAN(text)",                           "Remove non-printable characters"),
    ("CODE",       "text",        "CODE(text)",                            "Numeric code of first character"),
    ("CONCAT",     "text",        "CONCAT(value1,...)",                    "Concatenate values"),
    ("CONCATENATE","text",        "CONCATENATE(value1,...)",               "Concatenate values (legacy)"),
    ("EXACT",      "text",        "EXACT(text1, text2)",                   "Case-sensitive string comparison"),
    ("FIND",       "text",        "FIND(find_text, within_text, start)",   "Case-sensitive position search"),
    ("LEFT",       "text",        "LEFT(text, num_chars)",                 "Left portion of a string"),
    ("LEN",        "text",        "LEN(text)",                             "Number of characters in text"),
    ("LOWER",      "text",        "LOWER(text)",                           "Convert to lowercase"),
    ("MID",        "text",        "MID(text, start, num_chars)",           "Substring from middle of text"),
    ("PROPER",     "text",        "PROPER(text)",                          "Capitalise first letter of each word"),
    ("REPLACE",    "text",        "REPLACE(text, start, num_chars, new_text)", "Replace portion of text"),
    ("REPT",       "text",        "REPT(text, number_times)",              "Repeat text N times"),
    ("RIGHT",      "text",        "RIGHT(text, num_chars)",                "Right portion of a string"),
    ("SEARCH",     "text",        "SEARCH(find_text, within_text, start)", "Case-insensitive position search"),
    ("SUBSTITUTE", "text",        "SUBSTITUTE(text, old, new, instance)",  "Replace occurrences of a substring"),
    ("TEXT",       "text",        "TEXT(value, format)",                   "Format number as text"),
    ("TRIM",       "text",        "TRIM(text)",                            "Remove extra whitespace"),
    ("UPPER",      "text",        "UPPER(text)",                           "Convert to uppercase"),
    ("VALUE",      "text",        "VALUE(text)",                           "Convert text to number"),
    // statistical (7)
    ("AVERAGE",    "statistical", "AVERAGE(value1,...)",                   "Arithmetic mean of arguments"),
    ("AVERAGEIF",  "statistical", "AVERAGEIF(range, criteria, avg_range)", "Average of cells matching a condition"),
    ("COUNT",      "statistical", "COUNT(value1,...)",                     "Count numeric values"),
    ("COUNTA",     "statistical", "COUNTA(value1,...)",                    "Count non-empty values"),
    ("COUNTBLANK", "statistical", "COUNTBLANK(range)",                     "Count empty cells"),
    ("COUNTIF",    "statistical", "COUNTIF(range, criteria)",              "Count cells matching a condition"),
    ("MAX",        "statistical", "MAX(value1,...)",                       "Maximum value"),
    ("MIN",        "statistical", "MIN(value1,...)",                       "Minimum value"),
    // financial (1)
    ("PMT",        "financial",   "PMT(rate, nper, pv)",                   "Periodic payment for a loan"),
];
