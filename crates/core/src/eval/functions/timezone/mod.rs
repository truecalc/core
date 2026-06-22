//! Timezone-aware functions (Model B). They construct, convert, introspect and
//! display [`Value::Zoned`] instants. The naive<->zoned boundary is always an
//! explicit call here: `TZDATETIME`/`TZLOCALIZE` up, `TZSERIAL` down.

use chrono::{NaiveDate, Timelike};

use crate::eval::coercion::to_number;
use crate::eval::functions::date::serial::{date_to_serial, time_to_serial};
use crate::eval::functions::{check_arity, FunctionMeta, Registry};
use crate::types::zoned::{parse_zone, AmbiguousPolicy};
use crate::types::{ErrorKind, Value, ZoneId, ZonedInstant};

#[cfg(test)]
mod tests;

pub fn register_timezone(registry: &mut Registry) {
    registry.register_eager("TZDBVERSION", tzdbversion_fn, FunctionMeta { category: "timezone", signature: "TZDBVERSION()", description: "IANA time-zone database version the engine is using" });
    registry.register_eager("TZVALID", tzvalid_fn, FunctionMeta { category: "timezone", signature: "TZVALID(zone)", description: "TRUE if the text is a valid IANA zone name or fixed offset" });
    registry.register_eager("ISZONED", iszoned_fn, FunctionMeta { category: "timezone", signature: "ISZONED(value)", description: "TRUE if the value is a zone-aware instant" });
    registry.register_eager("TZDATETIME", tzdatetime_fn, FunctionMeta { category: "timezone", signature: "TZDATETIME(year,month,day,hour,minute,second,zone,[policy])", description: "Builds a zone-aware instant from a wall-clock time in a zone" });
    registry.register_eager("TZCONVERT", tzconvert_fn, FunctionMeta { category: "timezone", signature: "TZCONVERT(zoned,target_zone)", description: "Same instant shown as the local time in another zone" });
    registry.register_eager("TZSERIAL", tzserial_fn, FunctionMeta { category: "timezone", signature: "TZSERIAL(zoned)", description: "Drops the zone, returning the wall-clock time as a plain date serial" });
    registry.register_eager("TZOFFSET", tzoffset_fn, FunctionMeta { category: "timezone", signature: "TZOFFSET(zoned)", description: "Offset from UTC in minutes at this instant (DST-resolved)" });
    registry.register_eager("TZSTRING", tzstring_fn, FunctionMeta { category: "timezone", signature: "TZSTRING(zoned)", description: "Canonical RFC-9557 string for a zone-aware instant" });
}

// ── Shared helpers ────────────────────────────────────────────────────────────

/// Borrow a [`ZonedInstant`] from a `Zoned` argument, else `#VALUE!`.
fn arg_zoned(v: &Value) -> Result<&ZonedInstant, Value> {
    match v {
        Value::Zoned(z) => Ok(z),
        _ => Err(Value::Error(ErrorKind::Value)),
    }
}

/// Parse a zone from a text argument, else `#VALUE!`.
fn arg_zone(v: &Value) -> Result<ZoneId, Value> {
    match v {
        Value::Text(s) => parse_zone(s).ok_or(Value::Error(ErrorKind::Value)),
        _ => Err(Value::Error(ErrorKind::Value)),
    }
}

/// Coerce an argument to a truncated integer, propagating coercion errors.
fn int_arg(v: &Value) -> Result<i64, Value> {
    Ok(to_number(v.clone())?.trunc() as i64)
}

fn zoned(zi: ZonedInstant) -> Value {
    Value::Zoned(Box::new(zi))
}

// ── Functions ─────────────────────────────────────────────────────────────────

pub fn tzdbversion_fn(args: &[Value]) -> Value {
    if let Some(e) = check_arity(args, 0, 0) {
        return e;
    }
    Value::Text(chrono_tz::IANA_TZDB_VERSION.to_string())
}

