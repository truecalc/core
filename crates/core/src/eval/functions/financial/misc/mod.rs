/// Miscellaneous financial functions:
/// AMORLINC, CUMIPMT, CUMPRINC, DB, DDB, DOLLARDE, DOLLARFR,
/// DURATION, EFFECT, FVSCHEDULE, IPMT, ISPMT, MDURATION, MIRR,
/// NOMINAL, PDURATION, PPMT, RRI, SLN, SYD, VDB, XIRR, XNPV
use crate::eval::coercion::to_number;
use crate::eval::functions::check_arity;
use crate::types::{ErrorKind, Value};

fn opt_number(args: &[Value], idx: usize, default: f64) -> Result<f64, Value> {
    if idx < args.len() {
        to_number(args[idx].clone())
    } else {
        Ok(default)
    }
}

// ---------------------------------------------------------------------------
// PMT helper (used by IPMT, PPMT, CUMIPMT, CUMPRINC)
// ---------------------------------------------------------------------------
fn pmt_calc(rate: f64, nper: f64, pv: f64, fv: f64, typ: f64) -> f64 {
    if nper == 0.0 {
        return f64::NAN;
    }
    if rate == 0.0 {
        return -(pv + fv) / nper;
    }
    let factor = (1.0 + rate).powf(nper);
    let denom = factor - 1.0;
    if denom == 0.0 {
        return f64::NAN;
    }
    -(pv * rate * factor + fv * rate) / denom / (1.0 + rate * typ)
}

// ---------------------------------------------------------------------------
// IPMT
// ---------------------------------------------------------------------------
/// `IPMT(rate, per, nper, pv, [fv], [type])`
///
/// Interest payment for a given period of a loan/annuity.
pub fn ipmt_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 4, 6) {
        return err;
    }
    let rate = match to_number(args[0].clone()) { Ok(n) => n, Err(e) => return e };
    let per  = match to_number(args[1].clone()) { Ok(n) => n, Err(e) => return e };
    let nper = match to_number(args[2].clone()) { Ok(n) => n, Err(e) => return e };
    let pv   = match to_number(args[3].clone()) { Ok(n) => n, Err(e) => return e };
    let fv   = match opt_number(args, 4, 0.0) { Ok(n) => n, Err(e) => return e };
    let typ  = match opt_number(args, 5, 0.0) { Ok(n) => n, Err(e) => return e };

    if per < 1.0 || per > nper {
        return Value::Error(ErrorKind::Num);
    }

    if rate == 0.0 {
        return Value::Number(0.0);
    }

    let interest = ipmt_calc(rate, per, nper, pv, fv, typ);
    if !interest.is_finite() {
        return Value::Error(ErrorKind::Num);
    }
    Value::Number(interest)
}

/// Compute the interest component for period `per`.
///
/// For type=0 (end-of-period): interest = -(balance_at_start_of_per * rate)
/// For type=1 (beginning-of-period): IPMT(per=1)=0; IPMT(per>=2) = interest that
/// accrued during period per-1 = -(balance_after_payment_at_start_of_per-1) * rate
fn ipmt_calc(rate: f64, per: f64, nper: f64, pv: f64, fv: f64, typ: f64) -> f64 {
    if typ == 0.0 {
        // Balance at start of period per:
        // B = pv*(1+r)^(per-1) + pmt * ((1+r)^(per-1) - 1) / r
        let pmt = pmt_calc(rate, nper, pv, fv, 0.0);
        let k = per - 1.0;
        let factor_k = (1.0 + rate).powf(k);
        let bal = pv * factor_k + pmt * (factor_k - 1.0) / rate;
        -(bal * rate)
    } else {
        // type=1: IPMT(1) = 0, IPMT(k>=2) = interest accrued in period k-1
        if per == 1.0 {
            return 0.0;
        }
        let pmt1 = pmt_calc(rate, nper, pv, fv, 1.0);
        // Balance at start of period (per-1) before payment:
        // B = pv*(1+r)^(per-2) + pmt1*(1+r)*((1+r)^(per-2)-1)/r
        let k = per - 2.0;
        let factor_k = (1.0 + rate).powf(k);
        let bal_before = pv * factor_k + pmt1 * (1.0 + rate) * (factor_k - 1.0) / rate;
        // Balance after payment at start of period (per-1):
        let bal_after = bal_before + pmt1;
        -(bal_after * rate)
    }
}

