//! Zone-aware instant value (Model B).
//!
//! [`ZonedInstant`] stores an absolute UTC instant plus an IANA/fixed zone.
//! Offset, wall-clock fields, abbreviation and DST status are **recomputed on
//! demand** from the pinned `chrono-tz` database — never stored — so the tzdb
//! version is part of the engine's conformance contract. The existing tz-naive
//! `Value::Date(f64)` serial is unaffected; the naive<->zoned boundary is always
//! an explicit function call.

use chrono::{DateTime, Duration, NaiveDateTime, Offset, TimeZone, Utc};
use chrono_tz::{OffsetComponents, Tz};

#[cfg(test)]
mod tests;

/// The zone attached to a [`ZonedInstant`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ZoneId {
    /// A named IANA region zone carrying full DST rules (e.g. `Europe/Berlin`).
    Iana(Tz),
    /// A bare fixed offset east of UTC, in minutes (`+05:30` = 330). No DST.
    Fixed(i32),
}

/// An absolute instant labelled with a zone.
///
/// The engine's comparison operators define equality/ordering on `utc_nanos`
/// only (see the `operator` module). The derived `PartialEq` here is structural
/// (instant **and** zone) for Rust-level use such as test assertions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ZonedInstant {
    /// Nanoseconds since the Unix epoch (UTC).
    pub utc_nanos: i64,
    pub zone: ZoneId,
}

/// How to resolve a local wall-clock time that is invalid (spring-forward gap)
/// or ambiguous (fall-back fold) when building a [`ZonedInstant`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AmbiguousPolicy {
    /// Return an error on gap/fold — no silent shift. Deterministic default.
    #[default]
    Reject,
    /// Earlier of two valid instants (fold) / first valid after a gap.
    Earliest,
    /// Later of two valid instants.
    Latest,
    /// Temporal "compatible": fold -> earliest, gap -> push forward.
    Compatible,
}

impl ZonedInstant {
    pub fn from_instant(utc_nanos: i64, zone: ZoneId) -> Self {
        Self { utc_nanos, zone }
    }

    fn utc(&self) -> DateTime<Utc> {
        DateTime::<Utc>::from_timestamp_nanos(self.utc_nanos)
    }

    /// Offset east of UTC in minutes, DST-resolved at this instant.
    pub fn offset_minutes(&self) -> i32 {
        match &self.zone {
            ZoneId::Fixed(m) => *m,
            ZoneId::Iana(tz) => {
                tz.offset_from_utc_datetime(&self.utc().naive_utc())
                    .fix()
                    .local_minus_utc()
                    / 60
            }
        }
    }

    /// Local wall-clock time in this zone (DST-resolved).
    pub fn local(&self) -> NaiveDateTime {
        match &self.zone {
            ZoneId::Fixed(m) => self.utc().naive_utc() + Duration::minutes(*m as i64),
            ZoneId::Iana(tz) => self.utc().with_timezone(tz).naive_local(),
        }
    }

    /// Zone abbreviation at this instant (`CEST`), or a `±HH:MM` label for a
    /// fixed-offset zone.
    pub fn abbrev(&self) -> String {
        match &self.zone {
            ZoneId::Fixed(m) => offset_label(*m),
            ZoneId::Iana(tz) => {
                format!("{}", tz.offset_from_utc_datetime(&self.utc().naive_utc()))
            }
        }
    }

    /// Whether daylight-saving is in effect at this instant.
    pub fn is_dst(&self) -> bool {
        match &self.zone {
            ZoneId::Fixed(_) => false,
            ZoneId::Iana(tz) => {
                tz.offset_from_utc_datetime(&self.utc().naive_utc())
                    .dst_offset()
                    .num_seconds()
                    != 0
            }
        }
    }

    /// Canonical self-describing RFC-9557 string, e.g.
    /// `2026-07-14T11:00:00+02:00[Europe/Berlin]` (bracket omitted for a fixed
    /// offset).
    pub fn to_rfc9557(&self) -> String {
        let local = self.local().format("%Y-%m-%dT%H:%M:%S");
        let base = format!("{}{}", local, offset_label(self.offset_minutes()));
        match &self.zone {
            ZoneId::Iana(tz) => format!("{}[{}]", base, tz.name()),
            ZoneId::Fixed(_) => base,
        }
    }

