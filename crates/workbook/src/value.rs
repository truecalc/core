use std::hash::{Hash, Hasher};

use serde::de::Error as _;
use serde::ser::{Error as _, SerializeMap};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use truecalc_core::types::zoned::parse_rfc9557;
use truecalc_core::types::{SparklineChartType, SparklineSpec, SparklineValue, ZonedInstant};

/// An evaluated cell value — one of the seven types of schema spec §6.
///
/// Wire encodings extend the published `@truecalc/core` npm shapes:
/// `{ "type": "number", "value": 1.5 }`, with `error` using an `error` key
/// (`{ "error": "#REF!", "type": "error" }`).
///
/// Invariants (schema spec §6 and §8):
/// - `Number` and `Date` are always finite — NaN and infinity are
///   unrepresentable; the serializer rejects them and the deserializer
///   refuses them. `-0.0` is normalized to `0.0` on deserialization, and
///   equality/hashing treat them as the same value.
/// - `Array` is row-major, rectangular, non-empty, larger than 1×1 (a 1×1
///   array is collapsed to its scalar element before storage, schema spec
///   §6), and holds only scalar values (never a nested `Array`). It appears
///   only as a spill anchor's value (schema spec §5).
#[derive(Debug, Clone)]
pub enum Value {
    /// Finite IEEE-754 f64.
    Number(f64),
    /// Any Unicode string.
    Text(String),
    /// A boolean.
    Boolean(bool),
    /// A spreadsheet error code, e.g. `#REF!`. Allowed codes are the
    /// engine's error set for the workbook's flavor (registry-driven).
    Error(String),
    /// An error code carrying an *additive* diagnostic message (Google Sheets
    /// parity, e.g. the arity message for `DATE()`). The message is in-memory
    /// metadata only: it is **not** part of value identity (equality and
    /// hashing compare by code — see the hand-written `PartialEq`/`Hash`) and
    /// is **dropped** by canonical serialization, so the persisted JSON, the
    /// `to_json ∘ from_json = id` guarantee, and hash/equality are all
    /// byte-for-byte unchanged from a bare `Error(code)`. Consumers read it via
    /// [`Value::error_message`].
    ErrorMsg(String, String),
    /// An evaluated-empty result (a formula cell before first recalc, or a
    /// formula referencing an unauthored cell). Never used to pad the
    /// sparse grid.
    Empty,
    /// Row-major 2-D array of scalar values; a spill anchor's full
    /// evaluated array.
    Array(Vec<Vec<Value>>),
    /// A date as a serial number (fractional part = time of day). The epoch
    /// is implied by the workbook's engine flavor, never stored per-value.
    Date(f64),
    /// A zone-aware instant (Model B). Serialized as its canonical, self-
    /// describing RFC-9557 string, e.g. `2026-07-14T11:00:00+02:00[Europe/Berlin]`.
    Zoned(Box<ZonedInstant>),
    /// A sparkline: the parsed, validated render spec produced by `SPARKLINE`
    /// (Google Sheets models it as a value kind of its own — `TYPE()` reports
    /// the undocumented code `128`).
    ///
    /// Sheets keeps *two* notions of sameness for a sparkline, and this type
    /// carries the deeper one. The `=` operator reports any two sparklines
    /// equal, whatever they plot (that is the engine's
    /// [`truecalc_core::Value`] equality); `COUNTUNIQUE` nonetheless counts two
    /// different sparklines as 2 and two identical ones as 1. Storage needs the
    /// deeper notion: recalc writes a recomputed cell back only when the new
    /// value differs from the old, so if every sparkline compared equal here a
    /// changed chart would silently keep its stale spec. Equality and hashing
    /// therefore compare the whole spec, and canonical JSON carries it in
    /// full — serializing it lossily (as `""`, or by dropping it and
    /// recomputing from the formula) would collapse two genuinely different
    /// sparklines into one canonical form.
    Sparkline(Box<SparklineSpec>),
}

/// Bit pattern of a finite f64 with `-0.0` normalized to `0.0`, so that
/// hashing agrees with `==` (schema spec §8: hash and float equality operate
/// on post-normalization bit patterns; sound because the serializer rejects NaN).
fn normalized_bits(x: f64) -> u64 {
    if x == 0.0 {
        0.0_f64.to_bits()
    } else {
        x.to_bits()
    }
}

