use crate::eval::coercion::to_number;
use crate::eval::functions::check_arity;
use crate::types::{ErrorKind, Value};

// Style 0: classic Roman numerals
const STYLE0: &[(i32, &str)] = &[
    (1000, "M"),
    (900,  "CM"),
    (500,  "D"),
    (400,  "CD"),
    (100,  "C"),
    (90,   "XC"),
    (50,   "L"),
    (40,   "XL"),
    (10,   "X"),
    (9,    "IX"),
    (5,    "V"),
    (4,    "IV"),
    (1,    "I"),
];

// Style 1: slightly more concise
const STYLE1: &[(i32, &str)] = &[
    (1000, "M"),
    (950,  "LM"),
    (900,  "CM"),
    (500,  "D"),
    (450,  "LD"),
    (400,  "CD"),
    (100,  "C"),
    (90,   "XC"),
    (50,   "L"),
    (45,   "VL"),
    (40,   "XL"),
    (10,   "X"),
    (9,    "IX"),
    (5,    "V"),
    (4,    "IV"),
    (1,    "I"),
];

// Style 2: more concise
const STYLE2: &[(i32, &str)] = &[
    (1000, "M"),
    (995,  "VM"),
    (950,  "LM"),
    (900,  "CM"),
    (500,  "D"),
    (490,  "XD"),
    (450,  "LD"),
    (400,  "CD"),
    (100,  "C"),
    (95,   "VC"),
    (90,   "XC"),
    (50,   "L"),
    (45,   "VL"),
    (40,   "XL"),
    (10,   "X"),
    (9,    "IX"),
    (5,    "V"),
    (4,    "IV"),
    (1,    "I"),
];

// Style 3: even more concise
const STYLE3: &[(i32, &str)] = &[
    (1000, "M"),
    (999,  "IM"),
    (995,  "VM"),
    (990,  "XM"),
    (950,  "LM"),
    (900,  "CM"),
    (500,  "D"),
    (499,  "ID"),
    (495,  "VD"),
    (490,  "XD"),
    (450,  "LD"),
    (400,  "CD"),
    (100,  "C"),
    (99,   "IC"),
    (95,   "VC"),
    (90,   "XC"),
    (50,   "L"),
    (49,   "IL"),
    (45,   "VL"),
    (40,   "XL"),
    (10,   "X"),
    (9,    "IX"),
    (5,    "V"),
    (4,    "IV"),
    (1,    "I"),
];

// Style 4: most simplified / very concise
const STYLE4: &[(i32, &str)] = &[
    (1000, "M"),
    (999,  "IM"),
    (995,  "VM"),
    (990,  "XM"),
    (950,  "LM"),
    (900,  "CM"),
    (500,  "D"),
    (499,  "ID"),
    (495,  "VD"),
    (490,  "XD"),
    (450,  "LD"),
    (400,  "CD"),
    (100,  "C"),
    (99,   "IC"),
    (95,   "VC"),
    (90,   "XC"),
    (50,   "L"),
    (49,   "IL"),
    (45,   "VL"),
    (40,   "XL"),
    (10,   "X"),
    (9,    "IX"),
    (5,    "V"),
    (4,    "IV"),
    (1,    "I"),
];

/// `ROMAN(number, [style])` — converts an integer (1–3999) to a Roman numeral string.
/// style 0 = classic; 1–4 = progressively more simplified.
/// Returns `#VALUE!` for 0, negative, or out-of-range values.
pub fn roman_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 1, 2) {
        return err;
    }
    let n = match to_number(args[0].clone()) {
        Ok(v) => v,
        Err(e) => return e,
    };
    let n = n as i32;
    if n <= 0 || n > 3999 {
        return Value::Error(ErrorKind::Value);
    }
    let style = if args.len() == 2 {
        match to_number(args[1].clone()) {
            Ok(v) => v as i32,
            Err(e) => return e,
        }
    } else {
        0
    };
    if !(0..=4).contains(&style) {
        return Value::Error(ErrorKind::Value);
    }
    let table = match style {
        0 => STYLE0,
        1 => STYLE1,
        2 => STYLE2,
        3 => STYLE3,
        _ => STYLE4,
    };
    Value::Text(to_roman(n, table))
}

fn to_roman(mut n: i32, table: &[(i32, &str)]) -> String {
    let mut result = String::new();
    for &(val, sym) in table {
        while n >= val {
            result.push_str(sym);
            n -= val;
        }
    }
    result
}

#[cfg(test)]
mod tests;
