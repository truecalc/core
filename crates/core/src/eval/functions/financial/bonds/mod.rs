/// Bond and securities financial functions.
///
/// Implements:
///   ACCRINT, ACCRINTM, COUPDAYBS, COUPDAYS, COUPDAYSNC, COUPNCD, COUPNUM, COUPPCD
///   DISC, INTRATE, PRICE, PRICEDISC, PRICEMAT, RECEIVED
///   TBILLEQ, TBILLPRICE, TBILLYIELD
///   YIELD, YIELDDISC, YIELDMAT
use chrono::{Datelike, Duration, NaiveDate};

use crate::eval::coercion::to_number;
use crate::eval::functions::check_arity;
use crate::eval::functions::date::serial::{date_to_serial, serial_to_date};
use crate::types::{ErrorKind, Value};

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

fn opt_number(args: &[Value], idx: usize, default: f64) -> Result<f64, Value> {
    if idx < args.len() {
        to_number(args[idx].clone())
    } else {
        Ok(default)
    }
}

/// Validate a day-count basis integer (0–4). Returns Err(#NUM!) if invalid.
pub fn validate_basis(basis: f64) -> Result<u32, Value> {
    let b = basis.floor() as i64;
    if !(0..=4).contains(&b) {
        return Err(Value::Error(ErrorKind::Num));
    }
    Ok(b as u32)
}

/// Validate frequency: only 1, 2, 4 are allowed.
pub fn validate_frequency(freq: f64) -> Result<u32, Value> {
    let f = freq.floor() as i64;
    if f != 1 && f != 2 && f != 4 {
        return Err(Value::Error(ErrorKind::Num));
    }
    Ok(f as u32)
}

fn is_leap(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}

/// US 30/360 day count (NASD convention).
fn days_30_360_us(start: NaiveDate, end: NaiveDate) -> i64 {
    let (mut d1, m1, y1) = (start.day() as i64, start.month() as i64, start.year() as i64);
    let (mut d2, m2, y2) = (end.day() as i64, end.month() as i64, end.year() as i64);
    if d1 == 31 {
        d1 = 30;
    }
    if d2 == 31 && d1 == 30 {
        d2 = 30;
    }
    (y2 - y1) * 360 + (m2 - m1) * 30 + (d2 - d1)
}

/// European 30/360 day count.
fn days_30_360_eu(start: NaiveDate, end: NaiveDate) -> i64 {
    let (mut d1, m1, y1) = (start.day() as i64, start.month() as i64, start.year() as i64);
    let (mut d2, m2, y2) = (end.day() as i64, end.month() as i64, end.year() as i64);
    if d1 == 31 {
        d1 = 30;
    }
    if d2 == 31 {
        d2 = 30;
    }
    (y2 - y1) * 360 + (m2 - m1) * 30 + (d2 - d1)
}

/// Number of actual days between two dates (end - start).
fn actual_days(start: NaiveDate, end: NaiveDate) -> i64 {
    end.signed_duration_since(start).num_days()
}

/// Year-fraction denominator for basis 1 (Actual/Actual).
/// For the coupon period [pcd, ncd), the "year length" is:
/// If a single year: len(year), otherwise span * freq / 1 where span = ncd - pcd.
fn year_frac_act_act(start: NaiveDate, end: NaiveDate) -> f64 {
    if start == end {
        return 0.0;
    }
    let days = end.signed_duration_since(start).num_days() as f64;
    if start.year() == end.year() {
        let yl = if is_leap(start.year()) { 366.0 } else { 365.0 };
        return days / yl;
    }
    // Span multiple years — split at year boundaries.
    let mut total = 0.0;
    let mut cur = start;
    while cur.year() < end.year() {
        let next_year_start = NaiveDate::from_ymd_opt(cur.year() + 1, 1, 1).unwrap();
        let d = next_year_start.signed_duration_since(cur).num_days() as f64;
        let yl = if is_leap(cur.year()) { 366.0 } else { 365.0 };
        total += d / yl;
        cur = next_year_start;
    }
    let d = end.signed_duration_since(cur).num_days() as f64;
    let yl = if is_leap(end.year()) { 366.0 } else { 365.0 };
    total += d / yl;
    total
}

/// Year-fraction between two dates under the given basis.
pub fn yearfrac(start: NaiveDate, end: NaiveDate, basis: u32) -> f64 {
    match basis {
        0 => days_30_360_us(start, end) as f64 / 360.0,
        1 => year_frac_act_act(start, end),
        2 => actual_days(start, end) as f64 / 360.0,
        3 => actual_days(start, end) as f64 / 365.0,
        4 => days_30_360_eu(start, end) as f64 / 360.0,
        _ => 0.0,
    }
}

/// Number of days between two dates under the given basis (for coupon functions).
/// For basis 0/4: use 30/360 formula.
/// For basis 1/2/3: actual calendar days.
pub fn days_between(start: NaiveDate, end: NaiveDate, basis: u32) -> i64 {
    match basis {
        0 => days_30_360_us(start, end),
        1..=3 => actual_days(start, end),
        4 => days_30_360_eu(start, end),
        _ => 0,
    }
}

/// Number of days in a coupon period (from prev coupon to next coupon) under basis.
/// For basis 0: 360 / frequency.
/// For basis 1: actual days from prev to next coupon date.
/// For basis 2: 360 / frequency.
/// For basis 3: 365 / frequency.
/// For basis 4: 360 / frequency.
pub fn coupon_period_days(pcd: NaiveDate, ncd: NaiveDate, frequency: u32, basis: u32) -> f64 {
    match basis {
        0 | 4 => 360.0 / frequency as f64,
        1 => actual_days(pcd, ncd) as f64,
        2 => 360.0 / frequency as f64,
        3 => 365.0 / frequency as f64,
        _ => 360.0 / frequency as f64,
    }
}

