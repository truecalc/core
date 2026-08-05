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

/// `RATE(nper, pmt, pv, [fv], [type], [guess])` — interest rate per period.
///
/// Uses Newton-Raphson with Brent's method fallback for cases where NR
/// diverges (e.g. high-rate loans where the true rate is far from the guess).
pub fn rate_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 3, 6) {
        return err;
    }
    let nper  = match to_number(args[0].clone()) { Ok(n) => n, Err(e) => return e };
    let pmt   = match to_number(args[1].clone()) { Ok(n) => n, Err(e) => return e };
    let pv    = match to_number(args[2].clone()) { Ok(n) => n, Err(e) => return e };
    let fv    = match opt_number(args, 3, 0.0)   { Ok(n) => n, Err(e) => return e };
    let typ   = match opt_number(args, 4, 0.0)   { Ok(n) => n, Err(e) => return e };
    let guess = match opt_number(args, 5, 0.1)   { Ok(n) => n, Err(e) => return e };

    // Newton-Raphson: find rate such that TVM equation = 0
    // f(r) = pv*(1+r)^nper + pmt*(1+r*type)*((1+r)^nper - 1)/r + fv = 0
    if let Some(rate) = rate_newton(guess, nper, pmt, pv, fv, typ) {
        return Value::Number(rate);
    }

    // Fallback: Brent's method for cases where NR diverges (high-rate scenarios).
    match rate_brent(nper, pmt, pv, fv, typ) {
        Some(rate) => Value::Number(rate),
        None => Value::Error(ErrorKind::Num),
    }
}

fn rate_newton(guess: f64, nper: f64, pmt: f64, pv: f64, fv: f64, typ: f64) -> Option<f64> {
    let mut rate = guess;
    for _ in 0..100 {
        let (f, df) = rate_f_and_df(rate, nper, pmt, pv, fv, typ);
        if !f.is_finite() || !df.is_finite() || df == 0.0 {
            return None;
        }
        let new_rate = rate - f / df;
        if new_rate <= -1.0 || !new_rate.is_finite() {
            return None; // diverged
        }
        if (new_rate - rate).abs() < 1e-7 {
            return Some(new_rate);
        }
        rate = new_rate;
    }
    None
}

fn rate_brent(nper: f64, pmt: f64, pv: f64, fv: f64, typ: f64) -> Option<f64> {
    let f = |r: f64| -> f64 {
        let (val, _) = rate_f_and_df(r, nper, pmt, pv, fv, typ);
        val
    };

    // Scan for a sign change. Rate must be > -1; upper bound 100 covers all
    // realistic financial scenarios.
    let candidates: &[f64] = &[
        -0.999, -0.99, -0.9, -0.8, -0.7, -0.6, -0.5, -0.4, -0.3, -0.2,
        -0.1, -0.05, 0.0, 0.001, 0.01, 0.05, 0.1, 0.2, 0.3, 0.5, 1.0,
        2.0, 5.0, 10.0, 50.0, 100.0,
    ];

    let mut prev_r = candidates[0];
    let mut prev_fv = f(prev_r);

    for &r in &candidates[1..] {
        let f_r = f(r);
        if prev_fv.is_finite() && f_r.is_finite() && prev_fv * f_r <= 0.0 {
            return brent_root(f, prev_r, r, 1e-10);
        }
        prev_r = r;
        prev_fv = f_r;
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
            a * fb * fc / ((fa - fb) * (fa - fc))
                + b * fa * fc / ((fb - fa) * (fb - fc))
                + c * fa * fb / ((fc - fa) * (fc - fb))
        } else {
            b - fb * (b - a) / (fb - fa)
        };

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

fn rate_f_and_df(r: f64, nper: f64, pmt: f64, pv: f64, fv: f64, typ: f64) -> (f64, f64) {
    if r == 0.0 {
        // Limit as r → 0 (L'Hôpital)
        let f = pv + pmt * nper + fv;
        let df = pv * nper + pmt * nper * (nper - 1.0) / 2.0;
        return (f, df);
    }
    let factor = libm::pow(1.0 + r, nper);
    let annuity = pmt * (1.0 + r * typ) * (factor - 1.0) / r;
    let f = pv * factor + annuity + fv;

    let dfactor = nper * libm::pow(1.0 + r, nper - 1.0);
    let dann = pmt * (typ * (factor - 1.0) / r
        + (1.0 + r * typ) * (dfactor * r - (factor - 1.0)) / (r * r));
    let df = pv * dfactor + dann;
    (f, df)
}

#[cfg(test)]
mod tests;