impl Value {
    /// The error code if this value is any error variant (bare `Error` or
    /// message-carrying `ErrorMsg`), else `None`.
    fn error_code(&self) -> Option<&str> {
        match self {
            Value::Error(code) | Value::ErrorMsg(code, _) => Some(code.as_str()),
            _ => None,
        }
    }

    /// The additive diagnostic message attached to an error value, if any.
    /// Bare errors (and non-errors) return `None`.
    pub fn error_message(&self) -> Option<&str> {
        match self {
            Value::ErrorMsg(_, msg) => Some(msg.as_str()),
            _ => None,
        }
    }
}

/// Errors compare **by code only** — the diagnostic message is additive
/// metadata and must never affect identity (a message-carrying error stays
/// equal to the same bare error code, so canonical round-tripping and grid
/// deduplication are unchanged). Every non-error arm matches the previous
/// `#[derive(PartialEq)]` behaviour exactly.
impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Number(a), Value::Number(b)) => a == b,
            (Value::Text(a), Value::Text(b)) => a == b,
            (Value::Boolean(a), Value::Boolean(b)) => a == b,
            (Value::Empty, Value::Empty) => true,
            (Value::Array(a), Value::Array(b)) => a == b,
            (Value::Date(a), Value::Date(b)) => a == b,
            (Value::Zoned(a), Value::Zoned(b)) => a == b,
            // Storage identity is the deep (COUNTUNIQUE-grade) one, not the
            // `=` operator's — see the variant's doc comment.
            (Value::Sparkline(a), Value::Sparkline(b)) => a == b,
            _ => match (self.error_code(), other.error_code()) {
                (Some(a), Some(b)) => a == b,
                _ => false,
            },
        }
    }
}

impl Hash for Value {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // Both error variants hash identically (message ignored, fixed tag) so
        // that `Error(code)` and `ErrorMsg(code, _)` — which compare equal —
        // also hash equal, keeping the Hash/Eq contract intact.
        if let Some(code) = self.error_code() {
            "error".hash(state);
            code.hash(state);
            return;
        }
        std::mem::discriminant(self).hash(state);
        match self {
            Value::Number(n) | Value::Date(n) => normalized_bits(*n).hash(state),
            Value::Text(s) => s.hash(state),
            Value::Boolean(b) => b.hash(state),
            // Hash the canonical RFC-9557 form: structurally equal instants
            // (same utc_nanos + zone) produce the same string, agreeing with `==`.
            Value::Zoned(z) => z.to_rfc9557().hash(state),
            Value::Empty => {}
            Value::Array(rows) => {
                rows.len().hash(state);
                for row in rows {
                    row.len().hash(state);
                    for v in row {
                        v.hash(state);
                    }
                }
            }
            // Hash the whole spec: it is this value's identity, so two
            // sparklines that compare equal must hash equal.
            Value::Sparkline(spec) => {
                spec.chart_type.as_str().hash(state);
                spec.data.len().hash(state);
                for point in &spec.data {
                    hash_sparkline_value(point, state);
                }
                spec.options.len().hash(state);
                for (key, value) in &spec.options {
                    key.hash(state);
                    hash_sparkline_value(value, state);
                }
            }
            // Handled above via `error_code()`.
            Value::Error(_) | Value::ErrorMsg(_, _) => unreachable!(),
        }
    }
}

/// A sparkline data point / option value as an ordinary scalar cell value, so
/// a spec serializes in the same vocabulary as every other value on the wire.
fn sparkline_value_to_value(v: &SparklineValue) -> Value {
    match v {
        SparklineValue::Number(n) => Value::Number(if *n == 0.0 { 0.0 } else { *n }),
        SparklineValue::Text(s) => Value::Text(s.clone()),
        SparklineValue::Bool(b) => Value::Boolean(*b),
        SparklineValue::Blank => Value::Empty,
    }
}