/// Add months to a date, snapping to end-of-month if necessary.
pub fn add_months(date: NaiveDate, months: i32) -> NaiveDate {
    let total_months = date.year() * 12 + date.month() as i32 - 1 + months;
    let year = total_months / 12;
    let month = (total_months % 12 + 1) as u32;
    let day = date.day();
    // Try the same day, fall back to end of month
    NaiveDate::from_ymd_opt(year, month, day)
        .unwrap_or_else(|| {
            // end of month
            let next_month = if month == 12 {
                NaiveDate::from_ymd_opt(year + 1, 1, 1).unwrap()
            } else {
                NaiveDate::from_ymd_opt(year, month + 1, 1).unwrap()
            };
            next_month - Duration::days(1)
        })
}

/// Months per coupon period.
pub fn months_per_period(frequency: u32) -> i32 {
    12 / frequency as i32
}

/// Previous coupon date on or before settlement.
///
/// Steps backwards from maturity by `months_per_period` until the candidate
/// is ≤ settlement.
pub fn prev_coupon_date(settlement: NaiveDate, maturity: NaiveDate, frequency: u32) -> NaiveDate {
    let mpp = months_per_period(frequency);
    let mut candidate = maturity;
    while candidate > settlement {
        candidate = add_months(candidate, -mpp);
    }
    candidate
}

/// Next coupon date strictly after settlement (= prev coupon + one period).
pub fn next_coupon_date(settlement: NaiveDate, maturity: NaiveDate, frequency: u32) -> NaiveDate {
    let pcd = prev_coupon_date(settlement, maturity, frequency);
    let mpp = months_per_period(frequency);
    add_months(pcd, mpp)
}

// ---------------------------------------------------------------------------
// ACCRINT
// ---------------------------------------------------------------------------
/// `ACCRINT(issue, first_interest, settlement, rate, par, frequency, [basis], [calc_method])`
///
/// Accrued interest for a security that pays periodic interest.
/// Only frequency 1, 2, 4 allowed.
pub fn accrint_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 6, 8) {
        return err;
    }
    let issue_s = match to_number(args[0].clone()) {
        Ok(n) => n,
        Err(e) => return e,
    };
    let _first_interest_s = match to_number(args[1].clone()) {
        Ok(n) => n,
        Err(e) => return e,
    };
    let settlement_s = match to_number(args[2].clone()) {
        Ok(n) => n,
        Err(e) => return e,
    };
    let rate = match to_number(args[3].clone()) {
        Ok(n) => n,
        Err(e) => return e,
    };
    let par = match to_number(args[4].clone()) {
        Ok(n) => n,
        Err(e) => return e,
    };
    let freq_f = match to_number(args[5].clone()) {
        Ok(n) => n,
        Err(e) => return e,
    };
    let basis_f = match opt_number(args, 6, 0.0) {
        Ok(n) => n,
        Err(e) => return e,
    };

    if rate < 0.0 || par <= 0.0 {
        return Value::Error(ErrorKind::Num);
    }

    let frequency = match validate_frequency(freq_f) {
        Ok(f) => f,
        Err(e) => return e,
    };
    let basis = match validate_basis(basis_f) {
        Ok(b) => b,
        Err(e) => return e,
    };

    let issue = match serial_to_date(issue_s) {
        Some(d) => d,
        None => return Value::Error(ErrorKind::Value),
    };
    let settlement = match serial_to_date(settlement_s) {
        Some(d) => d,
        None => return Value::Error(ErrorKind::Value),
    };

    if settlement <= issue {
        return Value::Error(ErrorKind::Num);
    }

    // ACCRINT sums over coupon periods from issue to settlement.
    // For each period [pcd, ncd):
    //   AI += par * (rate/frequency) * A/E
    // where A = days from max(pcd, issue) to min(ncd, settlement)
    //       E = coupon_period_days(pcd, ncd, frequency, basis)
    //
    // Note: for basis 0 (30/360), Excel uses EU-style day count (d2=31→30)
    // for the numerator A, which matches `days_30_360_eu`.
    let mpp = months_per_period(frequency);
    let mut result = 0.0;
    let mut pcd = issue;
    let mut ncd = add_months(pcd, mpp);

    loop {
        let period_start = if pcd < issue { issue } else { pcd };
        let period_end = if ncd > settlement { settlement } else { ncd };

        if period_start >= period_end {
            if ncd > settlement {
                break;
            }
            pcd = ncd;
            ncd = add_months(pcd, mpp);
            continue;
        }

        // For basis 0, use EU 30/360 for the numerator (Excel behavior).
        // For basis 3, when the last period ends exactly at the coupon date
        // (period_end == ncd), use E as the numerator (full coupon; Excel behavior).
        let is_last = ncd >= settlement;
        let a = if basis == 0 {
            days_30_360_eu(period_start, period_end) as f64
        } else if basis == 3 && is_last && period_end == ncd {
            coupon_period_days(pcd, ncd, frequency, basis)
        } else {
            days_between(period_start, period_end, basis) as f64
        };
        let e = coupon_period_days(pcd, ncd, frequency, basis);

        if e > 0.0 {
            result += par * (rate / frequency as f64) * (a / e);
        }

        if is_last {
            break;
        }
        pcd = ncd;
        ncd = add_months(pcd, mpp);
    }

    if !result.is_finite() {
        return Value::Error(ErrorKind::Num);
    }
    Value::Number(result)
}