// ---------------------------------------------------------------------------
// PPMT
// ---------------------------------------------------------------------------
/// `PPMT(rate, per, nper, pv, [fv], [type])`
///
/// Principal payment for a given period.
pub fn ppmt_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 4, 6) {
        return err;
    }
    let rate = match to_number(args[0].clone()) { Ok(n) => n, Err(e) => return e };
    let per  = match to_number(args[1].clone()) { Ok(n) => n, Err(e) => return e };
    let nper = match to_number(args[2].clone()) { Ok(n) => n, Err(e) => return e };
    let pv   = match to_number(args[3].clone()) { Ok(n) => n, Err(e) => return e };
    let fv   = match opt_number(args, 4, 0.0) { Ok(n) => n, Err(e) => return e };
    let typ  = match opt_number(args, 5, 0.0) { Ok(n) => n, Err(e) => return e };

    if per < 1.0 || per > nper {
        return Value::Error(ErrorKind::Num);
    }

    let pmt = pmt_calc(rate, nper, pv, fv, typ);
    if !pmt.is_finite() {
        return Value::Error(ErrorKind::Num);
    }

    // PPMT = PMT - IPMT
    let ipmt = match ipmt_fn(&[
        Value::Number(rate), Value::Number(per), Value::Number(nper),
        Value::Number(pv), Value::Number(fv), Value::Number(typ)
    ]) {
        Value::Number(n) => n,
        e => return e,
    };

    let ppmt = pmt - ipmt;
    if !ppmt.is_finite() {
        return Value::Error(ErrorKind::Num);
    }
    Value::Number(ppmt)
}

// ---------------------------------------------------------------------------
// CUMIPMT
// ---------------------------------------------------------------------------
/// `CUMIPMT(rate, nper, pv, start_period, end_period, type)`
///
/// Cumulative interest paid between start and end period (inclusive).
pub fn cumipmt_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 6, 6) {
        return err;
    }
    let rate   = match to_number(args[0].clone()) { Ok(n) => n, Err(e) => return e };
    let nper   = match to_number(args[1].clone()) { Ok(n) => n, Err(e) => return e };
    let pv     = match to_number(args[2].clone()) { Ok(n) => n, Err(e) => return e };
    let start  = match to_number(args[3].clone()) { Ok(n) => n, Err(e) => return e };
    let end    = match to_number(args[4].clone()) { Ok(n) => n, Err(e) => return e };
    let typ    = match to_number(args[5].clone()) { Ok(n) => n, Err(e) => return e };

    let start = start.floor() as i64;
    let end = end.floor() as i64;

    if start > end || start < 1 {
        return Value::Error(ErrorKind::Num);
    }

    let mut total = 0.0;
    for per in start..=end {
        let ipmt = match ipmt_fn(&[
            Value::Number(rate), Value::Number(per as f64), Value::Number(nper),
            Value::Number(pv), Value::Number(0.0), Value::Number(typ)
        ]) {
            Value::Number(n) => n,
            e => return e,
        };
        total += ipmt;
    }

    if !total.is_finite() {
        return Value::Error(ErrorKind::Num);
    }
    Value::Number(total)
}

// ---------------------------------------------------------------------------
// CUMPRINC
// ---------------------------------------------------------------------------
/// `CUMPRINC(rate, nper, pv, start_period, end_period, type)`
///
/// Cumulative principal paid between start and end period (inclusive).
pub fn cumprinc_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 6, 6) {
        return err;
    }
    let rate  = match to_number(args[0].clone()) { Ok(n) => n, Err(e) => return e };
    let nper  = match to_number(args[1].clone()) { Ok(n) => n, Err(e) => return e };
    let pv    = match to_number(args[2].clone()) { Ok(n) => n, Err(e) => return e };
    let start = match to_number(args[3].clone()) { Ok(n) => n, Err(e) => return e };
    let end   = match to_number(args[4].clone()) { Ok(n) => n, Err(e) => return e };
    let typ   = match to_number(args[5].clone()) { Ok(n) => n, Err(e) => return e };

    let start = start.floor() as i64;
    let end = end.floor() as i64;

    if start > end || start < 1 {
        return Value::Error(ErrorKind::Num);
    }

    let mut total = 0.0;
    for per in start..=end {
        let ppmt = match ppmt_fn(&[
            Value::Number(rate), Value::Number(per as f64), Value::Number(nper),
            Value::Number(pv), Value::Number(0.0), Value::Number(typ)
        ]) {
            Value::Number(n) => n,
            e => return e,
        };
        total += ppmt;
    }

    if !total.is_finite() {
        return Value::Error(ErrorKind::Num);
    }
    Value::Number(total)
}

// ---------------------------------------------------------------------------
// ISPMT
// ---------------------------------------------------------------------------
/// `ISPMT(rate, per, nper, pv)`
///
/// Interest paid for a straight-line loan (ISPMT, not reducing balance).
/// ISPMT = pv * rate * (1 - per/nper)
pub fn ispmt_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 4, 4) {
        return err;
    }
    let rate = match to_number(args[0].clone()) { Ok(n) => n, Err(e) => return e };
    let per  = match to_number(args[1].clone()) { Ok(n) => n, Err(e) => return e };
    let nper = match to_number(args[2].clone()) { Ok(n) => n, Err(e) => return e };
    let pv   = match to_number(args[3].clone()) { Ok(n) => n, Err(e) => return e };

    if nper == 0.0 {
        return Value::Error(ErrorKind::DivByZero);
    }

    // ISPMT uses the same sign convention as PMT: negative for outflows.
    // ISPMT = -pv * rate * (1 - per/nper)
    let result = -(pv * rate * (1.0 - per / nper));
    if !result.is_finite() {
        return Value::Error(ErrorKind::Num);
    }
    Value::Number(result)
}

