use crate::eval::coercion::to_number;
use crate::eval::functions::check_arity;
use crate::types::{ErrorKind, Value};

/// `IRR(values, [guess])` — internal rate of return.
///
/// `values` may be passed as an array literal `{-100,30,40,50}` (a single
/// `Value::Array` arg) or as individual numeric args.  An optional numeric
/// guess arg is accepted immediately after the values.
///
/// Google Sheets ignores boolean values in cash flow arrays (same as Excel).
pub fn irr_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 1, 256) {
        return err;
    }

    let (cfs, guess) = match collect_cashflows_with_guess(args) {
        Ok(pair) => pair,
        Err(e) => return e,
    };

    if cfs.len() < 2 {
        return Value::Error(ErrorKind::NA);
    }

    // Must have at least one positive and one negative cash flow
    let has_positive = cfs.iter().any(|&n| n > 0.0);
    let has_negative = cfs.iter().any(|&n| n < 0.0);
    if !has_positive || !has_negative {
        return Value::Error(ErrorKind::Num);
    }

    // Try Newton-Raphson with the given guess
    if let Some(rate) = irr_newton(&cfs, guess) {
        return Value::Number(rate);
    }

    // Fallback: Brent's method — searches for a sign-change bracket and
    // converges robustly when NR diverges (e.g. guess far from solution).
    match irr_brent(&cfs) {
        Some(rate) => Value::Number(rate),
        None => Value::Error(ErrorKind::Num),
    }
}

fn irr_newton(cfs: &[f64], guess: f64) -> Option<f64> {
    let mut rate = guess;
    for _ in 0..100 {
        let (npv, dnpv) = npv_and_derivative(cfs, rate);
        if !npv.is_finite() || !dnpv.is_finite() || dnpv == 0.0 {
            return None;
        }
        let new_rate = rate - npv / dnpv;
        if new_rate <= -1.0 || !new_rate.is_finite() {
            return None; // diverged outside valid range
        }
        if (new_rate - rate).abs() < 1e-7 {
            return Some(new_rate);
        }
        rate = new_rate;
    }
    None
}

fn irr_brent(cfs: &[f64]) -> Option<f64> {
    let npv = |r: f64| -> f64 {
        cfs.iter()
            .enumerate()
            .fold(0.0, |acc, (t, &cf)| acc + cf / libm::pow(1.0 + r, t as f64))
    };

    // Scan for a sign change across a wide range that covers negative and
    // positive IRR values. Steps are denser near zero where most IRRs live.
    let candidates: &[f64] = &[
        -0.999, -0.99, -0.9, -0.8, -0.7, -0.6, -0.5, -0.4, -0.3, -0.2,
        -0.15, -0.1, -0.05, 0.0, 0.05, 0.1, 0.15, 0.2, 0.3, 0.5, 1.0,
        2.0, 5.0, 10.0, 50.0, 100.0,
    ];

    let mut prev_r = candidates[0];
    let mut prev_f = npv(prev_r);

    for &r in &candidates[1..] {
        let f_r = npv(r);
        if prev_f * f_r <= 0.0 {
            return brent_root(npv, prev_r, r, 1e-10);
        }
        prev_r = r;
        prev_f = f_r;
    }

    None
}

