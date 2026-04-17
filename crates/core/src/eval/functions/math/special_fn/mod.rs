use crate::eval::coercion::to_number;
use crate::eval::functions::check_arity;
use crate::types::{ErrorKind, Value};

// ── ERF / ERF.PRECISE ────────────────────────────────────────────────────────

/// Error function approximation using Abramowitz & Stegun (7.1.26).
/// Maximum error: 1.5e-7.
fn erf(x: f64) -> f64 {
    if x < 0.0 {
        return -erf(-x);
    }
    // Use complementary erfc for large x to avoid cancellation
    if x > 6.0 {
        return 1.0;
    }
    // Horner-form rational approximation (A&S 7.1.26, p=0.3275911)
    let t = 1.0 / (1.0 + 0.3275911 * x);
    let poly = t * (0.254_829_592
        + t * (-0.284_496_736
            + t * (1.421_413_741
                + t * (-1.453_152_027
                    + t * 1.061_405_429))));
    1.0 - poly * (-x * x).exp()
}

/// Complementary error function: erfc(x) = 1 - erf(x).
fn erfc(x: f64) -> f64 {
    1.0 - erf(x)
}

/// `ERF(lower_limit, [upper_limit])` — error function.
/// With one arg: ERF(x) = erf(x). With two: ERF(a,b) = erf(b) - erf(a).
pub fn erf_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 1, 2) {
        return err;
    }
    let x = match to_number(args[0].clone()) {
        Err(e) => return e,
        Ok(v) => v,
    };
    if args.len() == 1 {
        Value::Number(erf(x))
    } else {
        let y = match to_number(args[1].clone()) {
            Err(e) => return e,
            Ok(v) => v,
        };
        Value::Number(erf(y) - erf(x))
    }
}

/// `ERF.PRECISE(x)` — same as ERF with one argument (no two-arg form).
pub fn erf_precise_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 1, 1) {
        return err;
    }
    let x = match to_number(args[0].clone()) {
        Err(e) => return e,
        Ok(v) => v,
    };
    Value::Number(erf(x))
}

/// `ERFC(x)` — complementary error function = 1 - ERF(x).
pub fn erfc_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 1, 1) {
        return err;
    }
    let x = match to_number(args[0].clone()) {
        Err(e) => return e,
        Ok(v) => v,
    };
    Value::Number(erfc(x))
}

/// `ERFC.PRECISE(x)` — same as ERFC.
pub fn erfc_precise_fn(args: &[Value]) -> Value {
    erfc_fn(args)
}

// ── GAMMALN / GAMMALN.PRECISE ────────────────────────────────────────────────

/// Natural logarithm of the gamma function using Lanczos approximation.
/// Valid for x > 0.
fn gammaln(x: f64) -> f64 {
    // Lanczos approximation with g=7, n=9 (Numerical Recipes, 2nd ed.)
    const G: f64 = 7.0;
    const C: [f64; 9] = [
        0.999_999_999_999_809_3,
        676.520_368_121_885_1,
        -1_259.139_216_722_402_9,
        771.323_428_777_653_1,
        -176.615_029_162_140_6,
        12.507_343_278_686_905,
        -0.138_571_095_265_720_12,
        9.984_369_578_019_572e-6,
        1.505_632_735_149_311_6e-7,
    ];

    let x = x - 1.0;
    let t = x + G + 0.5;
    let mut ser = C[0];
    let mut xp = x;
    for c in &C[1..] {
        xp += 1.0;
        ser += c / xp;
    }
    use std::f64::consts::PI;
    (2.0 * PI).sqrt().ln() + ser.ln() + (x + 0.5) * t.ln() - t
}

/// `GAMMALN(x)` — natural log of the gamma function.
/// Returns #NUM! for x <= 0 or negative integers.
pub fn gammaln_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 1, 1) {
        return err;
    }
    let x = match to_number(args[0].clone()) {
        Err(e) => return e,
        Ok(v) => v,
    };
    if x <= 0.0 {
        return Value::Error(ErrorKind::Num);
    }
    let result = gammaln(x);
    if !result.is_finite() {
        return Value::Error(ErrorKind::Num);
    }
    Value::Number(result)
}

/// `GAMMALN.PRECISE(x)` — same as GAMMALN.
pub fn gammaln_precise_fn(args: &[Value]) -> Value {
    gammaln_fn(args)
}

#[cfg(test)]
mod tests;