// ---------------------------------------------------------------------------
// SLN
// ---------------------------------------------------------------------------
/// `SLN(cost, salvage, life)`
///
/// Straight-line depreciation.
pub fn sln_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 3, 3) {
        return err;
    }
    let cost    = match to_number(args[0].clone()) { Ok(n) => n, Err(e) => return e };
    let salvage = match to_number(args[1].clone()) { Ok(n) => n, Err(e) => return e };
    let life    = match to_number(args[2].clone()) { Ok(n) => n, Err(e) => return e };

    if life == 0.0 {
        return Value::Error(ErrorKind::DivByZero);
    }

    let result = (cost - salvage) / life;
    if !result.is_finite() {
        return Value::Error(ErrorKind::Num);
    }
    Value::Number(result)
}

// ---------------------------------------------------------------------------
// SYD
// ---------------------------------------------------------------------------
/// `SYD(cost, salvage, life, per)`
///
/// Sum-of-years-digits depreciation.
pub fn syd_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 4, 4) {
        return err;
    }
    let cost    = match to_number(args[0].clone()) { Ok(n) => n, Err(e) => return e };
    let salvage = match to_number(args[1].clone()) { Ok(n) => n, Err(e) => return e };
    let life    = match to_number(args[2].clone()) { Ok(n) => n, Err(e) => return e };
    let per     = match to_number(args[3].clone()) { Ok(n) => n, Err(e) => return e };

    if per > life || per <= 0.0 || life <= 0.0 {
        return Value::Error(ErrorKind::Num);
    }

    // SYD = (cost - salvage) * (life - per + 1) / (life * (life + 1) / 2)
    let result = (cost - salvage) * (life - per + 1.0) / (life * (life + 1.0) / 2.0);
    if !result.is_finite() {
        return Value::Error(ErrorKind::Num);
    }
    Value::Number(result)
}

// ---------------------------------------------------------------------------
// DDB
// ---------------------------------------------------------------------------
/// `DDB(cost, salvage, life, period, [factor])`
///
/// Double-declining-balance depreciation.
pub fn ddb_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 4, 5) {
        return err;
    }
    let cost    = match to_number(args[0].clone()) { Ok(n) => n, Err(e) => return e };
    let salvage = match to_number(args[1].clone()) { Ok(n) => n, Err(e) => return e };
    let life    = match to_number(args[2].clone()) { Ok(n) => n, Err(e) => return e };
    let period  = match to_number(args[3].clone()) { Ok(n) => n, Err(e) => return e };
    let factor  = match opt_number(args, 4, 2.0) { Ok(n) => n, Err(e) => return e };

    if period > life || life <= 0.0 {
        return Value::Error(ErrorKind::Num);
    }

    // Simulate period-by-period to handle salvage floor
    let rate = factor / life;
    let mut book = cost;
    let mut dep = 0.0;
    let per = period.floor() as i64;
    for _p in 1..=per {
        let d = book * rate;
        // Can't depreciate below salvage
        dep = if book - d < salvage { book - salvage } else { d };
        if dep < 0.0 { dep = 0.0; }
        book -= dep;
    }

    if !dep.is_finite() {
        return Value::Error(ErrorKind::Num);
    }
    Value::Number(dep)
}

// ---------------------------------------------------------------------------
// DB
// ---------------------------------------------------------------------------
/// `DB(cost, salvage, life, period, [month])`
///
/// Fixed-declining-balance depreciation.
///
/// Depreciation rate = 1 - (salvage/cost)^(1/life), rounded to 3 decimal places.
pub fn db_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 4, 5) {
        return err;
    }
    let cost    = match to_number(args[0].clone()) { Ok(n) => n, Err(e) => return e };
    let salvage = match to_number(args[1].clone()) { Ok(n) => n, Err(e) => return e };
    let life    = match to_number(args[2].clone()) { Ok(n) => n, Err(e) => return e };
    let period  = match to_number(args[3].clone()) { Ok(n) => n, Err(e) => return e };
    let month   = match opt_number(args, 4, 12.0) { Ok(n) => n, Err(e) => return e };

    let life_i = life.floor() as i64;
    let period_i = period.floor() as i64;
    let month_i = month.floor() as i64;

    if life_i <= 0 || cost <= 0.0 {
        return Value::Error(ErrorKind::Num);
    }

    // Special case: zero salvage
    if salvage == 0.0 {
        if period_i == 1 {
            return Value::Number(cost);
        }
        if period_i > life_i {
            return Value::Error(ErrorKind::Num);
        }
        // All depreciation in period 1
        return Value::Number(0.0);
    }

    // Rate = 1 - (salvage/cost)^(1/life), rounded to 3 decimal places
    let rate = {
        let r = 1.0 - (salvage / cost).powf(1.0 / life);
        (r * 1000.0).round() / 1000.0
    };

    if period_i > life_i + 1 {
        return Value::Error(ErrorKind::Num);
    }

    let mut book = cost;
    let mut dep = 0.0;

    for p in 1..=(life_i + 1) {
        dep = if p == 1 {
            cost * rate * month_i as f64 / 12.0
        } else if p == life_i + 1 {
            (book - salvage) * rate * (12.0 - month_i as f64) / 12.0
        } else {
            book * rate
        };
        if p == period_i {
            break;
        }
        book -= dep;
    }

    if !dep.is_finite() {
        return Value::Error(ErrorKind::Num);
    }
    Value::Number(dep)
}

