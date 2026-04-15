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
}
