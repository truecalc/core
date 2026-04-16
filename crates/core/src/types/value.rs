use super::error::ErrorKind;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    /// Finite numeric value. INVARIANT: must never hold NaN or infinity.
    /// Use `Value::Error(ErrorKind::Num)` for non-finite results instead.
    Number(f64),
    Text(String),
    Bool(bool),
    Error(ErrorKind),
    Empty,
    Array(Vec<Value>),
    /// A spreadsheet serial date number — same float encoding as Number but
    /// typed so ISDATE can distinguish it from a plain numeric literal.
    Date(f64),
}