// ---------------------------------------------------------------------------
// VDB
// ---------------------------------------------------------------------------
/// `VDB(cost, salvage, life, start_period, end_period, [factor], [no_switch])`
///
/// Variable-rate declining balance depreciation.
/// Switches to straight-line when SL > DDB (unless no_switch=TRUE).
pub fn vdb_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 5, 7) {
        return err;
    }
    let cost         = match to_number(args[0].clone()) { Ok(n) => n, Err(e) => return e };
    let salvage      = match to_number(args[1].clone()) { Ok(n) => n, Err(e) => return e };
    let life         = match to_number(args[2].clone()) { Ok(n) => n, Err(e) => return e };
    let start_period = match to_number(args[3].clone()) { Ok(n) => n, Err(e) => return e };
    let end_period   = match to_number(args[4].clone()) { Ok(n) => n, Err(e) => return e };
    let factor       = match opt_number(args, 5, 2.0) { Ok(n) => n, Err(e) => return e };
    // no_switch: for booleans we need special handling
    let no_switch = if args.len() > 6 {
        match &args[6] {
            Value::Bool(b) => *b,
            Value::Number(n) => *n != 0.0,
            Value::Empty => false,
            Value::Error(e) => return Value::Error(e.clone()),
            _ => false,
        }
    } else {
        false
    };

    if start_period > end_period {
        return Value::Error(ErrorKind::Num);
    }
    if life <= 0.0 {
        return Value::Error(ErrorKind::Num);
    }

    // Compute depreciation for fractional periods using the declining balance method
    let vdb_period = |start: f64, end: f64, cost: f64, salvage: f64| -> f64 {
        if start >= end {
            return 0.0;
        }
        let rate = factor / life;
        // We need to compute depreciation from period start to end.
        // For integer period: dep_at_period(p) with book value after p periods
        // First compute book value at start
        let book_at = |p: f64| -> f64 {
            if p <= 0.0 {
                return cost;
            }
            // Book value = cost * (1 - rate)^p but capped at salvage
            let bv = cost * (1.0 - rate).powf(p);
            if bv < salvage { salvage } else { bv }
        };

        // For fractional periods, compute integral of depreciation over [start, end]
        // This is book(start) - book(end) but with possible switch to SL
        if no_switch {
            // No SL switch: pure declining balance
            let bv_start = book_at(start);
            let bv_end = book_at(end);
            let dep = bv_start - bv_end;
            if dep < 0.0 { 0.0 } else { dep }
        } else {
            // With SL switch: iterate over integer periods in [start, end]
            // and switch to SL when SL >= DDB
            let s = start.floor() as i64;
            let e = end.floor() as i64;
            let mut total = 0.0;
            let mut book = book_at(start);
            let mut switched = false;

            // Handle fractional first period [start, ceil(start)]
            let first_end = (s + 1) as f64;
            let first_end = first_end.min(end);
            let first_frac = first_end - start;

            if first_frac > 0.0 {
                let ddb = book * rate * first_frac;
                let remaining_life = life - start;
                let sl = if remaining_life > 0.0 { (book - salvage) / remaining_life * first_frac } else { 0.0 };
                let dep = if !no_switch && sl >= ddb { switched = true; sl } else { ddb };
                let dep = dep.min(book - salvage);
                let dep = dep.max(0.0);
                total += dep;
                book -= dep;
            }

            // Handle full integer periods
            for p in (s + 1)..=e {
                let period_end = ((p + 1) as f64).min(end);
                let frac = period_end - p as f64;
                if frac <= 0.0 { break; }

                let ddb = book * rate * frac;
                let remaining_life = life - p as f64;
                let sl = if remaining_life > 0.0 { (book - salvage) / remaining_life * frac } else { 0.0 };
                let dep = if switched || sl >= ddb { switched = true; sl } else { ddb };
                let dep = dep.min(book - salvage);
                let dep = dep.max(0.0);
                total += dep;
                book -= dep;
            }

            total
        }
    };

    let result = vdb_period(start_period, end_period, cost, salvage);
    if !result.is_finite() {
        return Value::Error(ErrorKind::Num);
    }
    Value::Number(result)
}

