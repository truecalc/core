use chrono::{Duration, NaiveDate, NaiveTime, Timelike};

/// Base date for serial number 0 (December 30, 1899).
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