/// The inverse of [`sparkline_value_to_value`]; only scalar cell values can be
/// a data point or an option value.
fn value_to_sparkline_value(v: &Value) -> Result<SparklineValue, String> {
    match v {
        Value::Number(n) => Ok(SparklineValue::number(*n)),
        Value::Text(s) => Ok(SparklineValue::Text(s.clone())),
        Value::Boolean(b) => Ok(SparklineValue::Bool(*b)),
        Value::Empty => Ok(SparklineValue::Blank),
        _ => Err(
            "a sparkline data point or option value must be a number, text, boolean or empty"
                .to_string(),
        ),
    }
}

fn hash_sparkline_value<H: Hasher>(v: &SparklineValue, state: &mut H) {
    std::mem::discriminant(v).hash(state);
    match v {
        SparklineValue::Number(n) => normalized_bits(*n).hash(state),
        SparklineValue::Text(s) => s.hash(state),
        SparklineValue::Bool(b) => b.hash(state),
        SparklineValue::Blank => {}
    }
}

/// Canonical wire form of a parsed sparkline spec. Keys are emitted in
/// lexicographic order (`charttype` < `data` < `options`) so the encoding is
/// canonical (JCS) like every other value in this module.
struct SparklineSpecWire<'a>(&'a SparklineSpec);

impl Serialize for SparklineSpecWire<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let data: Vec<Value> = self.0.data.iter().map(sparkline_value_to_value).collect();
        let options: Vec<(&str, Value)> = self
            .0
            .options
            .iter()
            .map(|(k, v)| (k.as_str(), sparkline_value_to_value(v)))
            .collect();
        let mut map = serializer.serialize_map(Some(3))?;
        map.serialize_entry("charttype", self.0.chart_type.as_str())?;
        map.serialize_entry("data", &data)?;
        map.serialize_entry("options", &options)?;
        map.end()
    }
}

fn serialize_tagged_number<S: Serializer>(
    kind: &'static str,
    n: f64,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    if !n.is_finite() {
        return Err(S::Error::custom(format!(
            "non-finite {kind} value cannot be serialized (schema spec §8)"
        )));
    }
    let n = if n == 0.0 { 0.0 } else { n };
    let mut map = serializer.serialize_map(Some(2))?;
    map.serialize_entry("type", kind)?;
    map.serialize_entry("value", &n)?;
    map.end()
}

impl Serialize for Value {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Value::Number(n) => serialize_tagged_number("number", *n, serializer),
            Value::Date(n) => serialize_tagged_number("date", *n, serializer),
            Value::Zoned(z) => {
                let mut map = serializer.serialize_map(Some(2))?;
                map.serialize_entry("type", "zoned")?;
                map.serialize_entry("value", &z.to_rfc9557())?;
                map.end()
            }
            Value::Text(s) => {
                let mut map = serializer.serialize_map(Some(2))?;
                map.serialize_entry("type", "text")?;
                map.serialize_entry("value", s)?;
                map.end()
            }
            // The full parsed spec, never a lossy projection: it is the value's
            // identity, so a canonical form that dropped it would make two
            // different sparklines indistinguishable.
            Value::Sparkline(spec) => {
                let mut map = serializer.serialize_map(Some(2))?;
                map.serialize_entry("type", "sparkline")?;
                map.serialize_entry("value", &SparklineSpecWire(spec))?;
                map.end()
            }
            Value::Boolean(b) => {
                let mut map = serializer.serialize_map(Some(2))?;
                map.serialize_entry("type", "boolean")?;
                map.serialize_entry("value", b)?;
                map.end()
            }
            // Both error variants serialize identically: the diagnostic message
            // is in-memory-only metadata and is dropped here, so canonical JSON
            // (and the round-trip identity guarantee) is byte-for-byte unchanged.
            Value::Error(code) | Value::ErrorMsg(code, _) => {
                // Key is `error`, not `value`; emitted before `type` to match
                // canonical (JCS) key order.
                let mut map = serializer.serialize_map(Some(2))?;
                map.serialize_entry("error", code)?;
                map.serialize_entry("type", "error")?;
                map.end()
            }
            Value::Empty => {
                let mut map = serializer.serialize_map(Some(2))?;
                map.serialize_entry("type", "empty")?;
                map.serialize_entry("value", &())?;
                map.end()
            }
            Value::Array(rows) => {
                if rows.is_empty() || rows[0].is_empty() {
                    return Err(S::Error::custom("array value must be non-empty"));
                }
                if rows.len() == 1 && rows[0].len() == 1 {
                    return Err(S::Error::custom(
                        "a 1x1 array does not exist in serialized form; collapse it \
                         to its scalar element (schema spec §6)",
                    ));
                }
                let width = rows[0].len();
                for row in rows {
                    if row.len() != width {
                        return Err(S::Error::custom("array value must be rectangular"));
                    }
                    for v in row {
                        if matches!(v, Value::Array(_)) {
                            return Err(S::Error::custom(
                                "array elements must be scalar values (no nested arrays)",
                            ));
                        }
                    }
                }
                let mut map = serializer.serialize_map(Some(2))?;
                map.serialize_entry("type", "array")?;
                map.serialize_entry("value", rows)?;
                map.end()
            }
        }
    }
}