// ---------------------------------------------------------------------------
// AMORLINC
// ---------------------------------------------------------------------------
/// `AMORLINC(cost, date_purchased, first_period, salvage, period, rate, [basis])`
///
/// French linear depreciation for each accounting period.
pub fn amorlinc_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 6, 7) {
        return err;
    }
    let cost           = match to_number(args[0].clone()) { Ok(n) => n, Err(e) => return e };
    let _date_purchased = match to_number(args[1].clone()) { Ok(n) => n, Err(e) => return e };
    let _first_period  = match to_number(args[2].clone()) { Ok(n) => n, Err(e) => return e };
    let salvage        = match to_number(args[3].clone()) { Ok(n) => n, Err(e) => return e };
    let period         = match to_number(args[4].clone()) { Ok(n) => n, Err(e) => return e };
    let rate           = match to_number(args[5].clone()) { Ok(n) => n, Err(e) => return e };
    let _basis         = match opt_number(args, 6, 0.0) { Ok(n) => n, Err(e) => return e };

    if cost < salvage {
        return Value::Error(ErrorKind::Num);
    }
    if rate <= 0.0 || cost <= 0.0 {
        return Value::Error(ErrorKind::Num);
    }

    let per = period.floor() as i64;
    // AMORLINC: each full period depreciates at rate * cost
    // Period 1: cost * rate (for the full first year of accounting period)
    // Subsequent periods: cost * rate, until fully depreciated to salvage
    let dep_per_period = cost * rate;
    // Excel AMORLINC: life = INT((cost-salvage)/dep_per_period); period >= life returns 0
    let life = ((cost - salvage) / dep_per_period).floor() as i64;

    if per < 1 || per >= life {
        return Value::Number(0.0);
    }

    // Calculate cumulative depreciation before this period
    let cumulative_before = dep_per_period * (per - 1) as f64;
    let remaining = cost - salvage - cumulative_before;

    if remaining <= 0.0 {
        return Value::Number(0.0);
    }

    // This period's depreciation (capped at remaining)
    let dep = dep_per_period.min(remaining);
    Value::Number(dep)
}

// ---------------------------------------------------------------------------
// DOLLARDE
// ---------------------------------------------------------------------------
/// `DOLLARDE(fractional_dollar, fraction)`
///
/// Converts from fractional notation (e.g. 1.02 = 1 + 2/16) to decimal.
pub fn dollarde_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 2, 2) {
        return err;
    }
    let dollar   = match to_number(args[0].clone()) { Ok(n) => n, Err(e) => return e };
    let fraction = match to_number(args[1].clone()) { Ok(n) => n, Err(e) => return e };

    let fraction = fraction.floor() as i64;
    if fraction == 0 {
        return Value::Error(ErrorKind::DivByZero);
    }
    if fraction < 0 {
        return Value::Error(ErrorKind::Num);
    }

    let int_part = dollar.trunc();
    let frac_part = (dollar - int_part).abs();

    // DOLLARDE: decimal part is treated as numerator/fraction.
    // E.g. DOLLARDE(1.02, 16) = 1 + 2/16 = 1.125
    // The scale is 10^ceil(log10(fraction)) so:
    //   fraction=8  → scale=10  → numerator = 0.1 * 10 = 1
    //   fraction=16 → scale=100 → numerator = 0.02 * 100 = 2
    let scale = {
        let mut s = 1i64;
        while s < fraction { s *= 10; }
        s as f64
    };
    let numerator = (frac_part * scale).round();
    let result = int_part + (if dollar < 0.0 { -1.0 } else { 1.0 }) * numerator / fraction as f64;

    if !result.is_finite() {
        return Value::Error(ErrorKind::Num);
    }
    Value::Number(result)
}

// ---------------------------------------------------------------------------
// DOLLARFR
// ---------------------------------------------------------------------------
/// `DOLLARFR(decimal_dollar, fraction)`
///
/// Converts from decimal notation to fractional notation.
pub fn dollarfr_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 2, 2) {
        return err;
    }
    let dollar   = match to_number(args[0].clone()) { Ok(n) => n, Err(e) => return e };
    let fraction = match to_number(args[1].clone()) { Ok(n) => n, Err(e) => return e };

    let fraction = fraction.floor() as i64;
    if fraction == 0 {
        return Value::Error(ErrorKind::DivByZero);
    }
    if fraction < 0 {
        return Value::Error(ErrorKind::Num);
    }

    let int_part = dollar.trunc();
    let frac_part = (dollar - int_part).abs();

    // DOLLARFR: inverse of DOLLARDE. Convert decimal fraction to xxx/fraction notation.
    // E.g. DOLLARFR(1.125, 16) = 1 + 2/100 = 1.02  (since 0.125 * 16 = 2 → write as .02)
    // numerator = frac_part * fraction, then express as decimal in 10^ceil(log10(fraction)) scale
    let scale = {
        let mut s = 1i64;
        while s < fraction { s *= 10; }
        s as f64
    };
    let numerator = (frac_part * fraction as f64).round();
    let result = int_part + (if dollar < 0.0 { -1.0 } else { 1.0 }) * numerator / scale;

    if !result.is_finite() {
        return Value::Error(ErrorKind::Num);
    }
    Value::Number(result)
}

