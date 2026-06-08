use chrono::{Duration, NaiveDate, NaiveTime, Timelike};

/// Base date for serial number 0 in the **Sheets** date system
/// (December 30, 1899). Locked per engine flavor — see `engine::Flavor`
/// and P1.4 (issue #526).
fn base() -> NaiveDate {
    NaiveDate::from_ymd_opt(1899, 12, 30).unwrap()
}

/// Convert a serial number to a NaiveDate (integer part only).
pub fn serial_to_date(serial: f64) -> Option<NaiveDate> {
    let days = serial.floor() as i64;
    base().checked_add_signed(Duration::days(days))
}

/// Convert a NaiveDate to a serial number.
pub fn date_to_serial(date: NaiveDate) -> f64 {
    date.signed_duration_since(base()).num_days() as f64
}

/// Extract time-of-day components from the fractional part of a serial.
pub fn serial_to_time(serial: f64) -> (u32, u32, u32) {
    let frac = serial.fract().abs();
    let total_secs = (frac * 86400.0).round() as u32;
    let h = total_secs / 3600;
    let m = (total_secs % 3600) / 60;
    let s = total_secs % 60;
    (h, m, s)
}

/// Convert time components to a fractional-day serial.
pub fn time_to_serial(h: u32, m: u32, s: u32) -> f64 {
    (h as f64 * 3600.0 + m as f64 * 60.0 + s as f64) / 86400.0
}

/// Parse a date text string and return its serial number, or None on failure.
/// Supports the same formats as DATEVALUE.
pub fn text_to_date_serial(text: &str) -> Option<f64> {
    let formats = [
        "%m/%d/%Y", "%m/%d/%y", "%Y-%m-%d",
        "%d-%b-%Y", "%d-%b-%y",
        "%B %d, %Y", "%b %d, %Y",
        "%B %d %Y",  "%b %d %Y",
    ];
    for fmt in &formats {
        if let Ok(date) = NaiveDate::parse_from_str(text, fmt) {
            return Some(date_to_serial(date));
        }
    }
    None
}

/// Parse a time text string and return its fractional day serial, or None on failure.
/// Supports the same formats as TIMEVALUE.
pub fn text_to_time_serial(text: &str) -> Option<f64> {
    let formats = [
        "%H:%M:%S", "%H:%M",
        "%I:%M %p", "%I:%M:%S %p",
        "%I:%M%p",  "%I:%M:%S%p",
    ];
    for fmt in &formats {
        if let Ok(t) = NaiveTime::parse_from_str(text, fmt) {
            return Some(time_to_serial(t.hour(), t.minute(), t.second()));
        }
    }
    None
}

// ── Excel 1900 date system (flavor-locked, P1.4 issue #526) ─────────────────
//
// Excel's 1900 date system: serial 1 = 1900-01-01, **including** the
// historical Lotus 1-2-3 leap-year bug — serial 60 is the fictitious
// 1900-02-29. Consequences:
//   - serials 1..=59  map to 1900-01-01..=1900-02-28 (= Sheets serial − 1),
//   - serial  60      maps to the nonexistent 1900-02-29,
//   - serials ≥ 61    coincide with Sheets serials (1900-03-01 onward).
//
// These helpers lock the semantics behind the `Engine::excel()` flavor.
// Excel *evaluation* is still stubbed (`Engine::evaluate` returns `#N/A`
// for the Excel flavor), as issue #526 explicitly allows.

/// Convert a calendar date to an Excel 1900-system serial.
///
/// `(1900, 2, 29)` — the fictitious leap-bug date — returns `Some(60.0)`.
/// Dates before 1900-01-01 are not representable in the Excel 1900 system
/// and return `None`.
pub fn excel_ymd_to_serial(year: i32, month: u32, day: u32) -> Option<f64> {
    if (year, month, day) == (1900, 2, 29) {
        return Some(60.0);
    }
    let date = NaiveDate::from_ymd_opt(year, month, day)?;
    let sheets_serial = date_to_serial(date);
    // Sheets serial 2 = 1900-01-01 = Excel serial 1.
    if sheets_serial < 2.0 {
        return None;
    }
    // Before the (Sheets) serial of 1900-03-01 (= 61), Excel is one behind
    // Sheets; from 1900-03-01 on, the fictitious Feb 29 makes them coincide.
    Some(if sheets_serial < 61.0 { sheets_serial - 1.0 } else { sheets_serial })
}

/// Convert an Excel 1900-system serial (integer part) to `(year, month, day)`.
///
/// Serial 60 returns `Some((1900, 2, 29))` — the fictitious leap-bug date,
/// which is why this returns a tuple rather than a `NaiveDate`.
/// Serials below 1 return `None`.
pub fn excel_serial_to_ymd(serial: f64) -> Option<(i32, u32, u32)> {
    use chrono::Datelike;
    let days = serial.floor();
    if days < 1.0 {
        return None;
    }
    if days == 60.0 {
        return Some((1900, 2, 29));
    }
    let sheets_serial = if days < 60.0 { days + 1.0 } else { days };
    let date = serial_to_date(sheets_serial)?;
    Some((date.year(), date.month(), date.day()))
}