// ---------------------------------------------------------------------------
// ACCRINTM
// ---------------------------------------------------------------------------
/// `ACCRINTM(issue, settlement, rate, par, [basis])`
///
/// Accrued interest for a security that pays interest at maturity.
pub fn accrintm_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 4, 5) {
        return err;
    }
    let issue_s = match to_number(args[0].clone()) {
        Ok(n) => n,
        Err(e) => return e,
    };
    let settlement_s = match to_number(args[1].clone()) {
        Ok(n) => n,
        Err(e) => return e,
    };
    let rate = match to_number(args[2].clone()) {
        Ok(n) => n,
        Err(e) => return e,
    };
    let par = match to_number(args[3].clone()) {
        Ok(n) => n,
        Err(e) => return e,
    };
    let basis_f = match opt_number(args, 4, 0.0) {
        Ok(n) => n,
        Err(e) => return e,
    };

    if rate < 0.0 || par <= 0.0 {
        return Value::Error(ErrorKind::Num);
    }

    let basis = match validate_basis(basis_f) {
        Ok(b) => b,
        Err(e) => return e,
    };

    let issue = match serial_to_date(issue_s) {
        Some(d) => d,
        None => return Value::Error(ErrorKind::Value),
    };
    let settlement = match serial_to_date(settlement_s) {
        Some(d) => d,
        None => return Value::Error(ErrorKind::Value),
    };

    if settlement <= issue {
        return Value::Error(ErrorKind::Num);
    }

    let yf = yearfrac(issue, settlement, basis);
    let result = par * rate * yf;

    if !result.is_finite() {
        return Value::Error(ErrorKind::Num);
    }
    Value::Number(result)
}

// ---------------------------------------------------------------------------
// COUPDAYBS
// ---------------------------------------------------------------------------
/// `COUPDAYBS(settlement, maturity, frequency, [basis])`
///
/// Days from the beginning of the coupon period to the settlement date.
pub fn coupdaybs_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 3, 4) {
        return err;
    }
    let (settlement, maturity, frequency, basis) = match parse_coup_args(args) {
        Ok(v) => v,
        Err(e) => return e,
    };
    let pcd = prev_coupon_date(settlement, maturity, frequency);
    let days = days_between(pcd, settlement, basis);
    Value::Number(days as f64)
}

// ---------------------------------------------------------------------------
// COUPDAYS
// ---------------------------------------------------------------------------
/// `COUPDAYS(settlement, maturity, frequency, [basis])`
///
/// Number of days in the coupon period containing the settlement date.
pub fn coupdays_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 3, 4) {
        return err;
    }
    let (settlement, maturity, frequency, basis) = match parse_coup_args(args) {
        Ok(v) => v,
        Err(e) => return e,
    };
    let pcd = prev_coupon_date(settlement, maturity, frequency);
    let ncd = next_coupon_date(settlement, maturity, frequency);
    let days = coupon_period_days(pcd, ncd, frequency, basis);
    Value::Number(days)
}

// ---------------------------------------------------------------------------
// COUPDAYSNC
// ---------------------------------------------------------------------------
/// `COUPDAYSNC(settlement, maturity, frequency, [basis])`
///
/// Days from settlement to the next coupon date.
pub fn coupdaysnc_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 3, 4) {
        return err;
    }
    let (settlement, maturity, frequency, basis) = match parse_coup_args(args) {
        Ok(v) => v,
        Err(e) => return e,
    };
    let ncd = next_coupon_date(settlement, maturity, frequency);
    let days = days_between(settlement, ncd, basis);
    Value::Number(days as f64)
}

// ---------------------------------------------------------------------------
// COUPNCD
// ---------------------------------------------------------------------------
/// `COUPNCD(settlement, maturity, frequency, [basis])`
///
/// Next coupon date after settlement. Returns a date serial number.
pub fn coupncd_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 3, 4) {
        return err;
    }
    let (settlement, maturity, frequency, _basis) = match parse_coup_args(args) {
        Ok(v) => v,
        Err(e) => return e,
    };
    let ncd = next_coupon_date(settlement, maturity, frequency);
    Value::Number(date_to_serial(ncd))
}

// ---------------------------------------------------------------------------
// COUPNUM
// ---------------------------------------------------------------------------
/// `COUPNUM(settlement, maturity, frequency, [basis])`
///
/// Number of coupons remaining between settlement and maturity.
pub fn coupnum_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 3, 4) {
        return err;
    }
    let (settlement, maturity, frequency, _basis) = match parse_coup_args(args) {
        Ok(v) => v,
        Err(e) => return e,
    };
    let mpp = months_per_period(frequency);
    // Count from NCD to maturity
    let ncd = next_coupon_date(settlement, maturity, frequency);
    // Number of coupon periods from ncd to maturity
    let mut count = 1u32;
    let mut cur = ncd;
    while cur < maturity {
        let next = add_months(cur, mpp);
        if next > maturity {
            break;
        }
        count += 1;
        cur = next;
    }
    Value::Number(count as f64)
}

