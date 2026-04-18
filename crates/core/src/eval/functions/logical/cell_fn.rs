use crate::eval::{evaluate_expr, functions::EvalCtx};
use crate::parser::ast::Expr;
use crate::types::{ErrorKind, Value};

// Source: https://support.google.com/docs/answer/3267071
const VALID_INFO_TYPES: &[&str] = &[
    "address", "col", "color", "contents", "prefix", "row", "sheet", "type", "width",
];

pub fn cell_fn(args: &[Expr], ctx: &mut EvalCtx<'_>) -> Value {
    // arity checks
    if args.is_empty() || args.len() > 2 {
        return Value::Error(ErrorKind::NA);
    }
    if args.len() == 1 {
        return Value::Error(ErrorKind::NA); // no ref context
    }
    // validate info_type
    let info_type_val = evaluate_expr(&args[0], ctx);
    let info_type = match info_type_val {
        Value::Text(ref s) if !s.is_empty() => s.to_lowercase(),
        Value::Text(_) => return Value::Error(ErrorKind::Value), // empty string
        _ => return Value::Error(ErrorKind::Value),              // not text
    };
    if !VALID_INFO_TYPES.contains(&info_type.as_str()) {
        return Value::Error(ErrorKind::Value);
    }
    // For valid calls: return basic info based on the evaluated value of arg[1]
    let ref_val = evaluate_expr(&args[1], ctx);
    match info_type.as_str() {
        "type" => Value::Text(match &ref_val {
            Value::Empty => "b".to_string(),
            Value::Text(_) => "l".to_string(),
            _ => "v".to_string(),
        }),
        "contents" => ref_val,
        "col"   => Value::Number(1.0),
        "row"   => Value::Number(1.0),
        "color" => Value::Number(0.0),
        "width" => Value::Number(8.0),
        "sheet" => Value::Text(String::new()), // no sheet context available
        _ => Value::Text(String::new()),
    }
}
