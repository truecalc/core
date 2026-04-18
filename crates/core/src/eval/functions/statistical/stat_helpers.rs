use crate::types::Value;

/// Collect numeric values from args, flattening arrays.
/// Numbers and Dates are included. Bool/Text/Empty are ignored.
pub fn collect_nums(args: &[Value]) -> Vec<f64> {
    let mut nums = Vec::new();
    collect_nums_into(args, &mut nums);
    nums
}

fn collect_nums_into(args: &[Value], out: &mut Vec<f64>) {
    for arg in args {
        match arg {
            Value::Number(n) => out.push(*n),
            Value::Date(n) => out.push(*n),
            Value::Array(inner) => collect_nums_into(inner, out),
            _ => {}
        }
    }
}

/// Collect numeric values from args, flattening arrays.
/// "A" variant: also includes Bool (TRUE=1, FALSE=0) and Text as 0.
pub fn collect_nums_a(args: &[Value]) -> Vec<f64> {
    let mut nums = Vec::new();
    collect_nums_a_into(args, &mut nums);
    nums
}

fn collect_nums_a_into(args: &[Value], out: &mut Vec<f64>) {
    for arg in args {
        match arg {
            Value::Number(n) => out.push(*n),
            Value::Date(n) => out.push(*n),
            Value::Bool(b) => out.push(if *b { 1.0 } else { 0.0 }),
            Value::Text(_) => out.push(0.0),
            Value::Array(inner) => collect_nums_a_into(inner, out),
            Value::Empty => {}
            Value::Error(_) => {}
        }
    }
}