// ---------------------------------------------------------------------------
// COUPPCD
// ---------------------------------------------------------------------------
/// `COUPPCD(settlement, maturity, frequency, [basis])`
///
/// Previous coupon date before or on settlement. Returns a date serial number.
pub fn couppcd_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 3, 4) {
        return err;
    }
    let (settlement, maturity, frequency, _basis) = match parse_coup_args(args) {
        Ok(v) => v,
        Err(e) => return e,
    };
    let pcd = prev_coupon_date(settlement, maturity, frequency);
    Value::Number(date_to_serial(pcd))
}

// ---------------------------------------------------------------------------
// Common coupon arg parsing
// ---------------------------------------------------------------------------
fn parse_coup_args(
    args: &[Value],
) -> Result<(NaiveDate, NaiveDate, u32, u32), Value> {
    let settlement_s = to_number(args[0].clone())?;
    let maturity_s = to_number(args[1].clone())?;
    let freq_f = to_number(args[2].clone())?;
    let basis_f = opt_number(args, 3, 0.0)?;

    let frequency = validate_frequency(freq_f)?;
    let basis = validate_basis(basis_f)?;

    let settlement = serial_to_date(settlement_s).ok_or(Value::Error(ErrorKind::Value))?;
    let maturity = serial_to_date(maturity_s).ok_or(Value::Error(ErrorKind::Value))?;

    if settlement >= maturity {
        return Err(Value::Error(ErrorKind::Num));
    }

    Ok((settlement, maturity, frequency, basis))
}

// ---------------------------------------------------------------------------
// DISC
// ---------------------------------------------------------------------------
/// `DISC(settlement, maturity, pr, redemption, [basis])`
///
/// Discount rate for a security. DISC = (redemption - pr) / redemption / yearfrac
pub fn disc_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 4, 5) {
        return err;
    }
    let settlement_s = match to_number(args[0].clone()) {
        Ok(n) => n,
        Err(e) => return e,
    };
    let maturity_s = match to_number(args[1].clone()) {
        Ok(n) => n,
        Err(e) => return e,
    };
    let pr = match to_number(args[2].clone()) {
        Ok(n) => n,
        Err(e) => return e,
    };
    let redemption = match to_number(args[3].clone()) {
        Ok(n) => n,
        Err(e) => return e,
    };
    let basis_f = match opt_number(args, 4, 0.0) {
        Ok(n) => n,
        Err(e) => return e,
    };
    let basis = match validate_basis(basis_f) {
        Ok(b) => b,
        Err(e) => return e,
    };

    let settlement = match serial_to_date(settlement_s) {
        Some(d) => d,
        None => return Value::Error(ErrorKind::Value),
    };
    let maturity = match serial_to_date(maturity_s) {
        Some(d) => d,
        None => return Value::Error(ErrorKind::Value),
    };

    if settlement >= maturity {
        return Value::Error(ErrorKind::Num);
    }
    if redemption <= 0.0 {
        return Value::Error(ErrorKind::Num);
    }

    let yf = yearfrac(settlement, maturity, basis);
    if yf == 0.0 {
        return Value::Error(ErrorKind::Num);
    }

    let result = (redemption - pr) / redemption / yf;
    if !result.is_finite() {
        return Value::Error(ErrorKind::Num);
    }
    Value::Number(result)
}

// ---------------------------------------------------------------------------
// INTRATE
// ---------------------------------------------------------------------------
/// `INTRATE(settlement, maturity, investment, redemption, [basis])`
///
/// Interest rate for a fully invested security.
/// INTRATE = (redemption - investment) / investment / yearfrac
pub fn intrate_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 4, 5) {
        return err;
    }
    let settlement_s = match to_number(args[0].clone()) {
        Ok(n) => n,
        Err(e) => return e,
    };
    let maturity_s = match to_number(args[1].clone()) {
        Ok(n) => n,
        Err(e) => return e,
    };
    let investment = match to_number(args[2].clone()) {
        Ok(n) => n,
        Err(e) => return e,
    };
    let redemption = match to_number(args[3].clone()) {
        Ok(n) => n,
        Err(e) => return e,
    };
    let basis_f = match opt_number(args, 4, 0.0) {
        Ok(n) => n,
        Err(e) => return e,
    };
    let basis = match validate_basis(basis_f) {
        Ok(b) => b,
        Err(e) => return e,
    };

    let settlement = match serial_to_date(settlement_s) {
        Some(d) => d,
        None => return Value::Error(ErrorKind::Value),
    };
    let maturity = match serial_to_date(maturity_s) {
        Some(d) => d,
        None => return Value::Error(ErrorKind::Value),
    };

    if settlement >= maturity {
        return Value::Error(ErrorKind::Num);
    }
    if investment <= 0.0 {
        return Value::Error(ErrorKind::Num);
    }

    let yf = yearfrac(settlement, maturity, basis);
    if yf == 0.0 {
        return Value::Error(ErrorKind::Num);
    }

    let result = (redemption - investment) / investment / yf;
    if !result.is_finite() {
        return Value::Error(ErrorKind::Num);
    }
    Value::Number(result)
}