/// Brent's root-finding method within a bracket [a, b] where f(a)*f(b) ≤ 0.
fn brent_root<F: Fn(f64) -> f64>(f: F, mut a: f64, mut b: f64, tol: f64) -> Option<f64> {
    let mut fa = f(a);
    let mut fb = f(b);

    if !fa.is_finite() || !fb.is_finite() {
        return None;
    }

    // Ensure |f(b)| ≤ |f(a)|
    if fa.abs() < fb.abs() {
        std::mem::swap(&mut a, &mut b);
        std::mem::swap(&mut fa, &mut fb);
    }

    let mut c = a;
    let mut fc = fa;
    let mut mflag = true;
    let mut d = 0.0_f64;

    for _ in 0..200 {
        if fb.abs() < tol || (b - a).abs() < tol {
            return Some(b);
        }

        let s = if fa != fc && fb != fc {
            // Inverse quadratic interpolation
            a * fb * fc / ((fa - fb) * (fa - fc))
                + b * fa * fc / ((fb - fa) * (fb - fc))
                + c * fa * fb / ((fc - fa) * (fc - fb))
        } else {
            // Secant
            b - fb * (b - a) / (fb - fa)
        };

        // Brent's conditions: fall back to bisection if interpolation is poor
        let mid = (a + b) / 2.0;
        let use_bisect = !(((3.0 * a + b) / 4.0 < s && s < b)
            || (b < s && s < (3.0 * a + b) / 4.0))
            || (mflag && (s - b).abs() >= (b - c).abs() / 2.0)
            || (!mflag && (s - b).abs() >= (c - d).abs() / 2.0)
            || (mflag && (b - c).abs() < tol)
            || (!mflag && (c - d).abs() < tol);

        let s = if use_bisect { mid } else { s };
        mflag = use_bisect;

        let fs = f(s);
        d = c;
        c = b;
        fc = fb;

        if fa * fs < 0.0 {
            b = s;
            fb = fs;
        } else {
            a = s;
            fa = fs;
        }

        if fa.abs() < fb.abs() {
            std::mem::swap(&mut a, &mut b);
            std::mem::swap(&mut fa, &mut fb);
        }
    }

    Some(b)
}

/// Flatten args into cash flows, detecting an optional trailing guess.
///
/// Patterns:
///   IRR({-100,30,40,50})               → cfs from array, guess=0.1
///   IRR({-100,30,40,50}, 0.15)         → cfs from array, guess=0.15
///   IRR(-100, 30, 40, 50)              → all args as cfs, guess=0.1
///
/// Boolean values are ignored (Google Sheets / Excel behaviour).
fn collect_cashflows_with_guess(args: &[Value]) -> Result<(Vec<f64>, f64), Value> {
    // Case 1: first arg is an array
    if let Value::Array(items) = &args[0] {
        let cfs = flatten_values(items.clone())?;
        let guess = if args.len() > 1 {
            to_number(args[1].clone())?
        } else {
            0.1
        };
        return Ok((cfs, guess));
    }

    // Case 2: all args are scalars — treat them all as cash flows (skip booleans only;
    // text as a scalar arg causes #VALUE! in GS, which to_number propagates naturally)
    let mut cfs = Vec::with_capacity(args.len());
    for arg in args {
        match arg {
            Value::Bool(_) => {} // ignored per Excel/GS semantics
            _ => cfs.push(to_number(arg.clone())?),
        }
    }
    Ok((cfs, 0.1))
}

/// Flatten a `Vec<Value>` (may be nested arrays) into f64 cash flows.
/// Booleans and text strings are skipped — Excel/GS ignore non-numeric
/// types in IRR arrays rather than coercing them.
fn flatten_values(items: Vec<Value>) -> Result<Vec<f64>, Value> {
    let mut out = Vec::new();
    for v in items {
        match v {
            Value::Array(inner) => {
                let sub = flatten_values(inner)?;
                out.extend(sub);
            }
            Value::Bool(_) | Value::Text(_) => {} // ignored per Excel/GS semantics
            other => out.push(to_number(other)?),
        }
    }
    Ok(out)
}

fn npv_and_derivative(cfs: &[f64], rate: f64) -> (f64, f64) {
    let mut npv = 0.0;
    let mut dnpv = 0.0;
    for (i, &cf) in cfs.iter().enumerate() {
        let t = i as f64;
        let denom = libm::pow(1.0 + rate, t);
        npv  += cf / denom;
        dnpv -= t * cf / libm::pow(1.0 + rate, t + 1.0);
    }
    (npv, dnpv)
}

#[cfg(test)]
mod tests;