// ---------------------------------------------------------------------------
// EFFECT
// ---------------------------------------------------------------------------
/// `EFFECT(nominal_rate, npery)`
///
/// Effective annual interest rate. EFFECT = (1 + nominal_rate/npery)^npery - 1
pub fn effect_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 2, 2) {
        return err;
    }
    let nominal = match to_number(args[0].clone()) { Ok(n) => n, Err(e) => return e };
    let npery   = match to_number(args[1].clone()) { Ok(n) => n, Err(e) => return e };

    let npery = npery.floor() as i64;
    if npery < 1 || nominal <= 0.0 {
        return Value::Error(ErrorKind::Num);
    }

    let result = (1.0 + nominal / npery as f64).powi(npery as i32) - 1.0;
    if !result.is_finite() {
        return Value::Error(ErrorKind::Num);
    }
    Value::Number(result)
}

// ---------------------------------------------------------------------------
// NOMINAL
// ---------------------------------------------------------------------------
/// `NOMINAL(effect_rate, npery)`
///
/// Nominal annual interest rate. NOMINAL = npery * ((1 + effect_rate)^(1/npery) - 1)
pub fn nominal_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 2, 2) {
        return err;
    }
    let effect = match to_number(args[0].clone()) { Ok(n) => n, Err(e) => return e };
    let npery  = match to_number(args[1].clone()) { Ok(n) => n, Err(e) => return e };

    let npery = npery.floor() as i64;
    if npery < 1 || effect <= 0.0 {
        return Value::Error(ErrorKind::Num);
    }

    let result = npery as f64 * ((1.0 + effect).powf(1.0 / npery as f64) - 1.0);
    if !result.is_finite() {
        return Value::Error(ErrorKind::Num);
    }
    Value::Number(result)
}

// ---------------------------------------------------------------------------
// PDURATION
// ---------------------------------------------------------------------------
/// `PDURATION(rate, pv, fv)`
///
/// Number of periods for an investment to reach a target value.
/// PDURATION = log(fv/pv) / log(1+rate)
pub fn pduration_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 3, 3) {
        return err;
    }
    let rate = match to_number(args[0].clone()) { Ok(n) => n, Err(e) => return e };
    let pv   = match to_number(args[1].clone()) { Ok(n) => n, Err(e) => return e };
    let fv   = match to_number(args[2].clone()) { Ok(n) => n, Err(e) => return e };

    if rate <= 0.0 || pv <= 0.0 || fv <= 0.0 {
        return Value::Error(ErrorKind::Num);
    }

    let result = (fv / pv).ln() / (1.0 + rate).ln();
    if !result.is_finite() {
        return Value::Error(ErrorKind::Num);
    }
    Value::Number(result)
}

// ---------------------------------------------------------------------------
// RRI
// ---------------------------------------------------------------------------
/// `RRI(nper, pv, fv)`
///
/// Equivalent interest rate for growth of investment.
/// RRI = (fv/pv)^(1/nper) - 1
pub fn rri_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 3, 3) {
        return err;
    }
    let nper = match to_number(args[0].clone()) { Ok(n) => n, Err(e) => return e };
    let pv   = match to_number(args[1].clone()) { Ok(n) => n, Err(e) => return e };
    let fv   = match to_number(args[2].clone()) { Ok(n) => n, Err(e) => return e };

    if nper <= 0.0 {
        return Value::Error(ErrorKind::DivByZero);
    }
    if pv <= 0.0 || fv < 0.0 {
        return Value::Error(ErrorKind::Num);
    }
    if pv == 0.0 {
        return Value::Error(ErrorKind::DivByZero);
    }

    let result = (fv / pv).powf(1.0 / nper) - 1.0;
    if !result.is_finite() {
        return Value::Error(ErrorKind::Num);
    }
    Value::Number(result)
}

// ---------------------------------------------------------------------------
// DURATION
// ---------------------------------------------------------------------------
/// `DURATION(settlement, maturity, coupon, yld, frequency, [basis])`
///
/// Macaulay duration of a bond (weighted average time to cash flows).
pub fn duration_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 5, 6) {
        return err;
    }
    match duration_calc(args, false) {
        Ok(v) => Value::Number(v),
        Err(e) => e,
    }
}

/// `MDURATION(settlement, maturity, coupon, yld, frequency, [basis])`
///
/// Modified Macaulay duration. MDURATION = DURATION / (1 + yld/frequency)
pub fn mduration_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 5, 6) {
        return err;
    }
    let freq = match to_number(args[4].clone()) { Ok(n) => n, Err(e) => return e };
    let yld  = match to_number(args[3].clone()) { Ok(n) => n, Err(e) => return e };
    match duration_calc(args, false) {
        Ok(d) => {
            let result = d / (1.0 + yld / freq);
            if !result.is_finite() {
                return Value::Error(ErrorKind::Num);
            }
            Value::Number(result)
        }
        Err(e) => e,
    }
}

