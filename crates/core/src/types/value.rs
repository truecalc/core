use super::error::ErrorKind;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Number(f64),
    Text(String),
    Bool(bool),
    Error(ErrorKind),
    Empty,
    Array(Vec<Value>),
}
