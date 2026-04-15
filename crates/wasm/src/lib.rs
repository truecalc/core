// json! macro depth for the list_functions array (62 entries) exceeds the default 128
#![recursion_limit = "256"]

use std::collections::HashMap;

use serde_json::json;
use wasm_bindgen::prelude::*;

use ganit_core::Value;

/// Convert a JSON value (from JS) into a ganit-core Value.
fn json_to_value(v: &serde_json::Value) -> Value {
    match v {
        serde_json::Value::Number(n) => n
            .as_f64()
            .map(Value::Number)
            .unwrap_or(Value::Error(ganit_core::ErrorKind::Num)),
        serde_json::Value::String(s) => Value::Text(s.clone()),
        serde_json::Value::Bool(b) => Value::Bool(*b),
        serde_json::Value::Null => Value::Empty,
        _ => Value::Empty,
    }
}

/// Convert a ganit-core Value into a JS-friendly serde_json object.
fn value_to_json(v: Value) -> serde_json::Value {
    match v {
        Value::Number(n) => json!({ "value": n, "type": "number" }),
        Value::Text(s) => json!({ "value": s, "type": "text" }),
        Value::Bool(b) => json!({ "value": b, "type": "bool" }),
        Value::Error(e) => json!({ "value": null, "type": "error", "error": e.to_string() }),
        Value::Empty => json!({ "value": null, "type": "empty" }),
        Value::Array(_) => json!({ "value": null, "type": "array" }),
    }
}

/// Evaluate a formula with named variables supplied as a JS object.
///
/// `variables` must be a plain JS object mapping string keys to number/string/bool/null.
/// Passing `undefined` or `null` is safe and is treated as no variables.
/// Returns `{ value, type }` or `{ value: null, type: "error", error: "..." }`.
#[wasm_bindgen]
pub fn evaluate(formula: &str, variables: JsValue) -> JsValue {
    let vars_json: serde_json::Value = serde_wasm_bindgen::from_value(variables)
        .unwrap_or(serde_json::Value::Object(Default::default()));

    let vars: HashMap<String, Value> = match vars_json.as_object() {
        Some(map) => map
            .iter()
            .map(|(k, v)| (k.clone(), json_to_value(v)))
            .collect(),
        None => HashMap::new(),
    };

    let result = ganit_core::evaluate(formula, vars);
    let json_result = value_to_json(result);
    serde_wasm_bindgen::to_value(&json_result).unwrap_or(JsValue::NULL)
}

/// Validate a formula string without evaluating it.
///
/// Returns `{ valid: true }` on success or `{ valid: false, error: "..." }` on failure.
#[wasm_bindgen]
pub fn validate(formula: &str) -> JsValue {
    let result = match ganit_core::validate(formula) {
        Ok(_) => json!({ "valid": true }),
        Err(e) => json!({ "valid": false, "error": e.to_string() }),
    };
    serde_wasm_bindgen::to_value(&result).unwrap_or(JsValue::NULL)
}