// ---------------------------------------------------------------------------
// PRICE
// ---------------------------------------------------------------------------
/// `PRICE(settlement, maturity, rate, yld, redemption, frequency, [basis])`
///
/// Price per $100 face value for a security that pays periodic interest.
pub fn price_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 6, 7) {
        return err;
    }
    let settlement_s = match to_number(args[0].clone()) {
        Ok(n) => n,
        Err(e) => return e,
    };
    let maturity_s = match to_number(args[1].clone()) {
        Ok(n) => n,
        Err(e) => return e,
    };
    let rate = match to_number(args[2].clone()) {
        Ok(n) => n,
        Err(e) => return e,
    };
    let yld = match to_number(args[3].clone()) {
        Ok(n) => n,
        Err(e) => return e,
    };
    let redemption = match to_number(args[4].clone()) {
        Ok(n) => n,
        Err(e) => return e,
    };
    let freq_f = match to_number(args[5].clone()) {
        Ok(n) => n,
        Err(e) => return e,
    };
    let basis_f = match opt_number(args, 6, 0.0) {
        Ok(n) => n,
        Err(e) => return e,
    };

    let frequency = match validate_frequency(freq_f) {
        Ok(f) => f,
        Err(e) => return e,
    };
    let basis = match validate_basis(basis_f) {
        Ok(b) => b,
        Err(e) => return e,
    };

    let settlement = match serial_to_date(settlement_s) {
        Some(d) => d,
        None => return Value::Error(ErrorKind::Value),
    };
    let maturity = match serial_to_date(maturity_s) {
        Some(d) => d,
        None => return Value::Error(ErrorKind::Value),
    };

    if settlement >= maturity {
        return Value::Error(ErrorKind::Num);
    }
    if rate < 0.0 || yld < 0.0 || redemption <= 0.0 {
        return Value::Error(ErrorKind::Num);
    }

    let result = price_calc(settlement, maturity, rate, yld, redemption, frequency, basis);
    if !result.is_finite() {
        return Value::Error(ErrorKind::Num);
    }
    Value::Number(result)
}

/// Core PRICE calculation (also used by YIELD iteration).
pub fn price_calc(
    settlement: NaiveDate,
    maturity: NaiveDate,
    rate: f64,
    yld: f64,
    redemption: f64,
    frequency: u32,
    basis: u32,
) -> f64 {
    let freq = frequency as f64;
    let coupon = rate * 100.0 / freq;
    let pcd = prev_coupon_date(settlement, maturity, frequency);
    let ncd = next_coupon_date(settlement, maturity, frequency);
    let period_days = coupon_period_days(pcd, ncd, frequency, basis);
    let days_to_ncd = days_between(settlement, ncd, basis) as f64;

    // DSC/E: fractional periods from settlement to next coupon
    let dsc_e = days_to_ncd / period_days;

    // Number of coupons
    let n = {
        let mut count = 1i32;
        let mpp = months_per_period(frequency);
        let mut cur = ncd;
        while cur < maturity {
            let next = add_months(cur, mpp);
            if next > maturity {
                break;
            }
            count += 1;
            cur = next;
        }
        count as f64
    };

    let yld_f = yld / freq;
    let a = coupon / (1.0 + yld_f).powf(dsc_e);

    if (1.0 + yld_f) == 1.0 {
        // yld = 0 case
        let coupon_sum = coupon * n;
        return redemption + coupon_sum - coupon * (1.0 - dsc_e);
    }

    // Price = redemption / (1+yld/f)^(n-1+dsc_e) + sum of coupons
    let mut price = redemption / (1.0 + yld_f).powf(n - 1.0 + dsc_e);

    // Sum coupon PVs
    for k in 1..=(n as i32) {
        price += coupon / (1.0 + yld_f).powf(k as f64 - 1.0 + dsc_e);
    }
    let _ = a;
    price
}

// ---------------------------------------------------------------------------
// PRICEDISC
// ---------------------------------------------------------------------------
/// `PRICEDISC(settlement, maturity, discount, redemption, [basis])`
///
/// Price of a discounted security.
/// PRICEDISC = redemption * (1 - discount * yearfrac)
pub fn pricedisc_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 4, 5) {
        return err;
    }
    let settlement_s = match to_number(args[0].clone()) {
        Ok(n) => n,
        Err(e) => return e,
    };
    let maturity_s = match to_number(args[1].clone()) {
        Ok(n) => n,
        Err(e) => return e,
    };
    let discount = match to_number(args[2].clone()) {
        Ok(n) => n,
        Err(e) => return e,
    };
    let redemption = match to_number(args[3].clone()) {
        Ok(n) => n,
        Err(e) => return e,
    };
    let basis_f = match opt_number(args, 4, 0.0) {
        Ok(n) => n,
        Err(e) => return e,
    };
    let basis = match validate_basis(basis_f) {
        Ok(b) => b,
        Err(e) => return e,
    };

    let settlement = match serial_to_date(settlement_s) {
        Some(d) => d,
        None => return Value::Error(ErrorKind::Value),
    };
    let maturity = match serial_to_date(maturity_s) {
        Some(d) => d,
        None => return Value::Error(ErrorKind::Value),
    };

    if settlement >= maturity {
        return Value::Error(ErrorKind::Num);
    }
    if redemption <= 0.0 {
        return Value::Error(ErrorKind::Num);
    }

    let yf = yearfrac(settlement, maturity, basis);
    let result = redemption * (1.0 - discount * yf);
    if !result.is_finite() {
        return Value::Error(ErrorKind::Num);
    }
    Value::Number(result)
}

