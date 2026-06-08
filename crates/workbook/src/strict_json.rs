//! Duplicate-key-rejecting JSON parse (schema spec §1, follow-up #568 rule 1).
//!
//! Stock `serde_json` silently keeps the *last* of any duplicate key — both in
//! the derive paths and when buffering through `serde_json::Value` (whose own
//! deserializer drops duplicates). The schema requires duplicate keys be
//! **rejected**. This module parses bytes into a [`serde_json::Value`] tree
//! through a visitor that errors the moment a duplicate key appears in any
//! object, and also enforces UTF-8 / no-BOM at the byte boundary.

use serde::de::{self, Deserialize, Deserializer, MapAccess, SeqAccess, Visitor};
use serde_json::{Map, Value};
use std::fmt;

/// Parses JSON bytes into a [`serde_json::Value`] tree, rejecting duplicate
/// keys in any object and rejecting a leading UTF-8 BOM or invalid UTF-8
/// (schema spec §1).
pub fn parse_no_dup_keys(bytes: &[u8]) -> Result<Value, String> {
    // UTF-8, no BOM (schema spec §1). A BOM is three bytes EF BB BF.
    if bytes.starts_with(&[0xEF, 0xBB, 0xBF]) {
        return Err("a UTF-8 BOM is forbidden (schema spec §1)".to_string());
    }
    // serde_json::from_slice validates UTF-8 as it parses; invalid UTF-8 is
    // reported as a parse error.
    let mut de = serde_json::Deserializer::from_slice(bytes);
    let value = StrictValue::deserialize(&mut de).map_err(|e| e.to_string())?;
    de.end().map_err(|e| e.to_string())?;
    Ok(value.0)
}

/// Newtype whose `Deserialize` impl rejects duplicate object keys.
struct StrictValue(Value);

impl<'de> Deserialize<'de> for StrictValue {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer
            .deserialize_any(StrictValueVisitor)
            .map(StrictValue)
    }
}

struct StrictValueVisitor;

impl<'de> Visitor<'de> for StrictValueVisitor {
    type Value = Value;

    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("any valid JSON value")
    }

    fn visit_bool<E>(self, v: bool) -> Result<Value, E> {
        Ok(Value::Bool(v))
    }
    fn visit_i64<E>(self, v: i64) -> Result<Value, E> {
        Ok(Value::Number(v.into()))
    }
    fn visit_u64<E>(self, v: u64) -> Result<Value, E> {
        Ok(Value::Number(v.into()))
    }
    fn visit_f64<E>(self, v: f64) -> Result<Value, E> {
        Ok(serde_json::Number::from_f64(v).map_or(Value::Null, Value::Number))
    }
    fn visit_str<E>(self, v: &str) -> Result<Value, E> {
        Ok(Value::String(v.to_owned()))
    }
    fn visit_string<E>(self, v: String) -> Result<Value, E> {
        Ok(Value::String(v))
    }
    fn visit_none<E>(self) -> Result<Value, E> {
        Ok(Value::Null)
    }
    fn visit_unit<E>(self) -> Result<Value, E> {
        Ok(Value::Null)
    }

    fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Value, A::Error> {
        let mut items = Vec::new();
        while let Some(StrictValue(v)) = seq.next_element()? {
            items.push(v);
        }
        Ok(Value::Array(items))
    }

    fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Value, A::Error> {
        let mut out = Map::new();
        while let Some(key) = map.next_key::<String>()? {
            let StrictValue(value) = map.next_value()?;
            if out.contains_key(&key) {
                return Err(de::Error::custom(format!(
                    "duplicate object key {key:?} is forbidden (schema spec §1)"
                )));
            }
            out.insert(key, value);
        }
        Ok(Value::Object(out))
    }
}
