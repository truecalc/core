use crate::eval::coercion::to_number;
use crate::eval::functions::check_arity;
use crate::types::{ErrorKind, Value};

// ── ERF / ERF.PRECISE ────────────────────────────────────────────────────────

/// Error function approximation using Abramowitz & Stegun (7.1.26).
/// Maximum error: 1.5e-7.
fn erf(x: f64) -> f64 {
    if x >= 0.0 { 1.0 - erfc_precise(x) } else { erfc_precise(-x) - 1.0 }
}

fn erfc_precise(x: f64) -> f64 {
    if x < 0.0 { return 2.0 - erfc_precise(-x); }
    if x == 0.0 { return 1.0; }
    if x > 26.0 { return 0.0; }
    if x <= 0.5 {
        let x2 = x * x;
        let ev = x * std::f64::consts::FRAC_2_SQRT_PI
            * (1.0 - x2*(1.0/3.0 - x2*(1.0/10.0 - x2*(1.0/42.0 - x2*(1.0/216.0 - x2/1320.0)))));
        return 1.0 - ev;
    }
    if x <= 4.0 {
        let (p0,p1,p2,p3): (f64,f64,f64,f64) = (2.4266795523053173e2, 2.1979261618294152e1, 6.9963834886191355, -3.5609843701815385e-2);
        let (q0,q1,q2): (f64,f64,f64) = (2.1505887586986120e2, 9.1164905404325264e1, 1.5082797630407787e1);
        let num = ((p3*x+p2)*x+p1)*x+p0;
        let den = ((x+q2)*x+q1)*x+q0;
        return (-x*x).exp()*num/den;
    }
    let x2=x*x; let t=1.0/x2;
    let s=1.0+t*(-0.5+t*(0.75+t*(-1.875+t*6.5625)));
    (-x2).exp()*s/(x*std::f64::consts::PI.sqrt())
}

fn erfc(x: f64) -> f64 {
    erfc_precise(x)
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