// ---------------------------------------------------------------------------
// PRICEMAT
// ---------------------------------------------------------------------------
/// `PRICEMAT(settlement, maturity, issue, rate, yld, [basis])`
///
/// Price at maturity for a security that pays interest at maturity.
pub fn pricemat_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 5, 6) {
        return err;
    }
    let settlement_s = match to_number(args[0].clone()) {
        Ok(n) => n,
        Err(e) => return e,
    };
    let maturity_s = match to_number(args[1].clone()) {
        Ok(n) => n,
        Err(e) => return e,
    };
    let issue_s = match to_number(args[2].clone()) {
        Ok(n) => n,
        Err(e) => return e,
    };
    let rate = match to_number(args[3].clone()) {
        Ok(n) => n,
        Err(e) => return e,
    };
    let yld = match to_number(args[4].clone()) {
        Ok(n) => n,
        Err(e) => return e,
    };
    let basis_f = match opt_number(args, 5, 0.0) {
        Ok(n) => n,
        Err(e) => return e,
    };
    let basis = match validate_basis(basis_f) {
        Ok(b) => b,
        Err(e) => return e,
    };

    let settlement = match serial_to_date(settlement_s) {
        Some(d) => d,
        None => return Value::Error(ErrorKind::Value),
    };
    let maturity = match serial_to_date(maturity_s) {
        Some(d) => d,
        None => return Value::Error(ErrorKind::Value),
    };
    let issue = match serial_to_date(issue_s) {
        Some(d) => d,
        None => return Value::Error(ErrorKind::Value),
    };

    if settlement >= maturity {
        return Value::Error(ErrorKind::Num);
    }

    // dim = yearfrac(issue, maturity)
    // dis = yearfrac(issue, settlement)
    // dsm = yearfrac(settlement, maturity)
    let dim = yearfrac(issue, maturity, basis);
    let dis = yearfrac(issue, settlement, basis);
    let dsm = yearfrac(settlement, maturity, basis);

    let numerator = 1.0 + dim * rate;
    let denominator = 1.0 + dsm * yld;

    if denominator == 0.0 {
        return Value::Error(ErrorKind::Num);
    }

    // PRICEMAT = 100 * (numerator / denominator - dis * rate)
    let result = 100.0 * (numerator / denominator - dis * rate);
    if !result.is_finite() {
        return Value::Error(ErrorKind::Num);
    }
    Value::Number(result)
}

// ---------------------------------------------------------------------------
// RECEIVED
// ---------------------------------------------------------------------------
/// `RECEIVED(settlement, maturity, investment, discount, [basis])`
///
/// Amount received at maturity for a fully invested security.
/// RECEIVED = investment / (1 - discount * yearfrac)
pub fn received_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 4, 5) {
        return err;
    }
    let settlement_s = match to_number(args[0].clone()) {
        Ok(n) => n,
        Err(e) => return e,
    };
    let maturity_s = match to_number(args[1].clone()) {
        Ok(n) => n,
        Err(e) => return e,
    };
    let investment = match to_number(args[2].clone()) {
        Ok(n) => n,
        Err(e) => return e,
    };
    let discount = match to_number(args[3].clone()) {
        Ok(n) => n,
        Err(e) => return e,
    };
    let basis_f = match opt_number(args, 4, 0.0) {
        Ok(n) => n,
        Err(e) => return e,
    };
    let basis = match validate_basis(basis_f) {
        Ok(b) => b,
        Err(e) => return e,
    };

    let settlement = match serial_to_date(settlement_s) {
        Some(d) => d,
        None => return Value::Error(ErrorKind::Value),
    };
    let maturity = match serial_to_date(maturity_s) {
        Some(d) => d,
        None => return Value::Error(ErrorKind::Value),
    };

    if settlement >= maturity {
        return Value::Error(ErrorKind::Num);
    }

    let yf = yearfrac(settlement, maturity, basis);
    let denom = 1.0 - discount * yf;
    if denom <= 0.0 {
        return Value::Error(ErrorKind::Num);
    }

    let result = investment / denom;
    if !result.is_finite() {
        return Value::Error(ErrorKind::Num);
    }
    Value::Number(result)
}

// ---------------------------------------------------------------------------
// T-Bill functions
// ---------------------------------------------------------------------------

/// Days between settlement and maturity for T-Bill (actual days).
fn tbill_dsm(settlement: NaiveDate, maturity: NaiveDate) -> f64 {
    actual_days(settlement, maturity) as f64
}

/// `TBILLPRICE(settlement, maturity, discount)`
///
/// Price of a T-Bill per $100 face value.
/// TBILLPRICE = 100 * (1 - discount * DSM / 360)
/// Requires discount > 0, maturity within 1 year, maturity > settlement.
pub fn tbillprice_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 3, 3) {
        return err;
    }
    let settlement_s = match to_number(args[0].clone()) {
        Ok(n) => n,
        Err(e) => return e,
    };
    let maturity_s = match to_number(args[1].clone()) {
        Ok(n) => n,
        Err(e) => return e,
    };
    let discount = match to_number(args[2].clone()) {
        Ok(n) => n,
        Err(e) => return e,
    };

    let settlement = match serial_to_date(settlement_s) {
        Some(d) => d,
        None => return Value::Error(ErrorKind::Value),
    };
    let maturity = match serial_to_date(maturity_s) {
        Some(d) => d,
        None => return Value::Error(ErrorKind::Value),
    };

    if settlement >= maturity {
        return Value::Error(ErrorKind::Num);
    }
    if discount <= 0.0 {
        return Value::Error(ErrorKind::Num);
    }

    let dsm = tbill_dsm(settlement, maturity);
    // T-Bill maturity must be within 1 year (<=366 days)
    if dsm > 366.0 {
        return Value::Error(ErrorKind::Num);
    }

    let result = 100.0 * (1.0 - discount * dsm / 360.0);
    if !result.is_finite() || result <= 0.0 {
        return Value::Error(ErrorKind::Num);
    }
    Value::Number(result)
}