fn duration_calc(args: &[Value], _modified: bool) -> Result<f64, Value> {
    use crate::eval::functions::date::serial::serial_to_date;
    use crate::eval::functions::financial::bonds::{
        coupon_period_days, days_between, months_per_period, next_coupon_date,
        prev_coupon_date, add_months, validate_basis, validate_frequency,
    };

    let settlement_s = to_number(args[0].clone())?;
    let maturity_s   = to_number(args[1].clone())?;
    let coupon       = to_number(args[2].clone())?;
    let yld          = to_number(args[3].clone())?;
    let freq_f       = to_number(args[4].clone())?;
    let basis_f      = opt_number(args, 5, 0.0)?;

    let frequency = validate_frequency(freq_f)?;
    let basis = validate_basis(basis_f)?;

    let settlement = serial_to_date(settlement_s).ok_or(Value::Error(ErrorKind::Value))?;
    let maturity = serial_to_date(maturity_s).ok_or(Value::Error(ErrorKind::Value))?;

    if settlement >= maturity {
        return Err(Value::Error(ErrorKind::Num));
    }

    let freq = frequency as f64;
    let coupon_per_period = coupon / freq;
    let yld_per_period = yld / freq;

    let pcd = prev_coupon_date(settlement, maturity, frequency);
    let ncd = next_coupon_date(settlement, maturity, frequency);
    let period_days = coupon_period_days(pcd, ncd, frequency, basis);
    let days_to_ncd = days_between(settlement, ncd, basis) as f64;
    let dsc_e = days_to_ncd / period_days;

    // Count coupons
    let mpp = months_per_period(frequency);
    let mut coupon_dates: Vec<f64> = Vec::new();
    let mut t = dsc_e; // fractional periods from settlement to first coupon
    let mut cur = ncd;
    loop {
        coupon_dates.push(t);
        if cur >= maturity {
            break;
        }
        cur = add_months(cur, mpp);
        t += 1.0;
        if t > 1000.0 { break; } // safety
    }

    let n = coupon_dates.len();
    let mut price = 0.0;
    let mut weighted = 0.0;

    for (i, &t_i) in coupon_dates.iter().enumerate() {
        let is_last = i == n - 1;
        let cf = if is_last {
            coupon_per_period + 1.0 // per 1 face value
        } else {
            coupon_per_period
        };
        let pv_cf = cf / (1.0 + yld_per_period).powf(t_i);
        price += pv_cf;
        weighted += t_i * pv_cf;
    }

    if price == 0.0 {
        return Err(Value::Error(ErrorKind::Num));
    }

    // Duration in periods → convert to years
    let duration = (weighted / price) / freq;
    Ok(duration)
}

// ---------------------------------------------------------------------------
// Shared helper: flatten a Value::Array into f64 values
// ---------------------------------------------------------------------------
fn flatten_array(v: Value) -> Result<Vec<f64>, Value> {
    match v {
        Value::Array(items) => {
            let mut out = Vec::new();
            for item in items {
                match item {
                    Value::Array(inner) => {
                        for sub in flatten_array(Value::Array(inner))? {
                            out.push(sub);
                        }
                    }
                    other => out.push(to_number(other)?),
                }
            }
            Ok(out)
        }
        other => Ok(vec![to_number(other)?]),
    }
}

fn flatten_array_dates(v: Value) -> Result<Vec<f64>, Value> {
    // Same as flatten_array but for date serials (Value::Date is also f64)
    match v {
        Value::Array(items) => {
            let mut out = Vec::new();
            for item in items {
                match item {
                    Value::Array(inner) => {
                        for sub in flatten_array_dates(Value::Array(inner))? {
                            out.push(sub);
                        }
                    }
                    Value::Date(n) | Value::Number(n) => out.push(n),
                    other => return Err(to_number(other).unwrap_err()),
                }
            }
            Ok(out)
        }
        Value::Date(n) | Value::Number(n) => Ok(vec![n]),
        other => Err(to_number(other).unwrap_err()),
    }
}

// ---------------------------------------------------------------------------
// FVSCHEDULE
// ---------------------------------------------------------------------------
/// `FVSCHEDULE(principal, schedule)`
///
/// Future value of a principal after applying a schedule of variable interest rates.
/// schedule is an array of rates. FV = principal * prod(1 + rate_i)
pub fn fvschedule_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 2, 2) {
        return err;
    }
    let principal = match to_number(args[0].clone()) { Ok(n) => n, Err(e) => return e };
    let rates = match flatten_array(args[1].clone()) { Ok(r) => r, Err(e) => return e };

    let mut fv = principal;
    for r in rates {
        fv *= 1.0 + r;
    }
    if !fv.is_finite() {
        return Value::Error(ErrorKind::Num);
    }
    Value::Number(fv)
}