/// Return metadata for all built-in functions as a JS array.
///
/// Each entry: `{ name, category, syntax, description }`.
#[wasm_bindgen]
pub fn list_functions() -> JsValue {
    let functions = json!([
        // math
        { "name": "SUM", "category": "math", "syntax": "SUM(value1, ...)", "description": "Sum of all arguments" },
        { "name": "AVERAGE", "category": "math", "syntax": "AVERAGE(value1, ...)", "description": "Arithmetic mean of all arguments" },
        { "name": "PRODUCT", "category": "math", "syntax": "PRODUCT(value1, ...)", "description": "Product of all arguments" },
        { "name": "ROUND", "category": "math", "syntax": "ROUND(number, digits)", "description": "Round to specified decimal places" },
        { "name": "ROUNDUP", "category": "math", "syntax": "ROUNDUP(number, digits)", "description": "Round up away from zero" },
        { "name": "ROUNDDOWN", "category": "math", "syntax": "ROUNDDOWN(number, digits)", "description": "Round down toward zero" },
        { "name": "INT", "category": "math", "syntax": "INT(number)", "description": "Round down to nearest integer" },
        { "name": "ABS", "category": "math", "syntax": "ABS(number)", "description": "Absolute value" },
        { "name": "SIGN", "category": "math", "syntax": "SIGN(number)", "description": "Sign of a number: -1, 0, or 1" },
        { "name": "MOD", "category": "math", "syntax": "MOD(number, divisor)", "description": "Remainder after division" },
        { "name": "POWER", "category": "math", "syntax": "POWER(base, exponent)", "description": "Base raised to exponent" },
        { "name": "SQRT", "category": "math", "syntax": "SQRT(number)", "description": "Square root" },
        { "name": "LOG", "category": "math", "syntax": "LOG(number, [base])", "description": "Logarithm to given base (default 10)" },
        { "name": "LOG10", "category": "math", "syntax": "LOG10(number)", "description": "Base-10 logarithm" },
        { "name": "LN", "category": "math", "syntax": "LN(number)", "description": "Natural logarithm" },
        { "name": "EXP", "category": "math", "syntax": "EXP(number)", "description": "e raised to the given power" },
        { "name": "CEILING", "category": "math", "syntax": "CEILING(number, significance)", "description": "Round up to nearest multiple of significance" },
        { "name": "FLOOR", "category": "math", "syntax": "FLOOR(number, significance)", "description": "Round down to nearest multiple of significance" },
        { "name": "RAND", "category": "math", "syntax": "RAND()", "description": "Random number between 0 and 1" },
        { "name": "RANDBETWEEN", "category": "math", "syntax": "RANDBETWEEN(low, high)", "description": "Random integer between low and high" },
        { "name": "PI", "category": "math", "syntax": "PI()", "description": "Value of pi" },
        { "name": "SIN", "category": "math", "syntax": "SIN(angle)", "description": "Sine of angle in radians" },
        { "name": "COS", "category": "math", "syntax": "COS(angle)", "description": "Cosine of angle in radians" },
        { "name": "TAN", "category": "math", "syntax": "TAN(angle)", "description": "Tangent of angle in radians" },
        { "name": "QUOTIENT", "category": "math", "syntax": "QUOTIENT(numerator, denominator)", "description": "Integer portion of division" },
        // logical
        { "name": "IF", "category": "logical", "syntax": "IF(condition, value_if_true, value_if_false)", "description": "Conditional evaluation" },
        { "name": "AND", "category": "logical", "syntax": "AND(value1, ...)", "description": "True if all arguments are true" },
        { "name": "OR", "category": "logical", "syntax": "OR(value1, ...)", "description": "True if any argument is true" },
        { "name": "NOT", "category": "logical", "syntax": "NOT(value)", "description": "Logical negation" },
        { "name": "IFERROR", "category": "logical", "syntax": "IFERROR(value, value_if_error)", "description": "Return alternate value on error" },
        { "name": "IFNA", "category": "logical", "syntax": "IFNA(value, value_if_na)", "description": "Return alternate value on #N/A" },
        { "name": "IFS", "category": "logical", "syntax": "IFS(cond1, val1, ...)", "description": "First value whose condition is true" },
        { "name": "SWITCH", "category": "logical", "syntax": "SWITCH(expr, case1, val1, ..., [default])", "description": "Match expression against cases" },
        { "name": "ISNUMBER", "category": "logical", "syntax": "ISNUMBER(value)", "description": "True if value is a number" },
        { "name": "ISTEXT", "category": "logical", "syntax": "ISTEXT(value)", "description": "True if value is text" },
        { "name": "ISERROR", "category": "logical", "syntax": "ISERROR(value)", "description": "True if value is any error" },
        { "name": "ISBLANK", "category": "logical", "syntax": "ISBLANK(value)", "description": "True if value is empty" },
        { "name": "ISNA", "category": "logical", "syntax": "ISNA(value)", "description": "True if value is #N/A" },
        // text
        { "name": "LEFT", "category": "text", "syntax": "LEFT(text, [num_chars])", "description": "Leftmost characters from text" },
        { "name": "MID", "category": "text", "syntax": "MID(text, start, num_chars)", "description": "Characters from the middle of text" },
        { "name": "RIGHT", "category": "text", "syntax": "RIGHT(text, [num_chars])", "description": "Rightmost characters from text" },
        { "name": "LEN", "category": "text", "syntax": "LEN(text)", "description": "Length of text in characters" },
        { "name": "LOWER", "category": "text", "syntax": "LOWER(text)", "description": "Convert text to lowercase" },
        { "name": "UPPER", "category": "text", "syntax": "UPPER(text)", "description": "Convert text to uppercase" },
        { "name": "TRIM", "category": "text", "syntax": "TRIM(text)", "description": "Remove leading/trailing spaces" },
        { "name": "CONCATENATE", "category": "text", "syntax": "CONCATENATE(text1, ...)", "description": "Join text strings together" },
        { "name": "FIND", "category": "text", "syntax": "FIND(find_text, within_text, [start])", "description": "Position of substring (case-sensitive)" },
        { "name": "SUBSTITUTE", "category": "text", "syntax": "SUBSTITUTE(text, old, new, [instance])", "description": "Replace occurrences of a substring" },
        { "name": "REPLACE", "category": "text", "syntax": "REPLACE(text, start, num_chars, new_text)", "description": "Replace characters by position" },
        { "name": "TEXT", "category": "text", "syntax": "TEXT(value, format)", "description": "Format a number as text" },
        { "name": "VALUE", "category": "text", "syntax": "VALUE(text)", "description": "Convert text to a number" },
        { "name": "REPT", "category": "text", "syntax": "REPT(text, times)", "description": "Repeat text a given number of times" },
        // financial
        { "name": "PMT", "category": "financial", "syntax": "PMT(rate, nper, pv, [fv], [type])", "description": "Periodic payment for a loan" },
        { "name": "NPV", "category": "financial", "syntax": "NPV(rate, value1, ...)", "description": "Net present value of cash flows" },
        { "name": "IRR", "category": "financial", "syntax": "IRR(values, [guess])", "description": "Internal rate of return" },
        { "name": "PV", "category": "financial", "syntax": "PV(rate, nper, pmt, [fv], [type])", "description": "Present value of an investment" },
        { "name": "FV", "category": "financial", "syntax": "FV(rate, nper, pmt, [pv], [type])", "description": "Future value of an investment" },
        { "name": "RATE", "category": "financial", "syntax": "RATE(nper, pmt, pv, [fv], [type])", "description": "Interest rate per period" },
        { "name": "NPER", "category": "financial", "syntax": "NPER(rate, pmt, pv, [fv], [type])", "description": "Number of periods for an investment" },
        // statistical
        { "name": "COUNT", "category": "statistical", "syntax": "COUNT(value1, ...)", "description": "Count of numeric values" },
        { "name": "COUNTA", "category": "statistical", "syntax": "COUNTA(value1, ...)", "description": "Count of non-empty values" },
        { "name": "MAX", "category": "statistical", "syntax": "MAX(value1, ...)", "description": "Maximum value" },
        { "name": "MIN", "category": "statistical", "syntax": "MIN(value1, ...)", "description": "Minimum value" },
        { "name": "MEDIAN", "category": "statistical", "syntax": "MEDIAN(value1, ...)", "description": "Median value" }
    ]);

    serde_wasm_bindgen::to_value(&functions).unwrap_or(JsValue::NULL)
}