/// `TBILLYIELD(settlement, maturity, pr)`
///
/// Yield for a T-Bill.
/// TBILLYIELD = (100 - pr) / pr * 360 / DSM
pub fn tbillyield_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 3, 3) {
        return err;
    }
    let settlement_s = match to_number(args[0].clone()) {
        Ok(n) => n,
        Err(e) => return e,
    };
    let maturity_s = match to_number(args[1].clone()) {
        Ok(n) => n,
        Err(e) => return e,
    };
    let pr = match to_number(args[2].clone()) {
        Ok(n) => n,
        Err(e) => return e,
    };

    let settlement = match serial_to_date(settlement_s) {
        Some(d) => d,
        None => return Value::Error(ErrorKind::Value),
    };
    let maturity = match serial_to_date(maturity_s) {
        Some(d) => d,
        None => return Value::Error(ErrorKind::Value),
    };

    if settlement >= maturity {
        return Value::Error(ErrorKind::Num);
    }
    if pr <= 0.0 {
        return Value::Error(ErrorKind::Num);
    }

    let dsm = tbill_dsm(settlement, maturity);
    let result = (100.0 - pr) / pr * 360.0 / dsm;
    if !result.is_finite() {
        return Value::Error(ErrorKind::Num);
    }
    Value::Number(result)
}

/// `TBILLEQ(settlement, maturity, discount)`
///
/// Bond-equivalent yield for a T-Bill.
/// For DSM <= 182: TBILLEQ = 365 * discount / (360 - discount * DSM)
/// For DSM > 182: uses compound formula.
pub fn tbilleq_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 3, 3) {
        return err;
    }
    let settlement_s = match to_number(args[0].clone()) {
        Ok(n) => n,
        Err(e) => return e,
    };
    let maturity_s = match to_number(args[1].clone()) {
        Ok(n) => n,
        Err(e) => return e,
    };
    let discount = match to_number(args[2].clone()) {
        Ok(n) => n,
        Err(e) => return e,
    };

    let settlement = match serial_to_date(settlement_s) {
        Some(d) => d,
        None => return Value::Error(ErrorKind::Value),
    };
    let maturity = match serial_to_date(maturity_s) {
        Some(d) => d,
        None => return Value::Error(ErrorKind::Value),
    };

    if settlement >= maturity {
        return Value::Error(ErrorKind::Num);
    }
    if discount <= 0.0 {
        return Value::Error(ErrorKind::Num);
    }

    let dsm = tbill_dsm(settlement, maturity);
    // Max 1 year
    if dsm > 366.0 {
        return Value::Error(ErrorKind::Num);
    }

    let result = if dsm <= 182.0 {
        365.0 * discount / (360.0 - discount * dsm)
    } else {
        // For >182 days, Google Sheets still uses the simple formula in practice.
        let denom = 360.0 - discount * dsm;
        if denom == 0.0 {
            return Value::Error(ErrorKind::Num);
        }
        365.0 * discount / denom
    };

    if !result.is_finite() {
        return Value::Error(ErrorKind::Num);
    }
    Value::Number(result)
}

// ---------------------------------------------------------------------------
// YIELD
// ---------------------------------------------------------------------------
/// `YIELD(settlement, maturity, rate, pr, redemption, frequency, [basis])`
///
/// Yield of a security that pays periodic interest. Inverse of PRICE.
/// Uses Newton-Raphson iteration.
pub fn yield_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 6, 7) {
        return err;
    }
    let settlement_s = match to_number(args[0].clone()) {
        Ok(n) => n,
        Err(e) => return e,
    };
    let maturity_s = match to_number(args[1].clone()) {
        Ok(n) => n,
        Err(e) => return e,
    };
    let rate = match to_number(args[2].clone()) {
        Ok(n) => n,
        Err(e) => return e,
    };
    let pr = match to_number(args[3].clone()) {
        Ok(n) => n,
        Err(e) => return e,
    };
    let redemption = match to_number(args[4].clone()) {
        Ok(n) => n,
        Err(e) => return e,
    };
    let freq_f = match to_number(args[5].clone()) {
        Ok(n) => n,
        Err(e) => return e,
    };
    let basis_f = match opt_number(args, 6, 0.0) {
        Ok(n) => n,
        Err(e) => return e,
    };

    let frequency = match validate_frequency(freq_f) {
        Ok(f) => f,
        Err(e) => return e,
    };
    let basis = match validate_basis(basis_f) {
        Ok(b) => b,
        Err(e) => return e,
    };

    let settlement = match serial_to_date(settlement_s) {
        Some(d) => d,
        None => return Value::Error(ErrorKind::Value),
    };
    let maturity = match serial_to_date(maturity_s) {
        Some(d) => d,
        None => return Value::Error(ErrorKind::Value),
    };

    if settlement >= maturity {
        return Value::Error(ErrorKind::Num);
    }
    if rate < 0.0 || pr <= 0.0 || redemption <= 0.0 {
        return Value::Error(ErrorKind::Num);
    }

    // Newton-Raphson: find yld such that PRICE(yld) = pr
    let mut yld = rate; // Initial guess
    for _ in 0..200 {
        let p = price_calc(settlement, maturity, rate, yld, redemption, frequency, basis);
        let dp = price_calc(
            settlement,
            maturity,
            rate,
            yld + 1e-7,
            redemption,
            frequency,
            basis,
        );
        let deriv = (dp - p) / 1e-7;
        if !deriv.is_finite() || deriv.abs() < 1e-14 {
            break;
        }
        let new_yld = yld - (p - pr) / deriv;
        if !new_yld.is_finite() {
            return Value::Error(ErrorKind::Num);
        }
        if (new_yld - yld).abs() < 1e-10 {
            return Value::Number(new_yld);
        }
        yld = new_yld;
        if yld < -1.0 {
            yld = 0.0;
        }
    }

    let final_p = price_calc(settlement, maturity, rate, yld, redemption, frequency, basis);
    if (final_p - pr).abs() > 1e-4 * pr.abs() + 1e-6 {
        return Value::Error(ErrorKind::Num);
    }
    Value::Number(yld)
}