// ---------------------------------------------------------------------------
// MIRR
// ---------------------------------------------------------------------------
/// `MIRR(values, finance_rate, reinvest_rate)`
///
/// Modified internal rate of return.
/// MIRR = (FV of positive flows at reinvest_rate / PV of negative flows at finance_rate)^(1/(n-1)) - 1
pub fn mirr_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 3, 3) {
        return err;
    }
    let cfs = match flatten_array(args[0].clone()) { Ok(v) => v, Err(e) => return e };
    let finance_rate = match to_number(args[1].clone()) { Ok(n) => n, Err(e) => return e };
    let reinvest_rate = match to_number(args[2].clone()) { Ok(n) => n, Err(e) => return e };

    let n = cfs.len();
    if n < 2 {
        return Value::Error(ErrorKind::Num);
    }

    // NPV of negative flows discounted at finance_rate (periods 0..n-1)
    let mut npv_neg = 0.0_f64;
    for (i, &cf) in cfs.iter().enumerate() {
        if cf < 0.0 {
            npv_neg += cf / (1.0 + finance_rate).powi(i as i32);
        }
    }
    if npv_neg == 0.0 {
        return Value::Error(ErrorKind::DivByZero);
    }

    // FV of positive flows compounded at reinvest_rate to end of period
    let mut fv_pos = 0.0_f64;
    let last = (n - 1) as i32;
    for (i, &cf) in cfs.iter().enumerate() {
        if cf > 0.0 {
            fv_pos += cf * (1.0 + reinvest_rate).powi(last - i as i32);
        }
    }
    if fv_pos == 0.0 {
        return Value::Error(ErrorKind::Num);
    }

    let result = (-fv_pos / npv_neg).powf(1.0 / (n as f64 - 1.0)) - 1.0;
    if !result.is_finite() {
        return Value::Error(ErrorKind::Num);
    }
    Value::Number(result)
}

// ---------------------------------------------------------------------------
// XNPV
// ---------------------------------------------------------------------------
/// `XNPV(rate, values, dates)`
///
/// Net present value for irregular cash flows with explicit dates.
/// Uses day-count as actual days / 365.
pub fn xnpv_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 3, 3) {
        return err;
    }
    let rate = match to_number(args[0].clone()) { Ok(n) => n, Err(e) => return e };
    let values = match flatten_array(args[1].clone()) { Ok(v) => v, Err(e) => return e };
    let date_serials = match flatten_array_dates(args[2].clone()) { Ok(v) => v, Err(e) => return e };

    if values.len() != date_serials.len() || values.is_empty() {
        return Value::Error(ErrorKind::Num);
    }

    let d0 = date_serials[0];
    let mut npv = 0.0_f64;
    for (i, (&cf, &ds)) in values.iter().zip(date_serials.iter()).enumerate() {
        let _ = i;
        let t = (ds - d0) / 365.0;
        if t < 0.0 {
            return Value::Error(ErrorKind::Num);
        }
        let denom = (1.0 + rate).powf(t);
        if !denom.is_finite() || denom == 0.0 {
            return Value::Error(ErrorKind::Num);
        }
        npv += cf / denom;
    }
    if !npv.is_finite() {
        return Value::Error(ErrorKind::Num);
    }
    Value::Number(npv)
}

// ---------------------------------------------------------------------------
// XIRR
// ---------------------------------------------------------------------------
/// `XIRR(values, dates, [guess])`
///
/// Internal rate of return for irregular cash flows with explicit dates.
/// Uses Newton's method.
pub fn xirr_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 2, 3) {
        return err;
    }
    let values = match flatten_array(args[0].clone()) { Ok(v) => v, Err(e) => return e };
    let date_serials = match flatten_array_dates(args[1].clone()) { Ok(v) => v, Err(e) => return e };
    let guess = if args.len() > 2 {
        match to_number(args[2].clone()) { Ok(n) => n, Err(e) => return e }
    } else {
        0.1
    };

    if values.len() != date_serials.len() || values.len() < 2 {
        return Value::Error(ErrorKind::Num);
    }

    let has_positive = values.iter().any(|&n| n > 0.0);
    let has_negative = values.iter().any(|&n| n < 0.0);
    if !has_positive || !has_negative {
        return Value::Error(ErrorKind::Num);
    }

    let d0 = date_serials[0];
    let times: Vec<f64> = date_serials.iter().map(|&ds| (ds - d0) / 365.0).collect();

    let xnpv_at = |r: f64| -> f64 {
        values.iter().zip(times.iter()).map(|(&cf, &t)| cf / (1.0 + r).powf(t)).sum()
    };
    let dxnpv_at = |r: f64| -> f64 {
        values.iter().zip(times.iter())
            .map(|(&cf, &t)| -t * cf / (1.0 + r).powf(t + 1.0))
            .sum()
    };

    let mut rate = guess;
    for _ in 0..100 {
        let f = xnpv_at(rate);
        let df = dxnpv_at(rate);
        if !f.is_finite() || !df.is_finite() || df == 0.0 {
            return Value::Error(ErrorKind::Num);
        }
        let new_rate = rate - f / df;
        if (new_rate - rate).abs() < 1e-7 {
            if !new_rate.is_finite() {
                return Value::Error(ErrorKind::Num);
            }
            return Value::Number(new_rate);
        }
        rate = new_rate;
    }
    Value::Error(ErrorKind::Num)
}

#[cfg(test)]
mod tests;
