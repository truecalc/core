use super::error::ErrorKind;
use super::sparkline::SparklineSpec;
use super::zoned::ZonedInstant;

#[derive(Debug, Clone)]
pub enum Value {
    /// Finite numeric value. INVARIANT: must never hold NaN or infinity.
    /// Use `Value::Error(ErrorKind::Num)` for non-finite results instead.
    Number(f64),
    Text(String),
    Bool(bool),
    Error(ErrorKind),
    /// An error carrying an optional human-readable diagnostic message
    /// (Google Sheets parity, e.g. the arity message emitted by
    /// `check_arity`). The message is *additive metadata*: it never affects
    /// equality (errors compare by kind — see the hand-written `PartialEq`)
    /// nor the error code string, so conformance output is byte-identical to a
    /// bare `Value::Error(kind)`. Consumers read it via
    /// [`Value::error_message`].
    ErrorMsg(ErrorKind, String),
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
    /// A sparkline: the parsed, validated render spec produced by `SPARKLINE`.
    /// Google Sheets models this as a value kind of its own — `TYPE()` reports
    /// the undocumented code `128` and `ISERROR()` is `FALSE` — so it is
    /// neither text nor an error here either. Boxed for the same reason as
    /// `Zoned`: the payload is large and `Value` is cloned heavily.
    ///
    /// Like [`Value::ErrorMsg`], the payload is excluded from equality: in
    /// Sheets **any** two sparklines compare equal under `=`, whatever they
    /// plot. The spec is still carried in full by every surface, because
    /// `COUNTUNIQUE` does distinguish two different sparklines — Sheets keys
    /// uniqueness off something deeper than `=` compares.
    Sparkline(Box<SparklineSpec>),
}

impl Value {
    /// The [`ErrorKind`] if this value is any error variant (bare `Error` or
    /// message-carrying `ErrorMsg`), else `None`.
    pub fn error_kind(&self) -> Option<&ErrorKind> {
        match self {
            Value::Error(k) | Value::ErrorMsg(k, _) => Some(k),
            _ => None,
        }
    }

    /// True if this value is any error variant.
    pub fn is_error(&self) -> bool {
        matches!(self, Value::Error(_) | Value::ErrorMsg(_, _))
    }

    /// The optional diagnostic message attached to an error value. Bare errors
    /// (and non-errors) return `None`.
    pub fn error_message(&self) -> Option<&str> {
        match self {
            Value::ErrorMsg(_, msg) => Some(msg.as_str()),
            _ => None,
        }
    }
}

/// Hand-written so that the two error variants compare **by kind only** — the
/// diagnostic message is additive metadata and must never change equality
/// (conformance compares evaluated values against ground truth, and a
/// message-carrying error must stay equal to the same bare error code). Every
/// non-error arm matches the previous `#[derive(PartialEq)]` behaviour exactly.
impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Number(a), Value::Number(b)) => a == b,
            (Value::Text(a), Value::Text(b)) => a == b,
            (Value::Bool(a), Value::Bool(b)) => a == b,
            (Value::Empty, Value::Empty) => true,
            (Value::Array(a), Value::Array(b)) => a == b,
            (Value::Date(a), Value::Date(b)) => a == b,
            (Value::Zoned(a), Value::Zoned(b)) => a == b,
            // Any two sparklines are equal, whatever they plot: google.tsv
            // records `=SPARKLINE({1,2,3})=SPARKLINE({9,9,9})` as TRUE, as well
            // as the same row for differing charttypes and options. The spec is
            // still carried (COUNTUNIQUE distinguishes two sparklines with a
            // deeper key than `=` uses) — it just is not what `==` compares.
            (Value::Sparkline(_), Value::Sparkline(_)) => true,
            // Both error variants: equal iff same kind (message ignored).
            _ => match (self.error_kind(), other.error_kind()) {
                (Some(ka), Some(kb)) => ka == kb,
                _ => false,
            },
        }
    }
}