impl<'de> Deserialize<'de> for Value {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        // Buffer into a JSON tree first: the payload's parse depends on the
        // `type` tag, and non-canonical input may order keys arbitrarily.
        let raw = serde_json::Value::deserialize(deserializer)?;
        parse_value(&raw).map_err(D::Error::custom)
    }
}

fn parse_value(raw: &serde_json::Value) -> Result<Value, String> {
    let obj = match raw.as_object() {
        Some(obj) => obj,
        None => return Err("a cell value must be a JSON object".to_string()),
    };
    let kind = match obj.get("type").and_then(serde_json::Value::as_str) {
        Some(kind) => kind,
        None => return Err("a cell value requires a string \"type\" field".to_string()),
    };
    let payload_key = if kind == "error" { "error" } else { "value" };
    let payload = match obj.get(payload_key) {
        Some(payload) if obj.len() == 2 => payload,
        _ => {
            return Err(format!(
                "a {kind} value must have exactly the fields \"type\" and \"{payload_key}\""
            ));
        }
    };
    match kind {
        "number" => Ok(Value::Number(parse_finite_f64(payload, kind)?)),
        "date" => Ok(Value::Date(parse_finite_f64(payload, kind)?)),
        "zoned" => match payload.as_str() {
            Some(s) => parse_rfc9557(s)
                .map(|zi| Value::Zoned(Box::new(zi)))
                .ok_or_else(|| format!("a zoned value must be a valid RFC-9557 string, got {s:?}")),
            None => Err("a zoned value must be a JSON string".to_string()),
        },
        "text" => match payload.as_str() {
            Some(s) => Ok(Value::Text(s.to_owned())),
            None => Err("a text value must be a JSON string".to_string()),
        },
        "boolean" => match payload.as_bool() {
            Some(b) => Ok(Value::Boolean(b)),
            None => Err("a boolean value must be a JSON boolean".to_string()),
        },
        "error" => match payload.as_str() {
            Some(code) => Ok(Value::Error(code.to_owned())),
            None => Err("an error value must carry a string error code".to_string()),
        },
        "empty" => {
            if payload.is_null() {
                Ok(Value::Empty)
            } else {
                Err("an empty value must be JSON null".to_string())
            }
        }
        "array" => parse_array(payload),
        "sparkline" => parse_sparkline(payload),
        other => Err(format!("unknown value type {other:?}")),
    }
}

fn parse_finite_f64(payload: &serde_json::Value, kind: &str) -> Result<f64, String> {
    // `as_f64` is correctly rounded only because the crate enables serde_json's
    // `float_roundtrip` feature; without it, serde_json's default parser is off
    // by up to one ULP for some extreme exponents, which would break the
    // `to_json ∘ from_json = id` byte guarantee (the canonical bytes of the
    // reparsed value would differ in the last shortest-round-trip digit).
    let n = match payload.as_f64() {
        Some(n) => n,
        None => return Err(format!("a {kind} value must be a JSON number")),
    };
    if !n.is_finite() {
        return Err(format!(
            "non-finite {kind} value is forbidden (schema spec §8)"
        ));
    }
    // Normalize -0.0 to 0.0 at the value level (schema spec §8).
    Ok(if n == 0.0 { 0.0 } else { n })
}

