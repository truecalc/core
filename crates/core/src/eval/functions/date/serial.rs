use chrono::{Datelike, Duration, NaiveDate, NaiveTime, Timelike};

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
/// Supports the same formats as DATEVALUE (including datetime strings and Y/m/d slash).
pub fn text_to_date_serial(text: &str) -> Option<f64> {
    // Empty string coerces to serial 0 (Dec 30, 1899) — matches Google Sheets.
    if text.is_empty() {
        return Some(0.0);
    }

    // Strip a trailing time component only when the part after the space looks
    // like a time string (starts with digit(s) followed by ':').  This avoids
    // truncating month-name formats such as "January 1, 2023".
    let date_part: &str = if let Some(idx) = text.find(' ') {
        let after = &text[idx + 1..];
        let looks_like_time = after
            .chars()
            .next()
            .map(|c| c.is_ascii_digit())
            .unwrap_or(false)
            && after.contains(':');
        if looks_like_time { &text[..idx] } else { text }
    } else {
        text
    };

    let formats_4y = [
        "%m/%d/%Y", "%Y-%m-%d", "%Y/%m/%d",
        "%d-%b-%Y",
        "%B %d, %Y", "%b %d, %Y",
        "%B %d %Y",  "%b %d %Y",
    ];
    for fmt in &formats_4y {
        if let Ok(date) = NaiveDate::parse_from_str(date_part, fmt) {
            return Some(date_to_serial(date));
        }
    }
    // Two-digit-year formats with century remapping (00-29→2000s, 30-99→1900s).
    let formats_2y: &[&str] = &["%m/%d/%y", "%d-%b-%y"];
    for fmt in formats_2y {
        if let Ok(date) = NaiveDate::parse_from_str(date_part, fmt) {
            let y = date.year();
            let century_year = if y <= 29 { y + 2000 } else if y <= 99 { y + 1900 } else { y };
            if let Some(remapped) = NaiveDate::from_ymd_opt(century_year, date.month(), date.day()) {
                return Some(date_to_serial(remapped));
            }
        }
    }
    None
}

/// Parse a time text string and return its fractional day serial, or None on failure.
/// Supports the same formats as TIMEVALUE, including fractional seconds and datetime strings.
pub fn text_to_time_serial(text: &str) -> Option<f64> {
    // If it looks like a datetime (contains a space AND the part before the space
    // does not itself look like a time string), extract the time part.
    // "2023-06-15 14:30:00" → time_part = "14:30:00"
    // "2:15 PM" → time_part = "2:15 PM"  (kept whole, `:` before the space)
    // "12:00:00 AM" → time_part = "12:00:00 AM" (kept whole)
    let time_part = if let Some(idx) = text.find(' ') {
        let before = &text[..idx];
        if before.contains(':') {
            // Part before space already looks like a time component — keep the full string.
            text
        } else {
            &text[idx + 1..]
        }
    } else {
        text
    };

    // Try fractional-seconds formats first (e.g. "11:59:59.50 PM").
    // chrono's %S does not handle fractional seconds, so we strip the sub-second
    // part manually then delegate to the integer-second path.
    let stripped = strip_fractional_seconds(time_part);
    let candidates: &[&str] = if stripped.as_deref() != Some(time_part) {
        // We did strip something — try the stripped version.
        &[stripped.as_deref().unwrap_or(time_part), time_part]
    } else {
        &[time_part]
    };

    let formats = [
        "%H:%M:%S",
        "%H:%M",
        "%I:%M %p",
        "%I:%M:%S %p",
        "%I:%M%p",
        "%I:%M:%S%p",
    ];

    // For fractional-seconds inputs, parse the stripped string and add back the
    // sub-second fraction manually.
    if let Some(ref stripped_str) = stripped {
        if stripped_str.as_str() != time_part {
            // Determine the sub-second value.
            let sub_second = extract_sub_second(time_part).unwrap_or(0.0);
            for fmt in &formats {
                if let Ok(t) = NaiveTime::parse_from_str(stripped_str, fmt) {
                    let serial = time_to_serial(t.hour(), t.minute(), t.second());
                    return Some(serial + sub_second / 86400.0);
                }
            }
        }
    }

    for candidate in candidates {
        for fmt in &formats {
            if let Ok(t) = NaiveTime::parse_from_str(candidate, fmt) {
                return Some(time_to_serial(t.hour(), t.minute(), t.second()));
            }
        }
    }

    None
}

/// Remove a sub-second component from a time string so chrono can parse it.
/// "11:59:59.50 PM" → Some("11:59:59 PM"), "14:15:30" → None (no change needed).
fn strip_fractional_seconds(s: &str) -> Option<String> {
    // Look for a decimal point that follows two digits (the seconds field).
    if let Some(dot_pos) = s.find('.') {
        // Make sure there are at least 2 chars before the dot (the seconds digits).
        if dot_pos >= 2 {
            // Find where the sub-second digits end (next non-digit character or end of string).
            let after_dot = &s[dot_pos + 1..];
            let frac_len = after_dot.chars().take_while(|c| c.is_ascii_digit()).count();
            let end = dot_pos + 1 + frac_len;
            let result = format!("{}{}", &s[..dot_pos], &s[end..]);
            return Some(result);
        }
    }
    None
}

/// Extract the sub-second value (0.0..1.0) from a time string containing a decimal point.
fn extract_sub_second(s: &str) -> Option<f64> {
    if let Some(dot_pos) = s.find('.') {
        let after_dot = &s[dot_pos..]; // ".50 PM" etc.
        let frac_end = 1 + after_dot[1..].chars().take_while(|c| c.is_ascii_digit()).count();
        let frac_str = &after_dot[..frac_end]; // ".50"
        frac_str.parse::<f64>().ok()
    } else {
        None
    }
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