    /// Build from a local wall-clock time in `zone`, resolving gap/fold per
    /// `policy`. Returns `None` on gap/fold under [`AmbiguousPolicy::Reject`], or
    /// on an out-of-range instant. (Callers map `None` to `#VALUE!`; the
    /// gap-vs-fold distinction is surfaced separately by `TZLOCALSTATUS`.)
    pub fn from_local(zone: ZoneId, naive: NaiveDateTime, policy: AmbiguousPolicy) -> Option<Self> {
        use chrono::LocalResult;
        let utc_nanos = match &zone {
            ZoneId::Fixed(m) => (naive - Duration::minutes(*m as i64))
                .and_utc()
                .timestamp_nanos_opt()?,
            ZoneId::Iana(tz) => match tz.from_local_datetime(&naive) {
                LocalResult::Single(dt) => dt.timestamp_nanos_opt()?,
                LocalResult::Ambiguous(earliest, latest) => match policy {
                    AmbiguousPolicy::Reject => return None,
                    AmbiguousPolicy::Latest => latest.timestamp_nanos_opt()?,
                    _ => earliest.timestamp_nanos_opt()?,
                },
                LocalResult::None => match policy {
                    AmbiguousPolicy::Reject => return None,
                    _ => gap_forward(tz, naive)?,
                },
            },
        };
        Some(Self { utc_nanos, zone })
    }
}

/// Parse an IANA zone name or fixed-offset string into a [`ZoneId`].
/// Accepts `UTC`/`Z`, IANA names (`Europe/Berlin`) and `±HH:MM`/`±HHMM`/`±HH`.
pub fn parse_zone(text: &str) -> Option<ZoneId> {
    let t = text.trim();
    if t.eq_ignore_ascii_case("Z") || t.eq_ignore_ascii_case("UTC") {
        return Some(ZoneId::Iana(Tz::UTC));
    }
    if let Ok(tz) = t.parse::<Tz>() {
        return Some(ZoneId::Iana(tz));
    }
    parse_fixed_offset(t).map(ZoneId::Fixed)
}

/// Parse a canonical RFC-9557 string (`...±HH:MM[Zone]`, or offset-only RFC-3339)
/// into a [`ZonedInstant`].
pub fn parse_rfc9557(text: &str) -> Option<ZonedInstant> {
    let t = text.trim();
    match t.split_once('[') {
        Some((dt_part, rest)) => {
            let zone = parse_zone(rest.strip_suffix(']')?)?;
            let dt = DateTime::parse_from_rfc3339(dt_part).ok()?;
            Some(ZonedInstant {
                utc_nanos: dt.with_timezone(&Utc).timestamp_nanos_opt()?,
                zone,
            })
        }
        None => {
            let dt = DateTime::parse_from_rfc3339(t).ok()?;
            let off_min = dt.offset().local_minus_utc() / 60;
            Some(ZonedInstant {
                utc_nanos: dt.with_timezone(&Utc).timestamp_nanos_opt()?,
                zone: ZoneId::Fixed(off_min),
            })
        }
    }
}

/// Format a minutes-east-of-UTC offset as `±HH:MM` (330 -> `+05:30`).
fn offset_label(minutes: i32) -> String {
    let sign = if minutes < 0 { '-' } else { '+' };
    let a = minutes.abs();
    format!("{}{:02}:{:02}", sign, a / 60, a % 60)
}

/// Resolve a wall-clock time inside a spring-forward gap to the first valid
/// instant at or after it by probing increasing forward shifts.
fn gap_forward(tz: &Tz, naive: NaiveDateTime) -> Option<i64> {
    use chrono::LocalResult;
    for mins in [15i64, 30, 45, 60, 90, 120] {
        match tz.from_local_datetime(&(naive + Duration::minutes(mins))) {
            LocalResult::Single(dt) | LocalResult::Ambiguous(dt, _) => {
                return dt.timestamp_nanos_opt();
            }
            LocalResult::None => continue,
        }
    }
    None
}

/// Parse `±HH:MM` / `±HHMM` / `±HH` into minutes east of UTC.
fn parse_fixed_offset(t: &str) -> Option<i32> {
    let sign = match t.as_bytes().first()? {
        b'+' => 1,
        b'-' => -1,
        _ => return None,
    };
    let rest = &t[1..];
    let (h, m) = if let Some((h, m)) = rest.split_once(':') {
        (h.parse::<i32>().ok()?, m.parse::<i32>().ok()?)
    } else if rest.len() == 4 {
        (rest[..2].parse().ok()?, rest[2..].parse().ok()?)
    } else {
        (rest.parse::<i32>().ok()?, 0)
    };
    if !(0..=23).contains(&h) || !(0..=59).contains(&m) {
        return None;
    }
    Some(sign * (h * 60 + m))
}
