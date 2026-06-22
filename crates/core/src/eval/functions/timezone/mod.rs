//! Timezone-aware functions (Model B). They construct, convert, introspect and
//! display [`Value::Zoned`] instants. The naive<->zoned boundary is always an
//! explicit call here: `TZDATETIME`/`TZLOCALIZE` up, `TZSERIAL` down.

use chrono::{Datelike, Duration, Months, NaiveDate, NaiveDateTime, Timelike, Utc};

use crate::eval::coercion::to_number;
use crate::eval::evaluate_expr;
use crate::eval::functions::date::serial::{
    date_to_serial, serial_to_date, serial_to_time, time_to_serial,
};
use crate::eval::functions::{check_arity, check_arity_len, EvalCtx, FunctionMeta, Registry};
use crate::parser::ast::Expr;
use crate::types::zoned::{parse_rfc9557, parse_zone, AmbiguousPolicy};
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
    registry.register_eager("TZLOCALIZE", tzlocalize_fn, FunctionMeta { category: "timezone", signature: "TZLOCALIZE(date_serial,zone,[policy])", description: "Interprets a plain date/time serial as a wall-clock reading in a zone" });
    registry.register_eager("TZFROMEPOCH", tzfromepoch_fn, FunctionMeta { category: "timezone", signature: "TZFROMEPOCH(unix_seconds,zone)", description: "Builds a zone-aware instant from a Unix timestamp" });
    registry.register_eager("TZPARSE", tzparse_fn, FunctionMeta { category: "timezone", signature: "TZPARSE(text,[zone])", description: "Parses an ISO/RFC-9557 timestamp into a zone-aware instant" });
    registry.register_eager("TZISDST", tzisdst_fn, FunctionMeta { category: "timezone", signature: "TZISDST(zoned)", description: "TRUE if daylight-saving is in effect at this instant" });
    registry.register_eager("TZABBR", tzabbr_fn, FunctionMeta { category: "timezone", signature: "TZABBR(zoned)", description: "Zone abbreviation at this instant (e.g. CEST), display only" });
    registry.register_eager("TZOFFSETDIFF", tzoffsetdiff_fn, FunctionMeta { category: "timezone", signature: "TZOFFSETDIFF(zoned_a,zoned_b)", description: "Signed difference in minutes between two zones' offsets" });
    registry.register_eager("TZPART", tzpart_fn, FunctionMeta { category: "timezone", signature: "TZPART(zoned,unit)", description: "Local wall-clock field: year|month|day|hour|minute|second|weekday" });
    registry.register_eager("TZDIFF", tzdiff_fn, FunctionMeta { category: "timezone", signature: "TZDIFF(zoned_a,zoned_b,[unit])", description: "Elapsed time from b to a on the absolute timeline (DST-immune)" });
    registry.register_eager("TZADD", tzadd_fn, FunctionMeta { category: "timezone", signature: "TZADD(zoned,amount,unit,[policy])", description: "Adds time: seconds/minutes/hours are absolute, days/weeks/months/years are DST-aware calendar units" });
    registry.register_lazy("TZNOW", tznow_fn, FunctionMeta { category: "timezone", signature: "TZNOW(zone)", description: "The current moment stamped with the given zone (volatile; pinned during recalc)" });
    registry.register_eager("TZINWINDOW", tzinwindow_fn, FunctionMeta { category: "timezone", signature: "TZINWINDOW(zoned,start_local,end_local,[days_mask])", description: "TRUE if the local time-of-day falls in [start,end); days_mask is 7 chars Sun..Sat" });
    registry.register_eager("TZCANONICAL", tzcanonical_fn, FunctionMeta { category: "timezone", signature: "TZCANONICAL(zone)", description: "Validates and normalizes a zone name (best-effort; IANA links are not resolved)" });
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
    let policy = match optional_policy(args.get(7)) {
        Ok(p) => p,
        Err(e) => return e,
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

pub fn tzlocalize_fn(args: &[Value]) -> Value {
    if let Some(e) = check_arity(args, 2, 3) {
        return e;
    }
    let serial = match to_number(args[0].clone()) {
        Ok(n) => n,
        Err(e) => return e,
    };
    let zone = match arg_zone(&args[1]) {
        Ok(z) => z,
        Err(e) => return e,
    };
    let policy = match optional_policy(args.get(2)) {
        Ok(p) => p,
        Err(e) => return e,
    };
    let naive = match serial_to_naive(serial) {
        Some(n) => n,
        None => return Value::Error(ErrorKind::Value),
    };
    match ZonedInstant::from_local(zone, naive, policy) {
        Some(zi) => zoned(zi),
        None => Value::Error(ErrorKind::Value),
    }
}

pub fn tzfromepoch_fn(args: &[Value]) -> Value {
    if let Some(e) = check_arity(args, 2, 2) {
        return e;
    }
    let secs = match to_number(args[0].clone()) {
        Ok(n) => n,
        Err(e) => return e,
    };
    let zone = match arg_zone(&args[1]) {
        Ok(z) => z,
        Err(e) => return e,
    };
    let nanos = (secs.trunc() as i64)
        .checked_mul(1_000_000_000)
        .and_then(|whole| whole.checked_add((secs.fract() * 1e9).round() as i64));
    match nanos {
        Some(n) => zoned(ZonedInstant::from_instant(n, zone)),
        None => Value::Error(ErrorKind::Num),
    }
}

pub fn tzparse_fn(args: &[Value]) -> Value {
    if let Some(e) = check_arity(args, 1, 2) {
        return e;
    }
    let text = match &args[0] {
        Value::Text(s) => s.as_str(),
        _ => return Value::Error(ErrorKind::Value),
    };
    // Offset-bearing or bracketed strings are self-describing.
    if let Some(zi) = parse_rfc9557(text) {
        return zoned(zi);
    }
    // A naive datetime needs an explicit zone to become an instant.
    if args.len() == 2 {
        let zone = match arg_zone(&args[1]) {
            Ok(z) => z,
            Err(e) => return e,
        };
        if let Some(naive) = parse_naive_datetime(text) {
            return match ZonedInstant::from_local(zone, naive, AmbiguousPolicy::Reject) {
                Some(zi) => zoned(zi),
                None => Value::Error(ErrorKind::Value),
            };
        }
    }
    Value::Error(ErrorKind::Value)
}

pub fn tzisdst_fn(args: &[Value]) -> Value {
    if let Some(e) = check_arity(args, 1, 1) {
        return e;
    }
    match arg_zoned(&args[0]) {
        Ok(zi) => Value::Bool(zi.is_dst()),
        Err(e) => e,
    }
}

pub fn tzabbr_fn(args: &[Value]) -> Value {
    if let Some(e) = check_arity(args, 1, 1) {
        return e;
    }
    match arg_zoned(&args[0]) {
        Ok(zi) => Value::Text(zi.abbrev()),
        Err(e) => e,
    }
}

pub fn tzoffsetdiff_fn(args: &[Value]) -> Value {
    if let Some(e) = check_arity(args, 2, 2) {
        return e;
    }
    let a = match arg_zoned(&args[0]) {
        Ok(z) => z,
        Err(e) => return e,
    };
    let b = match arg_zoned(&args[1]) {
        Ok(z) => z,
        Err(e) => return e,
    };
    Value::Number((a.offset_minutes() - b.offset_minutes()) as f64)
}

pub fn tzpart_fn(args: &[Value]) -> Value {
    if let Some(e) = check_arity(args, 2, 2) {
        return e;
    }
    let zi = match arg_zoned(&args[0]) {
        Ok(z) => z,
        Err(e) => return e,
    };
    let unit = match &args[1] {
        Value::Text(s) => s.to_ascii_lowercase(),
        _ => return Value::Error(ErrorKind::Value),
    };
    let local = zi.local();
    let value = match unit.as_str() {
        "year" => local.year() as f64,
        "month" => local.month() as f64,
        "day" => local.day() as f64,
        "hour" => local.hour() as f64,
        "minute" => local.minute() as f64,
        "second" => local.second() as f64,
        // Sheets WEEKDAY default: Sunday = 1 .. Saturday = 7.
        "weekday" => (local.weekday().num_days_from_sunday() + 1) as f64,
        _ => return Value::Error(ErrorKind::Value),
    };
    Value::Number(value)
}

pub fn tzdiff_fn(args: &[Value]) -> Value {
    if let Some(e) = check_arity(args, 2, 3) {
        return e;
    }
    let a = match arg_zoned(&args[0]) {
        Ok(z) => z,
        Err(e) => return e,
    };
    let b = match arg_zoned(&args[1]) {
        Ok(z) => z,
        Err(e) => return e,
    };
    let unit = match args.get(2) {
        None => "seconds".to_string(),
        Some(Value::Text(s)) => s.to_ascii_lowercase(),
        Some(_) => return Value::Error(ErrorKind::Value),
    };
    // i128 avoids overflow on the difference of two i64 nanosecond counts.
    let diff_ns = a.utc_nanos as i128 - b.utc_nanos as i128;
    let value = match unit.as_str() {
        "nanoseconds" => diff_ns as f64,
        "seconds" => diff_ns as f64 / 1e9,
        "minutes" => diff_ns as f64 / 6e10,
        "hours" => diff_ns as f64 / 3.6e12,
        // Absolute days = 86400 s; calendar-day length is a TZADD concern.
        "days" => diff_ns as f64 / 8.64e13,
        _ => return Value::Error(ErrorKind::Value),
    };
    Value::Number(value)
}

pub fn tzadd_fn(args: &[Value]) -> Value {
    if let Some(e) = check_arity(args, 3, 4) {
        return e;
    }
    let zi = match arg_zoned(&args[0]) {
        Ok(z) => z,
        Err(e) => return e,
    };
    let amount = match to_number(args[1].clone()) {
        Ok(n) => n.trunc() as i64,
        Err(e) => return e,
    };
    let unit = match &args[2] {
        Value::Text(s) => s.to_ascii_lowercase(),
        _ => return Value::Error(ErrorKind::Value),
    };
    let policy = match optional_policy(args.get(3)) {
        Ok(p) => p,
        Err(e) => return e,
    };

    match unit.as_str() {
        // Absolute units shift the instant directly (DST-immune).
        "seconds" | "minutes" | "hours" => {
            let factor: i64 = match unit.as_str() {
                "minutes" => 60,
                "hours" => 3600,
                _ => 1,
            };
            let nanos = amount
                .checked_mul(factor)
                .and_then(|s| s.checked_mul(1_000_000_000))
                .and_then(|d| zi.utc_nanos.checked_add(d));
            match nanos {
                Some(n) => zoned(ZonedInstant::from_instant(n, zi.zone.clone())),
                None => Value::Error(ErrorKind::Num),
            }
        }
        // Calendar units shift the wall clock then re-resolve (DST-aware): a
        // calendar day may be 23 or 25 hours across a transition.
        "days" | "weeks" | "months" | "years" => {
            let local = zi.local();
            let new_local = match unit.as_str() {
                "days" => local.checked_add_signed(Duration::days(amount)),
                "weeks" => local.checked_add_signed(Duration::weeks(amount)),
                "months" => add_months(local, amount),
                _ => amount.checked_mul(12).and_then(|m| add_months(local, m)), // years
            };
            match new_local.and_then(|nl| ZonedInstant::from_local(zi.zone.clone(), nl, policy)) {
                Some(z) => zoned(z),
                None => Value::Error(ErrorKind::Value),
            }
        }
        _ => Value::Error(ErrorKind::Value),
    }
}

/// `TZNOW(zone)` — the current instant stamped with `zone`. Volatile and lazy:
/// it reads the pinned UTC instant from the context (set during recalc), falling
/// back to the ambient UTC clock when unpinned.
pub fn tznow_fn(args: &[Expr], ctx: &mut EvalCtx<'_>) -> Value {
    if let Some(e) = check_arity_len(args.len(), 1, 1) {
        return e;
    }
    let zone_val = evaluate_expr(&args[0], ctx);
    let zone = match &zone_val {
        Value::Text(s) => match parse_zone(s) {
            Some(z) => z,
            None => return Value::Error(ErrorKind::Value),
        },
        Value::Error(_) => return zone_val,
        _ => return Value::Error(ErrorKind::Value),
    };
    let nanos = match ctx.ctx.now_utc_nanos {
        Some(n) => n,
        None => match Utc::now().timestamp_nanos_opt() {
            Some(n) => n,
            None => return Value::Error(ErrorKind::Num),
        },
    };
    zoned(ZonedInstant::from_instant(nanos, zone))
}

/// `TZINWINDOW(zoned, start_local, end_local, [days_mask])` — is the local
/// time-of-day within `[start, end)`? `start`/`end` are time-of-day fractions
/// (0..1, e.g. `TIME(9,0,0)`). An overnight window (`start > end`) wraps past
/// midnight. Optional `days_mask` is 7 characters, index 0 = Sunday, '1' = the
/// day counts.
pub fn tzinwindow_fn(args: &[Value]) -> Value {
    if let Some(e) = check_arity(args, 3, 4) {
        return e;
    }
    let zi = match arg_zoned(&args[0]) {
        Ok(z) => z,
        Err(e) => return e,
    };
    let start = match to_number(args[1].clone()) {
        Ok(n) => n,
        Err(e) => return e,
    };
    let end = match to_number(args[2].clone()) {
        Ok(n) => n,
        Err(e) => return e,
    };
    let local = zi.local();
    if let Some(mask_arg) = args.get(3) {
        let mask = match mask_arg {
            Value::Text(s) => s,
            _ => return Value::Error(ErrorKind::Value),
        };
        let idx = local.weekday().num_days_from_sunday() as usize;
        match mask.chars().nth(idx) {
            Some('1') => {}
            Some(_) => return Value::Bool(false),
            None => return Value::Error(ErrorKind::Value),
        }
    }
    let frac = local.time().num_seconds_from_midnight() as f64 / 86_400.0;
    let inside = if start <= end {
        frac >= start && frac < end
    } else {
        // Overnight window wraps past midnight.
        frac >= start || frac < end
    };
    Value::Bool(inside)
}

/// `TZCANONICAL(zone)` — validate and normalize a zone name. Best-effort: a
/// valid IANA name or fixed offset is returned in canonical form, but chrono-tz
/// keeps backward-compat links (e.g. `US/Pacific`) as distinct names, so links
/// are not resolved to their target.
pub fn tzcanonical_fn(args: &[Value]) -> Value {
    if let Some(e) = check_arity(args, 1, 1) {
        return e;
    }
    match &args[0] {
        Value::Text(s) => match parse_zone(s) {
            Some(ZoneId::Iana(tz)) => Value::Text(tz.name().to_string()),
            Some(ZoneId::Fixed(m)) => {
                let a = m.abs();
                Value::Text(format!("{}{:02}:{:02}", if m < 0 { '-' } else { '+' }, a / 60, a % 60))
            }
            None => Value::Error(ErrorKind::Value),
        },
        _ => Value::Error(ErrorKind::Value),
    }
}

/// Add (or subtract, when negative) a whole number of calendar months.
fn add_months(dt: NaiveDateTime, months: i64) -> Option<NaiveDateTime> {
    if months >= 0 {
        u32::try_from(months).ok().and_then(|m| dt.checked_add_months(Months::new(m)))
    } else {
        u32::try_from(-months).ok().and_then(|m| dt.checked_sub_months(Months::new(m)))
    }
}

/// Resolve the optional ambiguous-policy argument shared by the construction
/// functions. Absent ⇒ [`AmbiguousPolicy::Reject`]; non-text or unknown ⇒ `#VALUE!`.
fn optional_policy(arg: Option<&Value>) -> Result<AmbiguousPolicy, Value> {
    match arg {
        None => Ok(AmbiguousPolicy::Reject),
        Some(Value::Text(s)) => parse_policy(s).ok_or(Value::Error(ErrorKind::Value)),
        Some(_) => Err(Value::Error(ErrorKind::Value)),
    }
}

/// Compose a date serial into a `NaiveDateTime` (integer part = date, fractional
/// part = time of day).
fn serial_to_naive(serial: f64) -> Option<NaiveDateTime> {
    let date = serial_to_date(serial)?;
    let (h, m, s) = serial_to_time(serial);
    date.and_hms_opt(h, m, s)
}

/// Parse a zone-less ISO-ish datetime (or bare date) into a `NaiveDateTime`.
fn parse_naive_datetime(s: &str) -> Option<NaiveDateTime> {
    let s = s.trim();
    for fmt in ["%Y-%m-%dT%H:%M:%S", "%Y-%m-%d %H:%M:%S", "%Y-%m-%dT%H:%M", "%Y-%m-%d %H:%M"] {
        if let Ok(dt) = NaiveDateTime::parse_from_str(s, fmt) {
            return Some(dt);
        }
    }
    NaiveDate::parse_from_str(s, "%Y-%m-%d")
        .ok()
        .and_then(|d| d.and_hms_opt(0, 0, 0))
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