/// Parse the payload of a `sparkline` value: the full parsed spec, in the same
/// shape [`SparklineSpecWire`] emits.
fn parse_sparkline(payload: &serde_json::Value) -> Result<Value, String> {
    let obj = payload
        .as_object()
        .ok_or_else(|| "a sparkline value must be a JSON object".to_string())?;
    if obj.len() != 3
        || !obj.contains_key("charttype")
        || !obj.contains_key("data")
        || !obj.contains_key("options")
    {
        return Err(
            "a sparkline value must have exactly the fields \"charttype\", \"data\" and \"options\""
                .to_string(),
        );
    }

    let raw_chart_type = obj["charttype"]
        .as_str()
        .ok_or_else(|| "a sparkline charttype must be a JSON string".to_string())?;
    let chart_type = SparklineChartType::parse(raw_chart_type)
        .ok_or_else(|| format!("unknown sparkline charttype {raw_chart_type:?}"))?;

    let raw_data = obj["data"]
        .as_array()
        .ok_or_else(|| "sparkline data must be a JSON array".to_string())?;
    // The evaluator rejects a single-point `data` argument with `#N/A`, so a
    // shorter spec is unrepresentable and must not round-trip in.
    if raw_data.len() < 2 {
        return Err("sparkline data must hold at least two points".to_string());
    }
    let mut data = Vec::with_capacity(raw_data.len());
    for raw in raw_data {
        data.push(value_to_sparkline_value(&parse_value(raw)?)?);
    }

    let raw_options = obj["options"]
        .as_array()
        .ok_or_else(|| "sparkline options must be a JSON array".to_string())?;
    let mut options = Vec::with_capacity(raw_options.len());
    for raw in raw_options {
        let pair = raw
            .as_array()
            .filter(|p| p.len() == 2)
            .ok_or_else(|| "a sparkline option must be a [key, value] pair".to_string())?;
        let key = pair[0]
            .as_str()
            .ok_or_else(|| "a sparkline option key must be a JSON string".to_string())?;
        if key != key.to_ascii_lowercase() {
            return Err(format!("a sparkline option key must be lower-case, got {key:?}"));
        }
        if key == "charttype" {
            return Err(
                "charttype is carried by the sparkline's own field, not in options".to_string(),
            );
        }
        options.push((key.to_owned(), value_to_sparkline_value(&parse_value(&pair[1])?)?));
    }

    Ok(Value::Sparkline(Box::new(SparklineSpec {
        chart_type,
        data,
        options,
    })))
}

fn parse_array(payload: &serde_json::Value) -> Result<Value, String> {
    let raw_rows = match payload.as_array() {
        Some(rows) => rows,
        None => return Err("an array value must be a 2-D JSON array".to_string()),
    };
    if raw_rows.is_empty() {
        return Err("array value must be non-empty".to_string());
    }
    let mut rows = Vec::with_capacity(raw_rows.len());
    let mut width = None;
    for raw_row in raw_rows {
        let raw_row = match raw_row.as_array() {
            Some(row) => row,
            None => return Err("array rows must be JSON arrays".to_string()),
        };
        if raw_row.is_empty() {
            return Err("array value must be non-empty".to_string());
        }
        match width {
            None => width = Some(raw_row.len()),
            Some(w) if w != raw_row.len() => {
                return Err("array value must be rectangular".to_string());
            }
            Some(_) => {}
        }
        let mut row = Vec::with_capacity(raw_row.len());
        for raw_elem in raw_row {
            let elem = parse_value(raw_elem)?;
            if matches!(elem, Value::Array(_)) {
                return Err("array elements must be scalar values (no nested arrays)".to_string());
            }
            row.push(elem);
        }
        rows.push(row);
    }
    if rows.len() == 1 && rows[0].len() == 1 {
        return Err(
            "a 1x1 array does not exist in serialized form; it must be collapsed \
             to its scalar element (schema spec §6)"
                .to_string(),
        );
    }
    Ok(Value::Array(rows))
}