// ---------------------------------------------------------------------------
// YIELDDISC
// ---------------------------------------------------------------------------
/// `YIELDDISC(settlement, maturity, pr, redemption, [basis])`
///
/// Annual yield for a discounted security.
/// YIELDDISC = (redemption - pr) / pr / yearfrac
pub fn yielddisc_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 4, 5) {
        return err;
    }
    let settlement_s = match to_number(args[0].clone()) {
        Ok(n) => n,
        Err(e) => return e,
    };
    let maturity_s = match to_number(args[1].clone()) {
        Ok(n) => n,
        Err(e) => return e,
    };
    let pr = match to_number(args[2].clone()) {
        Ok(n) => n,
        Err(e) => return e,
    };
    let redemption = match to_number(args[3].clone()) {
        Ok(n) => n,
        Err(e) => return e,
    };
    let basis_f = match opt_number(args, 4, 0.0) {
        Ok(n) => n,
        Err(e) => return e,
    };
    let basis = match validate_basis(basis_f) {
        Ok(b) => b,
        Err(e) => return e,
    };

    let settlement = match serial_to_date(settlement_s) {
        Some(d) => d,
        None => return Value::Error(ErrorKind::Value),
    };
    let maturity = match serial_to_date(maturity_s) {
        Some(d) => d,
        None => return Value::Error(ErrorKind::Value),
    };

    if settlement >= maturity {
        return Value::Error(ErrorKind::Num);
    }
    if pr <= 0.0 {
        return Value::Error(ErrorKind::Num);
    }
    // pr >= redemption means zero or negative yield — allowed (returns 0 or negative), but
    // let's match Google Sheets: pr >= redemption gives 0 (not NUM), per fixture row11.
    // The test says YIELDDISC(date,date,100,100) = 0, not #NUM!

    let yf = yearfrac(settlement, maturity, basis);
    if yf == 0.0 {
        return Value::Error(ErrorKind::Num);
    }

    let result = (redemption - pr) / pr / yf;
    if !result.is_finite() {
        return Value::Error(ErrorKind::Num);
    }
    Value::Number(result)
}

// ---------------------------------------------------------------------------
// YIELDMAT
// ---------------------------------------------------------------------------
/// `YIELDMAT(settlement, maturity, issue, rate, pr, [basis])`
///
/// Annual yield for a security that pays interest at maturity.
pub fn yieldmat_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 5, 6) {
        return err;
    }
    let settlement_s = match to_number(args[0].clone()) {
        Ok(n) => n,
        Err(e) => return e,
    };
    let maturity_s = match to_number(args[1].clone()) {
        Ok(n) => n,
        Err(e) => return e,
    };
    let issue_s = match to_number(args[2].clone()) {
        Ok(n) => n,
        Err(e) => return e,
    };
    let rate = match to_number(args[3].clone()) {
        Ok(n) => n,
        Err(e) => return e,
    };
    let pr = match to_number(args[4].clone()) {
        Ok(n) => n,
        Err(e) => return e,
    };
    let basis_f = match opt_number(args, 5, 0.0) {
        Ok(n) => n,
        Err(e) => return e,
    };
    let basis = match validate_basis(basis_f) {
        Ok(b) => b,
        Err(e) => return e,
    };

    let settlement = match serial_to_date(settlement_s) {
        Some(d) => d,
        None => return Value::Error(ErrorKind::Value),
    };
    let maturity = match serial_to_date(maturity_s) {
        Some(d) => d,
        None => return Value::Error(ErrorKind::Value),
    };
    let issue = match serial_to_date(issue_s) {
        Some(d) => d,
        None => return Value::Error(ErrorKind::Value),
    };

    if settlement >= maturity {
        return Value::Error(ErrorKind::Num);
    }
    if pr <= 0.0 {
        return Value::Error(ErrorKind::Num);
    }

    // YIELDMAT formula (inverse of PRICEMAT):
    // dim = yearfrac(issue, maturity)
    // dis = yearfrac(issue, settlement)
    // dsm = yearfrac(settlement, maturity)
    // numerator = (1 + dim * rate) / (pr/100 + dis * rate) - 1
    // YIELDMAT = numerator / dsm
    let dim = yearfrac(issue, maturity, basis);
    let dis = yearfrac(issue, settlement, basis);
    let dsm = yearfrac(settlement, maturity, basis);

    if dsm == 0.0 {
        return Value::Error(ErrorKind::Num);
    }

    let denom = pr / 100.0 + dis * rate;
    if denom == 0.0 {
        return Value::Error(ErrorKind::Num);
    }

    let result = ((1.0 + dim * rate) / denom - 1.0) / dsm;
    if !result.is_finite() {
        return Value::Error(ErrorKind::Num);
    }
    Value::Number(result)
}

#[cfg(test)]
mod tests;