pub fn tzvalid_fn(args: &[Value]) -> Value {
    if let Some(e) = check_arity(args, 1, 1) {
        return e;
    }
    match &args[0] {
        Value::Text(s) => Value::Bool(parse_zone(s).is_some()),
        _ => Value::Bool(false),
    }
}

pub fn iszoned_fn(args: &[Value]) -> Value {
    if let Some(e) = check_arity(args, 1, 1) {
        return e;
    }
    Value::Bool(matches!(args[0], Value::Zoned(_)))
}

pub fn tzdatetime_fn(args: &[Value]) -> Value {
    if let Some(e) = check_arity(args, 7, 8) {
        return e;
    }
    let parts = match (0..6).map(|i| int_arg(&args[i])).collect::<Result<Vec<_>, _>>() {
        Ok(p) => p,
        Err(e) => return e,
    };
    let zone = match arg_zone(&args[6]) {
        Ok(z) => z,
        Err(e) => return e,
    };
    let policy = if args.len() == 8 {
        match &args[7] {
            Value::Text(s) => match parse_policy(s) {
                Some(p) => p,
                None => return Value::Error(ErrorKind::Value),
            },
            _ => return Value::Error(ErrorKind::Value),
        }
    } else {
        AmbiguousPolicy::Reject
    };

    let (year, month, day, hour, minute, second) =
        (parts[0], parts[1], parts[2], parts[3], parts[4], parts[5]);
    let naive = match (
        i32::try_from(year).ok(),
        u32::try_from(month).ok(),
        u32::try_from(day).ok(),
        u32::try_from(hour).ok(),
        u32::try_from(minute).ok(),
        u32::try_from(second).ok(),
    ) {
        (Some(y), Some(mo), Some(d), Some(h), Some(mi), Some(s)) => NaiveDate::from_ymd_opt(y, mo, d)
            .and_then(|date| date.and_hms_opt(h, mi, s)),
        _ => None,
    };
    let naive = match naive {
        Some(n) => n,
        None => return Value::Error(ErrorKind::Value),
    };
    match ZonedInstant::from_local(zone, naive, policy) {
        Some(zi) => zoned(zi),
        None => Value::Error(ErrorKind::Value),
    }
}

pub fn tzconvert_fn(args: &[Value]) -> Value {
    if let Some(e) = check_arity(args, 2, 2) {
        return e;
    }
    let zi = match arg_zoned(&args[0]) {
        Ok(z) => z,
        Err(e) => return e,
    };
    let target = match arg_zone(&args[1]) {
        Ok(z) => z,
        Err(e) => return e,
    };
    zoned(ZonedInstant::from_instant(zi.utc_nanos, target))
}

pub fn tzserial_fn(args: &[Value]) -> Value {
    if let Some(e) = check_arity(args, 1, 1) {
        return e;
    }
    let zi = match arg_zoned(&args[0]) {
        Ok(z) => z,
        Err(e) => return e,
    };
    let local = zi.local();
    let serial = date_to_serial(local.date())
        + time_to_serial(local.hour(), local.minute(), local.second());
    Value::Date(serial)
}

pub fn tzoffset_fn(args: &[Value]) -> Value {
    if let Some(e) = check_arity(args, 1, 1) {
        return e;
    }
    match arg_zoned(&args[0]) {
        Ok(zi) => Value::Number(zi.offset_minutes() as f64),
        Err(e) => e,
    }
}

pub fn tzstring_fn(args: &[Value]) -> Value {
    if let Some(e) = check_arity(args, 1, 1) {
        return e;
    }
    match arg_zoned(&args[0]) {
        Ok(zi) => Value::Text(zi.to_rfc9557()),
        Err(e) => e,
    }
}

fn parse_policy(s: &str) -> Option<AmbiguousPolicy> {
    match s.to_ascii_lowercase().as_str() {
        "reject" => Some(AmbiguousPolicy::Reject),
        "earliest" => Some(AmbiguousPolicy::Earliest),
        "latest" => Some(AmbiguousPolicy::Latest),
        "compatible" => Some(AmbiguousPolicy::Compatible),
        _ => None,
    }
}
