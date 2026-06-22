use super::error::ErrorKind;
use super::zoned::ZonedInstant;

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
    /// A zone-aware instant (Model B): an absolute UTC instant plus an IANA or
    /// fixed zone. Boxed because the payload carries a ~600-variant `Tz`, and
    /// `Value` is cloned heavily on the hot numeric path. Equality/ordering for
    /// the engine's comparison operators is defined on the instant only.
    Zoned(Box<ZonedInstant>),
}
