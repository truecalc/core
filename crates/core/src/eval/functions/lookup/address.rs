use crate::eval::functions::check_arity;
use crate::types::{ErrorKind, Value};
use super::cell_ref::col_index_to_label;

/// `ADDRESS(row, col, [abs_mode], [a1], [sheet_text])`
/// Returns a cell address string.
/// abs_mode: 1=$A$1, 2=A$1, 3=$A1, 4=A1
/// a1: TRUE=A1 style (default), FALSE=R1C1 style
pub fn address_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 2, 5) {
        return err;
    }

    let row = match &args[0] {
        Value::Number(n) => {
            let n = n.trunc() as i64;
            if n < 1 { return Value::Error(ErrorKind::Value); }
            n as usize
        }
        _ => return Value::Error(ErrorKind::Value),
    };

    let col = match &args[1] {
        Value::Number(n) => {
            let n = n.trunc() as i64;
            if n < 1 { return Value::Error(ErrorKind::Value); }
            n as usize
        }
        _ => return Value::Error(ErrorKind::Value),
    };

    let abs_mode = if args.len() >= 3 {
        match &args[2] {
            Value::Number(n) => n.trunc() as i64,
            _ => return Value::Error(ErrorKind::Value),
        }
    } else {
        1
    };

    let a1_style = if args.len() >= 4 {
        match &args[3] {
            Value::Bool(b) => *b,
            Value::Number(n) => *n != 0.0,
            _ => return Value::Error(ErrorKind::Value),
        }
    } else {
        true
    };

    // Sheet prefix is ignored in standalone evaluator (no multi-sheet support)

    let addr = if a1_style {
        let col_label = col_index_to_label(col);
        match abs_mode {
            1 => format!("${col_label}${row}"),
            2 => format!("{col_label}${row}"),
            3 => format!("${col_label}{row}"),
            4 => format!("{col_label}{row}"),
            _ => return Value::Error(ErrorKind::Value),
        }
    } else {
        // R1C1 style
        match abs_mode {
            1 => format!("R{row}C{col}"),
            2 => format!("R{row}C[{col}]"),
            3 => format!("R[{row}]C{col}"),
            4 => format!("R[{row}]C[{col}]"),
            _ => return Value::Error(ErrorKind::Value),
        }
    };

    Value::Text(addr)
}
