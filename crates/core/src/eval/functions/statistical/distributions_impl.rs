//! Implementation of all M3 statistical distribution functions.
//!
//! Each function is a standalone `pub fn` returning `Value`.

#![allow(clippy::manual_range_contains)]

use crate::types::{ErrorKind, Value};
use super::stat_helpers::collect_nums;
use super::distributions as d;

// ---------------------------------------------------------------------------
// Helper: extract a bool arg (TRUE/FALSE)
// ---------------------------------------------------------------------------
fn as_bool(v: &Value) -> Option<bool> {
    match v {
        Value::Bool(b) => Some(*b),
        Value::Number(n) => Some(*n != 0.0),
        Value::Text(s) => match s.to_uppercase().as_str() {
            "TRUE" => Some(true),
            "FALSE" => Some(false),
            _ => None,
        },
        _ => None,
    }
}

fn as_f64(v: &Value) -> Option<f64> {
    match v {
        Value::Number(n) => Some(*n),
        Value::Bool(b) => Some(if *b { 1.0 } else { 0.0 }),
        _ => None,
    }
}

// ---------------------------------------------------------------------------
// AVERAGE.WEIGHTED
// ---------------------------------------------------------------------------
pub fn average_weighted_fn(args: &[Value]) -> Value {
    if args.len() < 2 {
        return Value::Error(ErrorKind::NA);
    }
    let values = collect_nums(std::slice::from_ref(&args[0]));
    let weights = collect_nums(std::slice::from_ref(&args[1]));
    if values.is_empty() || values.len() != weights.len() {
        return Value::Error(ErrorKind::NA);
    }
    let total_weight: f64 = weights.iter().sum();
    if total_weight == 0.0 {
        return Value::Error(ErrorKind::DivByZero);
    }
    let weighted_sum: f64 = values.iter().zip(weights.iter()).map(|(v, w)| v * w).sum();
    Value::Number(weighted_sum / total_weight)
}

// ---------------------------------------------------------------------------
// NORM.S.DIST / NORMSDIST
// ---------------------------------------------------------------------------
pub fn norm_s_dist_fn(args: &[Value]) -> Value {
    if args.is_empty() {
        return Value::Error(ErrorKind::NA);
    }
    let x = match as_f64(&args[0]) {
        Some(v) => v,
        None => return Value::Error(ErrorKind::Value),
    };
    // Second arg: cumulative (optional, default TRUE for NORMSDIST compatibility)
    let cumulative = if args.len() >= 2 {
        match as_bool(&args[1]) {
            Some(b) => b,
            None => return Value::Error(ErrorKind::Value),
        }
    } else {
        true // NORMSDIST legacy form has only one arg
    };
    if cumulative {
        Value::Number(d::norm_s_cdf(x))
    } else {
        Value::Number(d::norm_s_pdf(x))
    }
}

// NORMSDIST (legacy, always cumulative)
pub fn normsdist_fn(args: &[Value]) -> Value {
    if args.is_empty() {
        return Value::Error(ErrorKind::NA);
    }
    let x = match as_f64(&args[0]) {
        Some(v) => v,
        None => return Value::Error(ErrorKind::Value),
    };
    Value::Number(d::norm_s_cdf(x))
}

// ---------------------------------------------------------------------------
// NORM.S.INV / NORMSINV
// ---------------------------------------------------------------------------
pub fn norm_s_inv_fn(args: &[Value]) -> Value {
    if args.is_empty() {
        return Value::Error(ErrorKind::NA);
    }
    let p = match as_f64(&args[0]) {
        Some(v) => v,
        None => return Value::Error(ErrorKind::Value),
    };
    if p <= 0.0 || p >= 1.0 {
        return Value::Error(ErrorKind::Num);
    }
    let v = d::norm_s_inv(p);
    if !v.is_finite() {
        Value::Error(ErrorKind::Num)
    } else {
        Value::Number(v)
    }
}

// ---------------------------------------------------------------------------
// NORM.DIST / NORMDIST
// ---------------------------------------------------------------------------
pub fn norm_dist_fn(args: &[Value]) -> Value {
    if args.len() < 4 {
        return Value::Error(ErrorKind::NA);
    }
    let x = match as_f64(&args[0]) { Some(v) => v, None => return Value::Error(ErrorKind::Value) };
    let mean = match as_f64(&args[1]) { Some(v) => v, None => return Value::Error(ErrorKind::Value) };
    let stdev = match as_f64(&args[2]) { Some(v) => v, None => return Value::Error(ErrorKind::Value) };
    if stdev <= 0.0 {
        return Value::Error(ErrorKind::Num);
    }
    let cumulative = match as_bool(&args[3]) {
        Some(b) => b,
        None => return Value::Error(ErrorKind::Value),
    };
    if cumulative {
        Value::Number(d::norm_cdf(x, mean, stdev))
    } else {
        Value::Number(d::norm_pdf(x, mean, stdev))
    }
}

// NORMDIST (legacy, same signature as NORM.DIST)
pub fn normdist_fn(args: &[Value]) -> Value {
    norm_dist_fn(args)
}

// ---------------------------------------------------------------------------
// NORM.INV / NORMINV
// ---------------------------------------------------------------------------
pub fn norm_inv_fn(args: &[Value]) -> Value {
    if args.len() < 3 {
        return Value::Error(ErrorKind::NA);
    }
    let p = match as_f64(&args[0]) { Some(v) => v, None => return Value::Error(ErrorKind::Value) };
    let mean = match as_f64(&args[1]) { Some(v) => v, None => return Value::Error(ErrorKind::Value) };
    let stdev = match as_f64(&args[2]) { Some(v) => v, None => return Value::Error(ErrorKind::Value) };
    if p <= 0.0 || p >= 1.0 || stdev <= 0.0 {
        return Value::Error(ErrorKind::Num);
    }
    let v = d::norm_inv(p, mean, stdev);
    if !v.is_finite() {
        Value::Error(ErrorKind::Num)
    } else {
        Value::Number(v)
    }
}

// ---------------------------------------------------------------------------
// GAUSS
// ---------------------------------------------------------------------------
pub fn gauss_fn(args: &[Value]) -> Value {
    if args.is_empty() {
        return Value::Error(ErrorKind::NA);
    }
    let x = match as_f64(&args[0]) {
        Some(v) => v,
        None => return Value::Error(ErrorKind::Value),
    };
    Value::Number(d::norm_s_cdf(x) - 0.5)
}

// ---------------------------------------------------------------------------
// PHI
// ---------------------------------------------------------------------------
pub fn phi_fn(args: &[Value]) -> Value {
    if args.is_empty() {
        return Value::Error(ErrorKind::NA);
    }
    let x = match as_f64(&args[0]) {
        Some(v) => v,
        None => return Value::Error(ErrorKind::Value),
    };
    Value::Number(d::norm_s_pdf(x))
}

// ---------------------------------------------------------------------------
// STANDARDIZE
// ---------------------------------------------------------------------------
pub fn standardize_fn(args: &[Value]) -> Value {
    if args.len() < 3 {
        return Value::Error(ErrorKind::NA);
    }
    let x = match as_f64(&args[0]) { Some(v) => v, None => return Value::Error(ErrorKind::Value) };
    let mean = match as_f64(&args[1]) { Some(v) => v, None => return Value::Error(ErrorKind::Value) };
    let stdev = match as_f64(&args[2]) { Some(v) => v, None => return Value::Error(ErrorKind::Value) };
    if stdev <= 0.0 {
        return Value::Error(ErrorKind::Num);
    }
    Value::Number((x - mean) / stdev)
}

// ---------------------------------------------------------------------------
// CONFIDENCE / CONFIDENCE.NORM
// ---------------------------------------------------------------------------
pub fn confidence_fn(args: &[Value]) -> Value {
    if args.len() < 3 {
        return Value::Error(ErrorKind::NA);
    }
    let alpha = match as_f64(&args[0]) { Some(v) => v, None => return Value::Error(ErrorKind::Value) };
    let stdev = match as_f64(&args[1]) { Some(v) => v, None => return Value::Error(ErrorKind::Value) };
    let size = match as_f64(&args[2]) { Some(v) => v, None => return Value::Error(ErrorKind::Value) };
    if alpha <= 0.0 || alpha >= 1.0 || stdev <= 0.0 || size < 1.0 {
        return Value::Error(ErrorKind::Num);
    }
    let z = d::norm_s_inv(1.0 - alpha / 2.0);
    Value::Number(z * stdev / size.sqrt())
}

// ---------------------------------------------------------------------------
// CONFIDENCE.T
// ---------------------------------------------------------------------------
pub fn confidence_t_fn(args: &[Value]) -> Value {
    if args.len() < 3 {
        return Value::Error(ErrorKind::NA);
    }
    let alpha = match as_f64(&args[0]) { Some(v) => v, None => return Value::Error(ErrorKind::Value) };
    let stdev = match as_f64(&args[1]) { Some(v) => v, None => return Value::Error(ErrorKind::Value) };
    let size = match as_f64(&args[2]) { Some(v) => v, None => return Value::Error(ErrorKind::Value) };
    if alpha <= 0.0 || alpha >= 1.0 || stdev <= 0.0 || size < 2.0 {
        return Value::Error(ErrorKind::Num);
    }
    let df = size - 1.0;
    let t = d::t_inv(1.0 - alpha / 2.0, df);
    if !t.is_finite() {
        return Value::Error(ErrorKind::Num);
    }
    Value::Number(t * stdev / size.sqrt())
}

// ---------------------------------------------------------------------------
// CORREL / PEARSON
// ---------------------------------------------------------------------------
fn correl_impl(args: &[Value]) -> Value {
    if args.len() < 2 {
        return Value::Error(ErrorKind::NA);
    }
    let xs = collect_nums(std::slice::from_ref(&args[0]));
    let ys = collect_nums(std::slice::from_ref(&args[1]));
    if xs.len() != ys.len() || xs.len() < 2 {
        return Value::Error(ErrorKind::NA);
    }
    match d::pearson_corr(&xs, &ys) {
        Some(r) => {
            if !r.is_finite() {
                Value::Error(ErrorKind::DivByZero)
            } else {
                Value::Number(r)
            }
        }
        None => Value::Error(ErrorKind::DivByZero),
    }
}

pub fn correl_fn(args: &[Value]) -> Value {
    correl_impl(args)
}

pub fn pearson_fn(args: &[Value]) -> Value {
    correl_impl(args)
}

// ---------------------------------------------------------------------------
// Linear regression: SLOPE, INTERCEPT, RSQ, FORECAST, FORECAST.LINEAR, STEYX
// ---------------------------------------------------------------------------
fn get_two_arrays(args: &[Value]) -> Result<(Vec<f64>, Vec<f64>), Value> {
    if args.len() < 2 {
        return Err(Value::Error(ErrorKind::NA));
    }
    let ys = collect_nums(std::slice::from_ref(&args[0]));
    let xs = collect_nums(std::slice::from_ref(&args[1]));
    Ok((xs, ys))
}

pub fn slope_fn(args: &[Value]) -> Value {
    let (xs, ys) = match get_two_arrays(args) {
        Ok(v) => v,
        Err(e) => return e,
    };
    if xs.len() != ys.len() || xs.is_empty() {
        return Value::Error(ErrorKind::NA);
    }
    match d::linear_regression(&xs, &ys) {
        Some((slope, _)) => Value::Number(slope),
        None => Value::Error(ErrorKind::DivByZero),
    }
}

pub fn intercept_fn(args: &[Value]) -> Value {
    let (xs, ys) = match get_two_arrays(args) {
        Ok(v) => v,
        Err(e) => return e,
    };
    if xs.len() != ys.len() || xs.is_empty() {
        return Value::Error(ErrorKind::NA);
    }
    match d::linear_regression(&xs, &ys) {
        Some((_, intercept)) => Value::Number(intercept),
        None => Value::Error(ErrorKind::DivByZero),
    }
}

pub fn rsq_fn(args: &[Value]) -> Value {
    if args.len() < 2 {
        return Value::Error(ErrorKind::NA);
    }
    // RSQ takes (known_y, known_x)
    let ys = collect_nums(std::slice::from_ref(&args[0]));
    let xs = collect_nums(std::slice::from_ref(&args[1]));
    if xs.len() != ys.len() || xs.len() < 2 {
        return Value::Error(ErrorKind::NA);
    }
    match d::pearson_corr(&xs, &ys) {
        Some(r) => {
            if !r.is_finite() {
                Value::Error(ErrorKind::DivByZero)
            } else {
                Value::Number(r * r)
            }
        }
        None => Value::Error(ErrorKind::DivByZero),
    }
}

fn forecast_impl(args: &[Value]) -> Value {
    if args.len() < 3 {
        return Value::Error(ErrorKind::NA);
    }
    let x = match as_f64(&args[0]) {
        Some(v) => v,
        None => return Value::Error(ErrorKind::Value),
    };
    let ys = collect_nums(std::slice::from_ref(&args[1]));
    let xs = collect_nums(std::slice::from_ref(&args[2]));
    if xs.len() != ys.len() || xs.is_empty() {
        return Value::Error(ErrorKind::NA);
    }
    match d::linear_regression(&xs, &ys) {
        Some((slope, intercept)) => Value::Number(slope * x + intercept),
        None => Value::Error(ErrorKind::DivByZero),
    }
}

pub fn forecast_fn(args: &[Value]) -> Value {
    forecast_impl(args)
}

pub fn forecast_linear_fn(args: &[Value]) -> Value {
    forecast_impl(args)
}

pub fn steyx_fn(args: &[Value]) -> Value {
    let (xs, ys) = match get_two_arrays(args) {
        Ok(v) => v,
        Err(e) => return e,
    };
    let n = xs.len();
    if n != ys.len() || n < 3 {
        return Value::Error(ErrorKind::NA);
    }
    let nf = n as f64;
    let mean_x = xs.iter().sum::<f64>() / nf;
    let mean_y = ys.iter().sum::<f64>() / nf;
    let ss_xx: f64 = xs.iter().map(|&x| (x - mean_x).powi(2)).sum();
    let ss_yy: f64 = ys.iter().map(|&y| (y - mean_y).powi(2)).sum();
    let ss_xy: f64 = xs.iter().zip(ys.iter()).map(|(&x, &y)| (x - mean_x) * (y - mean_y)).sum();
    if ss_xx == 0.0 {
        return Value::Error(ErrorKind::DivByZero);
    }
    let se2 = (ss_yy - ss_xy * ss_xy / ss_xx) / (nf - 2.0);
    if se2 < 0.0 {
        return Value::Number(0.0);
    }
    Value::Number(se2.sqrt())
}

// ---------------------------------------------------------------------------
// CHISQ.DIST / CHIDIST
// ---------------------------------------------------------------------------
pub fn chisq_dist_fn(args: &[Value]) -> Value {
    if args.len() < 3 {
        return Value::Error(ErrorKind::NA);
    }
    let x = match as_f64(&args[0]) { Some(v) => v, None => return Value::Error(ErrorKind::Value) };
    let df = match as_f64(&args[1]) { Some(v) => v, None => return Value::Error(ErrorKind::Value) };
    let cumulative = match as_bool(&args[2]) {
        Some(b) => b,
        None => return Value::Error(ErrorKind::Value),
    };
    if x < 0.0 || df < 1.0 {
        return Value::Error(ErrorKind::Num);
    }
    if cumulative {
        Value::Number(d::chisq_cdf(x, df))
    } else {
        Value::Number(d::chisq_pdf(x, df))
    }
}

// CHISQ.DIST.RT (right tail)
pub fn chisq_dist_rt_fn(args: &[Value]) -> Value {
    if args.len() < 2 {
        return Value::Error(ErrorKind::NA);
    }
    let x = match as_f64(&args[0]) { Some(v) => v, None => return Value::Error(ErrorKind::Value) };
    let df = match as_f64(&args[1]) { Some(v) => v, None => return Value::Error(ErrorKind::Value) };
    if x < 0.0 || df < 1.0 {
        return Value::Error(ErrorKind::Num);
    }
    Value::Number(1.0 - d::chisq_cdf(x, df))
}

// CHIDIST (legacy, right-tail)
pub fn chidist_fn(args: &[Value]) -> Value {
    if args.len() < 2 {
        return Value::Error(ErrorKind::NA);
    }
    let x = match as_f64(&args[0]) { Some(v) => v, None => return Value::Error(ErrorKind::Value) };
    let df = match as_f64(&args[1]) { Some(v) => v, None => return Value::Error(ErrorKind::Value) };
    if x < 0.0 || df < 1.0 {
        return Value::Error(ErrorKind::Num);
    }
    Value::Number(1.0 - d::chisq_cdf(x, df))
}

// ---------------------------------------------------------------------------
// CHISQ.INV / CHIINV
// ---------------------------------------------------------------------------
pub fn chisq_inv_fn(args: &[Value]) -> Value {
    if args.len() < 2 {
        return Value::Error(ErrorKind::NA);
    }
    let p = match as_f64(&args[0]) { Some(v) => v, None => return Value::Error(ErrorKind::Value) };
    let df = match as_f64(&args[1]) { Some(v) => v, None => return Value::Error(ErrorKind::Value) };
    if p < 0.0 || p > 1.0 || df < 1.0 {
        return Value::Error(ErrorKind::Num);
    }
    let v = d::chisq_inv(p, df);
    if !v.is_finite() {
        Value::Error(ErrorKind::Num)
    } else {
        Value::Number(v)
    }
}

// CHISQ.INV.RT (right-tail inverse)
pub fn chisq_inv_rt_fn(args: &[Value]) -> Value {
    if args.len() < 2 {
        return Value::Error(ErrorKind::NA);
    }
    let p = match as_f64(&args[0]) { Some(v) => v, None => return Value::Error(ErrorKind::Value) };
    let df = match as_f64(&args[1]) { Some(v) => v, None => return Value::Error(ErrorKind::Value) };
    if p < 0.0 || p > 1.0 || df < 1.0 {
        return Value::Error(ErrorKind::Num);
    }
    let v = d::chisq_inv(1.0 - p, df);
    if !v.is_finite() {
        Value::Error(ErrorKind::Num)
    } else {
        Value::Number(v)
    }
}

// CHIINV (legacy, right-tail)
pub fn chiinv_fn(args: &[Value]) -> Value {
    chisq_inv_rt_fn(args)
}

// ---------------------------------------------------------------------------
// CHISQ.TEST / CHITEST
// ---------------------------------------------------------------------------
fn chisq_test_impl(args: &[Value]) -> Value {
    if args.len() < 2 {
        return Value::Error(ErrorKind::NA);
    }
    let observed = collect_nums(std::slice::from_ref(&args[0]));
    let expected = collect_nums(std::slice::from_ref(&args[1]));
    if observed.len() != expected.len() || observed.is_empty() {
        return Value::Error(ErrorKind::NA);
    }
    // Chi-squared statistic
    let chi2: f64 = observed.iter().zip(expected.iter())
        .map(|(o, e)| {
            if *e == 0.0 { 0.0 } else { (o - e).powi(2) / e }
        })
        .sum();
    let df = (observed.len() - 1) as f64;
    if df <= 0.0 {
        return Value::Error(ErrorKind::DivByZero);
    }
    // Return right-tail p-value
    Value::Number(1.0 - d::chisq_cdf(chi2, df))
}

pub fn chisq_test_fn(args: &[Value]) -> Value {
    chisq_test_impl(args)
}

pub fn chitest_fn(args: &[Value]) -> Value {
    chisq_test_impl(args)
}

// ---------------------------------------------------------------------------
// T.DIST / TDIST
// ---------------------------------------------------------------------------
pub fn t_dist_fn(args: &[Value]) -> Value {
    if args.len() < 3 {
        return Value::Error(ErrorKind::NA);
    }
    let x = match as_f64(&args[0]) { Some(v) => v, None => return Value::Error(ErrorKind::Value) };
    let df = match as_f64(&args[1]) { Some(v) => v, None => return Value::Error(ErrorKind::Value) };
    let cumulative = match as_bool(&args[2]) {
        Some(b) => b,
        None => return Value::Error(ErrorKind::Value),
    };
    if df < 1.0 {
        return Value::Error(ErrorKind::Num);
    }
    if cumulative {
        Value::Number(d::t_cdf(x, df))
    } else {
        Value::Number(d::t_pdf(x, df))
    }
}

// T.DIST.RT (right tail)
pub fn t_dist_rt_fn(args: &[Value]) -> Value {
    if args.len() < 2 {
        return Value::Error(ErrorKind::NA);
    }
    let x = match as_f64(&args[0]) { Some(v) => v, None => return Value::Error(ErrorKind::Value) };
    let df = match as_f64(&args[1]) { Some(v) => v, None => return Value::Error(ErrorKind::Value) };
    if df < 1.0 {
        return Value::Error(ErrorKind::Num);
    }
    Value::Number(1.0 - d::t_cdf(x, df))
}

// T.DIST.2T (two-tailed)
pub fn t_dist_2t_fn(args: &[Value]) -> Value {
    if args.len() < 2 {
        return Value::Error(ErrorKind::NA);
    }
    let x = match as_f64(&args[0]) { Some(v) => v, None => return Value::Error(ErrorKind::Value) };
    let df = match as_f64(&args[1]) { Some(v) => v, None => return Value::Error(ErrorKind::Value) };
    if df < 1.0 || x < 0.0 {
        return Value::Error(ErrorKind::Num);
    }
    Value::Number(2.0 * (1.0 - d::t_cdf(x, df)))
}

// TDIST (legacy: takes x, df, tails)
pub fn tdist_fn(args: &[Value]) -> Value {
    if args.len() < 3 {
        return Value::Error(ErrorKind::NA);
    }
    let x = match as_f64(&args[0]) { Some(v) => v, None => return Value::Error(ErrorKind::Value) };
    let df = match as_f64(&args[1]) { Some(v) => v, None => return Value::Error(ErrorKind::Value) };
    let tails = match as_f64(&args[2]) { Some(v) => v as i64, None => return Value::Error(ErrorKind::Value) };
    if df < 1.0 || x < 0.0 {
        return Value::Error(ErrorKind::Num);
    }
    if tails == 1 {
        Value::Number(1.0 - d::t_cdf(x, df))
    } else if tails == 2 {
        Value::Number(2.0 * (1.0 - d::t_cdf(x, df)))
    } else {
        Value::Error(ErrorKind::Num)
    }
}

// ---------------------------------------------------------------------------
// T.INV / TINV
// ---------------------------------------------------------------------------
pub fn t_inv_fn(args: &[Value]) -> Value {
    if args.len() < 2 {
        return Value::Error(ErrorKind::NA);
    }
    let p = match as_f64(&args[0]) { Some(v) => v, None => return Value::Error(ErrorKind::Value) };
    let df = match as_f64(&args[1]) { Some(v) => v, None => return Value::Error(ErrorKind::Value) };
    if p <= 0.0 || p >= 1.0 || df < 1.0 {
        return Value::Error(ErrorKind::Num);
    }
    let v = d::t_inv(p, df);
    if !v.is_finite() {
        Value::Error(ErrorKind::Num)
    } else {
        Value::Number(v)
    }
}

// T.INV.2T (two-tailed inverse)
pub fn t_inv_2t_fn(args: &[Value]) -> Value {
    if args.len() < 2 {
        return Value::Error(ErrorKind::NA);
    }
    let p = match as_f64(&args[0]) { Some(v) => v, None => return Value::Error(ErrorKind::Value) };
    let df = match as_f64(&args[1]) { Some(v) => v, None => return Value::Error(ErrorKind::Value) };
    if p <= 0.0 || p > 1.0 || df < 1.0 {
        return Value::Error(ErrorKind::Num);
    }
    // Two-tail: find t such that P(|T| > t) = p, i.e., P(T <= t) = 1 - p/2
    let v = d::t_inv(1.0 - p / 2.0, df);
    if !v.is_finite() {
        Value::Error(ErrorKind::Num)
    } else {
        Value::Number(v)
    }
}

// TINV (legacy, two-tailed: same as T.INV.2T)
pub fn tinv_fn(args: &[Value]) -> Value {
    t_inv_2t_fn(args)
}

// ---------------------------------------------------------------------------
// T.TEST / TTEST
// ---------------------------------------------------------------------------
fn t_test_impl(args: &[Value]) -> Value {
    if args.len() < 4 {
        return Value::Error(ErrorKind::NA);
    }
    let arr1 = collect_nums(std::slice::from_ref(&args[0]));
    let arr2 = collect_nums(std::slice::from_ref(&args[1]));
    let tails = match as_f64(&args[2]) { Some(v) => v as i64, None => return Value::Error(ErrorKind::Value) };
    let typ = match as_f64(&args[3]) { Some(v) => v as i64, None => return Value::Error(ErrorKind::Value) };

    if arr1.is_empty() || arr2.is_empty() {
        return Value::Error(ErrorKind::NA);
    }

    let (t_stat, df) = match typ {
        1 => {
            // Paired t-test
            if arr1.len() != arr2.len() {
                return Value::Error(ErrorKind::NA);
            }
            let n = arr1.len() as f64;
            let diffs: Vec<f64> = arr1.iter().zip(arr2.iter()).map(|(a, b)| a - b).collect();
            let mean_d = diffs.iter().sum::<f64>() / n;
            let var_d = diffs.iter().map(|d| (d - mean_d).powi(2)).sum::<f64>() / (n - 1.0);
            if var_d == 0.0 {
                return Value::Error(ErrorKind::DivByZero);
            }
            let t = mean_d / (var_d / n).sqrt();
            (t, n - 1.0)
        }
        2 => {
            // Two-sample equal variance
            let n1 = arr1.len() as f64;
            let n2 = arr2.len() as f64;
            if n1 < 2.0 || n2 < 2.0 {
                return Value::Error(ErrorKind::DivByZero);
            }
            let mean1 = arr1.iter().sum::<f64>() / n1;
            let mean2 = arr2.iter().sum::<f64>() / n2;
            let var1 = arr1.iter().map(|x| (x - mean1).powi(2)).sum::<f64>() / (n1 - 1.0);
            let var2 = arr2.iter().map(|x| (x - mean2).powi(2)).sum::<f64>() / (n2 - 1.0);
            let sp2 = ((n1 - 1.0) * var1 + (n2 - 1.0) * var2) / (n1 + n2 - 2.0);
            if sp2 == 0.0 {
                return Value::Error(ErrorKind::DivByZero);
            }
            let t = (mean1 - mean2) / (sp2 * (1.0 / n1 + 1.0 / n2)).sqrt();
            (t, n1 + n2 - 2.0)
        }
        3 => {
            // Two-sample unequal variance (Welch's)
            let n1 = arr1.len() as f64;
            let n2 = arr2.len() as f64;
            if n1 < 2.0 || n2 < 2.0 {
                return Value::Error(ErrorKind::DivByZero);
            }
            let mean1 = arr1.iter().sum::<f64>() / n1;
            let mean2 = arr2.iter().sum::<f64>() / n2;
            let var1 = arr1.iter().map(|x| (x - mean1).powi(2)).sum::<f64>() / (n1 - 1.0);
            let var2 = arr2.iter().map(|x| (x - mean2).powi(2)).sum::<f64>() / (n2 - 1.0);
            let s1 = var1 / n1;
            let s2 = var2 / n2;
            let denom = s1 + s2;
            if denom == 0.0 {
                return Value::Error(ErrorKind::DivByZero);
            }
            let t = (mean1 - mean2) / denom.sqrt();
            let df_welch = (s1 + s2).powi(2) / (s1.powi(2) / (n1 - 1.0) + s2.powi(2) / (n2 - 1.0));
            (t, df_welch)
        }
        _ => return Value::Error(ErrorKind::Num),
    };

    if !t_stat.is_finite() || !df.is_finite() {
        return Value::Error(ErrorKind::Num);
    }

    let p = if tails == 1 {
        1.0 - d::t_cdf(t_stat.abs(), df)
    } else if tails == 2 {
        2.0 * (1.0 - d::t_cdf(t_stat.abs(), df))
    } else {
        return Value::Error(ErrorKind::Num);
    };
    Value::Number(p)
}

pub fn t_test_fn(args: &[Value]) -> Value {
    t_test_impl(args)
}

pub fn ttest_fn(args: &[Value]) -> Value {
    t_test_impl(args)
}

// ---------------------------------------------------------------------------
// F.DIST / FDIST
// ---------------------------------------------------------------------------
pub fn f_dist_fn(args: &[Value]) -> Value {
    if args.len() < 4 {
        return Value::Error(ErrorKind::NA);
    }
    let x = match as_f64(&args[0]) { Some(v) => v, None => return Value::Error(ErrorKind::Value) };
    let df1 = match as_f64(&args[1]) { Some(v) => v, None => return Value::Error(ErrorKind::Value) };
    let df2 = match as_f64(&args[2]) { Some(v) => v, None => return Value::Error(ErrorKind::Value) };
    let cumulative = match as_bool(&args[3]) {
        Some(b) => b,
        None => return Value::Error(ErrorKind::Value),
    };
    if x < 0.0 || df1 < 1.0 || df2 < 1.0 {
        return Value::Error(ErrorKind::Num);
    }
    if cumulative {
        Value::Number(d::f_cdf(x, df1, df2))
    } else {
        Value::Number(d::f_pdf(x, df1, df2))
    }
}

// F.DIST.RT (right tail)
pub fn f_dist_rt_fn(args: &[Value]) -> Value {
    if args.len() < 3 {
        return Value::Error(ErrorKind::NA);
    }
    let x = match as_f64(&args[0]) { Some(v) => v, None => return Value::Error(ErrorKind::Value) };
    let df1 = match as_f64(&args[1]) { Some(v) => v, None => return Value::Error(ErrorKind::Value) };
    let df2 = match as_f64(&args[2]) { Some(v) => v, None => return Value::Error(ErrorKind::Value) };
    if x < 0.0 || df1 < 1.0 || df2 < 1.0 {
        return Value::Error(ErrorKind::Num);
    }
    Value::Number(1.0 - d::f_cdf(x, df1, df2))
}

// FDIST (legacy, right tail)
pub fn fdist_fn(args: &[Value]) -> Value {
    f_dist_rt_fn(args)
}

// ---------------------------------------------------------------------------
// F.INV / FINV
// ---------------------------------------------------------------------------
pub fn f_inv_fn(args: &[Value]) -> Value {
    if args.len() < 3 {
        return Value::Error(ErrorKind::NA);
    }
    let p = match as_f64(&args[0]) { Some(v) => v, None => return Value::Error(ErrorKind::Value) };
    let df1 = match as_f64(&args[1]) { Some(v) => v, None => return Value::Error(ErrorKind::Value) };
    let df2 = match as_f64(&args[2]) { Some(v) => v, None => return Value::Error(ErrorKind::Value) };
    if p < 0.0 || p > 1.0 || df1 < 1.0 || df2 < 1.0 {
        return Value::Error(ErrorKind::Num);
    }
    let v = d::f_inv(p, df1, df2);
    if !v.is_finite() {
        Value::Error(ErrorKind::Num)
    } else {
        Value::Number(v)
    }
}

// F.INV.RT (right-tail inverse)
pub fn f_inv_rt_fn(args: &[Value]) -> Value {
    if args.len() < 3 {
        return Value::Error(ErrorKind::NA);
    }
    let p = match as_f64(&args[0]) { Some(v) => v, None => return Value::Error(ErrorKind::Value) };
    let df1 = match as_f64(&args[1]) { Some(v) => v, None => return Value::Error(ErrorKind::Value) };
    let df2 = match as_f64(&args[2]) { Some(v) => v, None => return Value::Error(ErrorKind::Value) };
    if p < 0.0 || p > 1.0 || df1 < 1.0 || df2 < 1.0 {
        return Value::Error(ErrorKind::Num);
    }
    let v = d::f_inv(1.0 - p, df1, df2);
    if !v.is_finite() {
        Value::Error(ErrorKind::Num)
    } else {
        Value::Number(v)
    }
}

// FINV (legacy, right-tail)
pub fn finv_fn(args: &[Value]) -> Value {
    f_inv_rt_fn(args)
}

// ---------------------------------------------------------------------------
// F.TEST / FTEST
// ---------------------------------------------------------------------------
fn f_test_impl(args: &[Value]) -> Value {
    if args.len() < 2 {
        return Value::Error(ErrorKind::NA);
    }
    let arr1 = collect_nums(std::slice::from_ref(&args[0]));
    let arr2 = collect_nums(std::slice::from_ref(&args[1]));
    let n1 = arr1.len() as f64;
    let n2 = arr2.len() as f64;
    if n1 < 2.0 || n2 < 2.0 {
        return Value::Error(ErrorKind::DivByZero);
    }
    let mean1 = arr1.iter().sum::<f64>() / n1;
    let mean2 = arr2.iter().sum::<f64>() / n2;
    let var1 = arr1.iter().map(|x| (x - mean1).powi(2)).sum::<f64>() / (n1 - 1.0);
    let var2 = arr2.iter().map(|x| (x - mean2).powi(2)).sum::<f64>() / (n2 - 1.0);
    if var2 == 0.0 {
        return Value::Error(ErrorKind::DivByZero);
    }
    let f = var1 / var2;
    let df1 = n1 - 1.0;
    let df2 = n2 - 1.0;
    // Two-tailed F-test p-value
    let p1 = d::f_cdf(f, df1, df2);
    let p = 2.0 * p1.min(1.0 - p1);
    Value::Number(p)
}

pub fn f_test_fn(args: &[Value]) -> Value {
    f_test_impl(args)
}

pub fn ftest_fn(args: &[Value]) -> Value {
    f_test_impl(args)
}

// ---------------------------------------------------------------------------
// GAMMA function
// ---------------------------------------------------------------------------
pub fn gamma_fn_impl(args: &[Value]) -> Value {
    if args.is_empty() {
        return Value::Error(ErrorKind::NA);
    }
    let x = match as_f64(&args[0]) { Some(v) => v, None => return Value::Error(ErrorKind::Value) };
    if x <= 0.0 && (x == x.floor()) {
        return Value::Error(ErrorKind::Num);
    }
    if x == 0.0 {
        return Value::Error(ErrorKind::Num);
    }
    let v = d::gamma_fn(x);
    if !v.is_finite() {
        Value::Error(ErrorKind::Num)
    } else {
        Value::Number(v)
    }
}

// ---------------------------------------------------------------------------
// GAMMA.DIST / GAMMADIST
// ---------------------------------------------------------------------------
pub fn gamma_dist_fn(args: &[Value]) -> Value {
    if args.len() < 4 {
        return Value::Error(ErrorKind::NA);
    }
    let x = match as_f64(&args[0]) { Some(v) => v, None => return Value::Error(ErrorKind::Value) };
    let alpha = match as_f64(&args[1]) { Some(v) => v, None => return Value::Error(ErrorKind::Value) };
    let beta = match as_f64(&args[2]) { Some(v) => v, None => return Value::Error(ErrorKind::Value) };
    let cumulative = match as_bool(&args[3]) {
        Some(b) => b,
        None => return Value::Error(ErrorKind::Value),
    };
    if x < 0.0 || alpha <= 0.0 || beta <= 0.0 {
        return Value::Error(ErrorKind::Num);
    }
    if cumulative {
        Value::Number(d::gamma_dist_cdf(x, alpha, beta))
    } else {
        Value::Number(d::gamma_dist_pdf(x, alpha, beta))
    }
}

// ---------------------------------------------------------------------------
// GAMMA.INV / GAMMAINV
// ---------------------------------------------------------------------------
pub fn gamma_inv_fn(args: &[Value]) -> Value {
    if args.len() < 3 {
        return Value::Error(ErrorKind::NA);
    }
    let p = match as_f64(&args[0]) { Some(v) => v, None => return Value::Error(ErrorKind::Value) };
    let alpha = match as_f64(&args[1]) { Some(v) => v, None => return Value::Error(ErrorKind::Value) };
    let beta = match as_f64(&args[2]) { Some(v) => v, None => return Value::Error(ErrorKind::Value) };
    if p < 0.0 || p >= 1.0 || alpha <= 0.0 || beta <= 0.0 {
        return Value::Error(ErrorKind::Num);
    }
    let v = d::gamma_dist_inv(p, alpha, beta);
    if !v.is_finite() {
        Value::Error(ErrorKind::Num)
    } else {
        Value::Number(v)
    }
}

// ---------------------------------------------------------------------------
// BETA.DIST / BETADIST
// ---------------------------------------------------------------------------
pub fn beta_dist_fn(args: &[Value]) -> Value {
    if args.len() < 4 {
        return Value::Error(ErrorKind::NA);
    }
    let x = match as_f64(&args[0]) { Some(v) => v, None => return Value::Error(ErrorKind::Value) };
    let alpha = match as_f64(&args[1]) { Some(v) => v, None => return Value::Error(ErrorKind::Value) };
    let beta = match as_f64(&args[2]) { Some(v) => v, None => return Value::Error(ErrorKind::Value) };
    let cumulative = match as_bool(&args[3]) {
        Some(b) => b,
        None => return Value::Error(ErrorKind::Value),
    };
    // Optional lo/hi bounds (args[4] and args[5])
    let lo = if args.len() >= 5 {
        match as_f64(&args[4]) { Some(v) => v, None => return Value::Error(ErrorKind::Value) }
    } else { 0.0 };
    let hi = if args.len() >= 6 {
        match as_f64(&args[5]) { Some(v) => v, None => return Value::Error(ErrorKind::Value) }
    } else { 1.0 };
    if alpha <= 0.0 || beta <= 0.0 || lo >= hi {
        return Value::Error(ErrorKind::Num);
    }
    if x < lo || x > hi {
        return Value::Error(ErrorKind::Num);
    }
    if cumulative {
        Value::Number(d::beta_dist_cdf(x, alpha, beta, lo, hi))
    } else {
        Value::Number(d::beta_dist_pdf(x, alpha, beta, lo, hi))
    }
}

// BETADIST (legacy, always CDF with lo/hi)
pub fn betadist_fn(args: &[Value]) -> Value {
    if args.len() < 5 {
        return Value::Error(ErrorKind::NA);
    }
    let x = match as_f64(&args[0]) { Some(v) => v, None => return Value::Error(ErrorKind::Value) };
    let alpha = match as_f64(&args[1]) { Some(v) => v, None => return Value::Error(ErrorKind::Value) };
    let beta = match as_f64(&args[2]) { Some(v) => v, None => return Value::Error(ErrorKind::Value) };
    let lo = match as_f64(&args[3]) { Some(v) => v, None => return Value::Error(ErrorKind::Value) };
    let hi = match as_f64(&args[4]) { Some(v) => v, None => return Value::Error(ErrorKind::Value) };
    if alpha <= 0.0 || beta <= 0.0 || lo >= hi {
        return Value::Error(ErrorKind::Num);
    }
    if x < lo || x > hi {
        return Value::Error(ErrorKind::Num);
    }
    Value::Number(d::beta_dist_cdf(x, alpha, beta, lo, hi))
}

// ---------------------------------------------------------------------------
// BETA.INV / BETAINV
// ---------------------------------------------------------------------------
pub fn beta_inv_fn(args: &[Value]) -> Value {
    if args.len() < 5 {
        return Value::Error(ErrorKind::NA);
    }
    let p = match as_f64(&args[0]) { Some(v) => v, None => return Value::Error(ErrorKind::Value) };
    let alpha = match as_f64(&args[1]) { Some(v) => v, None => return Value::Error(ErrorKind::Value) };
    let beta = match as_f64(&args[2]) { Some(v) => v, None => return Value::Error(ErrorKind::Value) };
    let lo = match as_f64(&args[3]) { Some(v) => v, None => return Value::Error(ErrorKind::Value) };
    let hi = match as_f64(&args[4]) { Some(v) => v, None => return Value::Error(ErrorKind::Value) };
    if p < 0.0 || p > 1.0 || alpha <= 0.0 || beta <= 0.0 || lo >= hi {
        return Value::Error(ErrorKind::Num);
    }
    if p == 0.0 {
        return Value::Error(ErrorKind::Num);
    }
    let v = d::beta_dist_inv(p, alpha, beta, lo, hi);
    if !v.is_finite() {
        Value::Error(ErrorKind::Num)
    } else {
        Value::Number(v)
    }
}

// ---------------------------------------------------------------------------
// BINOM.DIST / BINOMDIST
// ---------------------------------------------------------------------------
pub fn binom_dist_fn(args: &[Value]) -> Value {
    if args.len() < 4 {
        return Value::Error(ErrorKind::NA);
    }
    let k_f = match as_f64(&args[0]) { Some(v) => v, None => return Value::Error(ErrorKind::Value) };
    let n_f = match as_f64(&args[1]) { Some(v) => v, None => return Value::Error(ErrorKind::Value) };
    let p = match as_f64(&args[2]) { Some(v) => v, None => return Value::Error(ErrorKind::Value) };
    let cumulative = match as_bool(&args[3]) {
        Some(b) => b,
        None => return Value::Error(ErrorKind::Value),
    };
    if k_f < 0.0 || n_f < 0.0 || p < 0.0 || p > 1.0 || k_f > n_f {
        return Value::Error(ErrorKind::Num);
    }
    let k = k_f as u64;
    let n = n_f as u64;
    if cumulative {
        Value::Number(d::binom_cdf(k, n, p))
    } else {
        // Handle edge cases: p=0 means P(X=k)=1 iff k=0, p=1 means P(X=k)=1 iff k=n
        let pmf = if p == 0.0 {
            if k == 0 { 1.0 } else { 0.0 }
        } else if p == 1.0 {
            if k == n { 1.0 } else { 0.0 }
        } else {
            (d::binom_coeff_ln_pub(n, k)
                + (k as f64) * p.ln()
                + ((n - k) as f64) * (1.0 - p).ln())
            .exp()
        };
        Value::Number(pmf)
    }
}

// ---------------------------------------------------------------------------
// BINOM.INV / CRITBINOM
// ---------------------------------------------------------------------------
pub fn binom_inv_fn(args: &[Value]) -> Value {
    if args.len() < 3 {
        return Value::Error(ErrorKind::NA);
    }
    let n_f = match as_f64(&args[0]) { Some(v) => v, None => return Value::Error(ErrorKind::Value) };
    let p = match as_f64(&args[1]) { Some(v) => v, None => return Value::Error(ErrorKind::Value) };
    let alpha = match as_f64(&args[2]) { Some(v) => v, None => return Value::Error(ErrorKind::Value) };
    if n_f < 0.0 || p < 0.0 || p > 1.0 || alpha <= 0.0 || alpha >= 1.0 {
        return Value::Error(ErrorKind::Num);
    }
    let n = n_f as u64;
    Value::Number(d::binom_inv(n, p, alpha) as f64)
}

// ---------------------------------------------------------------------------
// POISSON / POISSON.DIST
// ---------------------------------------------------------------------------
fn poisson_impl(args: &[Value]) -> Value {
    if args.len() < 3 {
        return Value::Error(ErrorKind::NA);
    }
    let x_f = match as_f64(&args[0]) { Some(v) => v, None => return Value::Error(ErrorKind::Value) };
    let lambda = match as_f64(&args[1]) { Some(v) => v, None => return Value::Error(ErrorKind::Value) };
    let cumulative = match as_bool(&args[2]) {
        Some(b) => b,
        None => return Value::Error(ErrorKind::Value),
    };
    if x_f < 0.0 || lambda < 0.0 {
        return Value::Error(ErrorKind::Num);
    }
    let x = x_f as u64;
    if cumulative {
        Value::Number(d::poisson_cdf(x, lambda))
    } else {
        Value::Number(d::poisson_pmf(x, lambda))
    }
}

pub fn poisson_fn(args: &[Value]) -> Value {
    poisson_impl(args)
}

pub fn poisson_dist_fn(args: &[Value]) -> Value {
    poisson_impl(args)
}

// ---------------------------------------------------------------------------
// NEGBINOM.DIST / NEGBINOMDIST
// ---------------------------------------------------------------------------
fn negbinom_impl(args: &[Value]) -> Value {
    if args.len() < 4 {
        return Value::Error(ErrorKind::NA);
    }
    let x_f = match as_f64(&args[0]) { Some(v) => v, None => return Value::Error(ErrorKind::Value) };
    let r_f = match as_f64(&args[1]) { Some(v) => v, None => return Value::Error(ErrorKind::Value) };
    let p = match as_f64(&args[2]) { Some(v) => v, None => return Value::Error(ErrorKind::Value) };
    let cumulative = match as_bool(&args[3]) {
        Some(b) => b,
        None => return Value::Error(ErrorKind::Value),
    };
    if x_f < 0.0 || r_f < 1.0 || p < 0.0 || p > 1.0 {
        return Value::Error(ErrorKind::Num);
    }
    let x = x_f as u64;
    let r = r_f as u64;
    if cumulative {
        Value::Number(d::negbinom_cdf(x, r, p))
    } else {
        Value::Number(d::negbinom_pmf(x, r, p))
    }
}

pub fn negbinom_dist_fn(args: &[Value]) -> Value {
    negbinom_impl(args)
}

pub fn negbinomdist_fn(args: &[Value]) -> Value {
    // Legacy: 3 args, no cumulative (always PMF)
    if args.len() < 3 {
        return Value::Error(ErrorKind::NA);
    }
    let x_f = match as_f64(&args[0]) { Some(v) => v, None => return Value::Error(ErrorKind::Value) };
    let r_f = match as_f64(&args[1]) { Some(v) => v, None => return Value::Error(ErrorKind::Value) };
    let p = match as_f64(&args[2]) { Some(v) => v, None => return Value::Error(ErrorKind::Value) };
    if x_f < 0.0 || r_f < 1.0 || p < 0.0 || p > 1.0 {
        return Value::Error(ErrorKind::Num);
    }
    let x = x_f as u64;
    let r = r_f as u64;
    Value::Number(d::negbinom_pmf(x, r, p))
}

// ---------------------------------------------------------------------------
// HYPGEOM.DIST / HYPGEOMDIST
// ---------------------------------------------------------------------------
fn hypgeom_impl(args: &[Value], has_cumulative: bool) -> Value {
    let min_args = if has_cumulative { 5 } else { 4 };
    if args.len() < min_args {
        return Value::Error(ErrorKind::NA);
    }
    let x_f = match as_f64(&args[0]) { Some(v) => v, None => return Value::Error(ErrorKind::Value) };
    let n_f = match as_f64(&args[1]) { Some(v) => v, None => return Value::Error(ErrorKind::Value) };
    let k_f = match as_f64(&args[2]) { Some(v) => v, None => return Value::Error(ErrorKind::Value) };
    let pop_f = match as_f64(&args[3]) { Some(v) => v, None => return Value::Error(ErrorKind::Value) };
    let cumulative = if has_cumulative {
        match as_bool(&args[4]) {
            Some(b) => b,
            None => return Value::Error(ErrorKind::Value),
        }
    } else {
        false
    };
    if x_f < 0.0 || n_f < 0.0 || k_f < 0.0 || pop_f < 0.0 {
        return Value::Error(ErrorKind::Num);
    }
    let x = x_f as u64;
    let n = n_f as u64;
    let k = k_f as u64;
    let pop = pop_f as u64;
    if x > n || x > k || n > pop || k > pop {
        return Value::Error(ErrorKind::Num);
    }
    if cumulative {
        Value::Number(d::hypgeom_cdf(x, pop, k, n))
    } else {
        Value::Number(d::hypgeom_pmf(x, pop, k, n))
    }
}

pub fn hypgeom_dist_fn(args: &[Value]) -> Value {
    hypgeom_impl(args, true)
}

pub fn hypgeomdist_fn(args: &[Value]) -> Value {
    hypgeom_impl(args, false)
}

// ---------------------------------------------------------------------------
// EXPON.DIST / EXPONDIST
// ---------------------------------------------------------------------------
fn expon_impl(args: &[Value]) -> Value {
    if args.len() < 3 {
        return Value::Error(ErrorKind::NA);
    }
    let x = match as_f64(&args[0]) { Some(v) => v, None => return Value::Error(ErrorKind::Value) };
    let lambda = match as_f64(&args[1]) { Some(v) => v, None => return Value::Error(ErrorKind::Value) };
    let cumulative = match as_bool(&args[2]) {
        Some(b) => b,
        None => return Value::Error(ErrorKind::Value),
    };
    if x < 0.0 || lambda <= 0.0 {
        return Value::Error(ErrorKind::Num);
    }
    if cumulative {
        Value::Number(d::expon_cdf(x, lambda))
    } else {
        Value::Number(d::expon_pdf(x, lambda))
    }
}

pub fn expon_dist_fn(args: &[Value]) -> Value {
    expon_impl(args)
}

pub fn expondist_fn(args: &[Value]) -> Value {
    expon_impl(args)
}

// ---------------------------------------------------------------------------
// WEIBULL / WEIBULL.DIST
// ---------------------------------------------------------------------------
fn weibull_impl(args: &[Value]) -> Value {
    if args.len() < 4 {
        return Value::Error(ErrorKind::NA);
    }
    let x = match as_f64(&args[0]) { Some(v) => v, None => return Value::Error(ErrorKind::Value) };
    let alpha = match as_f64(&args[1]) { Some(v) => v, None => return Value::Error(ErrorKind::Value) };
    let beta = match as_f64(&args[2]) { Some(v) => v, None => return Value::Error(ErrorKind::Value) };
    let cumulative = match as_bool(&args[3]) {
        Some(b) => b,
        None => return Value::Error(ErrorKind::Value),
    };
    if x < 0.0 || alpha <= 0.0 || beta <= 0.0 {
        return Value::Error(ErrorKind::Num);
    }
    if cumulative {
        Value::Number(d::weibull_cdf(x, alpha, beta))
    } else {
        Value::Number(d::weibull_pdf(x, alpha, beta))
    }
}

pub fn weibull_fn(args: &[Value]) -> Value {
    weibull_impl(args)
}

pub fn weibull_dist_fn(args: &[Value]) -> Value {
    weibull_impl(args)
}

// ---------------------------------------------------------------------------
// LOGNORM.DIST / LOGNORMDIST
// ---------------------------------------------------------------------------
fn lognorm_dist_impl(args: &[Value]) -> Value {
    if args.len() < 4 {
        return Value::Error(ErrorKind::NA);
    }
    let x = match as_f64(&args[0]) { Some(v) => v, None => return Value::Error(ErrorKind::Value) };
    let mean = match as_f64(&args[1]) { Some(v) => v, None => return Value::Error(ErrorKind::Value) };
    let stdev = match as_f64(&args[2]) { Some(v) => v, None => return Value::Error(ErrorKind::Value) };
    let cumulative = match as_bool(&args[3]) {
        Some(b) => b,
        None => return Value::Error(ErrorKind::Value),
    };
    if x <= 0.0 || stdev <= 0.0 {
        return Value::Error(ErrorKind::Num);
    }
    if cumulative {
        Value::Number(d::lognorm_cdf(x, mean, stdev))
    } else {
        Value::Number(d::lognorm_pdf(x, mean, stdev))
    }
}

pub fn lognorm_dist_fn(args: &[Value]) -> Value {
    lognorm_dist_impl(args)
}

// LOGNORMDIST (legacy: 3 args, always CDF)
pub fn lognormdist_fn(args: &[Value]) -> Value {
    if args.len() < 3 {
        return Value::Error(ErrorKind::NA);
    }
    let x = match as_f64(&args[0]) { Some(v) => v, None => return Value::Error(ErrorKind::Value) };
    let mean = match as_f64(&args[1]) { Some(v) => v, None => return Value::Error(ErrorKind::Value) };
    let stdev = match as_f64(&args[2]) { Some(v) => v, None => return Value::Error(ErrorKind::Value) };
    if x <= 0.0 || stdev <= 0.0 {
        return Value::Error(ErrorKind::Num);
    }
    Value::Number(d::lognorm_cdf(x, mean, stdev))
}

// ---------------------------------------------------------------------------
// LOGNORM.INV / LOGINV
// ---------------------------------------------------------------------------
fn lognorm_inv_impl(args: &[Value]) -> Value {
    if args.len() < 3 {
        return Value::Error(ErrorKind::NA);
    }
    let p = match as_f64(&args[0]) { Some(v) => v, None => return Value::Error(ErrorKind::Value) };
    let mean = match as_f64(&args[1]) { Some(v) => v, None => return Value::Error(ErrorKind::Value) };
    let stdev = match as_f64(&args[2]) { Some(v) => v, None => return Value::Error(ErrorKind::Value) };
    if p <= 0.0 || p >= 1.0 || stdev <= 0.0 {
        return Value::Error(ErrorKind::Num);
    }
    let v = d::lognorm_inv(p, mean, stdev);
    if !v.is_finite() {
        Value::Error(ErrorKind::Num)
    } else {
        Value::Number(v)
    }
}

pub fn lognorm_inv_fn(args: &[Value]) -> Value {
    lognorm_inv_impl(args)
}

pub fn loginv_fn(args: &[Value]) -> Value {
    lognorm_inv_impl(args)
}

// ---------------------------------------------------------------------------
// FISHER / FISHERINV
// ---------------------------------------------------------------------------
pub fn fisher_fn(args: &[Value]) -> Value {
    if args.is_empty() {
        return Value::Error(ErrorKind::NA);
    }
    let x = match as_f64(&args[0]) { Some(v) => v, None => return Value::Error(ErrorKind::Value) };
    if x <= -1.0 || x >= 1.0 {
        return Value::Error(ErrorKind::Num);
    }
    Value::Number(d::fisher(x))
}

pub fn fisher_inv_fn(args: &[Value]) -> Value {
    if args.is_empty() {
        return Value::Error(ErrorKind::NA);
    }
    let y = match as_f64(&args[0]) { Some(v) => v, None => return Value::Error(ErrorKind::Value) };
    Value::Number(d::fisher_inv(y))
}

// ---------------------------------------------------------------------------
// PERMUT / PERMUTATIONA
// ---------------------------------------------------------------------------
pub fn permut_fn(args: &[Value]) -> Value {
    if args.len() < 2 {
        return Value::Error(ErrorKind::NA);
    }
    let n = match as_f64(&args[0]) { Some(v) => v as i64, None => return Value::Error(ErrorKind::Value) };
    let k = match as_f64(&args[1]) { Some(v) => v as i64, None => return Value::Error(ErrorKind::Value) };
    if n < 0 || k < 0 || k > n {
        return Value::Error(ErrorKind::Num);
    }
    // n! / (n-k)!
    let mut result = 1.0_f64;
    for i in (n - k + 1)..=n {
        result *= i as f64;
        if !result.is_finite() {
            return Value::Error(ErrorKind::Num);
        }
    }
    Value::Number(result)
}

// PERMUTATIONA: repetition allowed = n^k
pub fn permutationa_fn(args: &[Value]) -> Value {
    if args.len() < 2 {
        return Value::Error(ErrorKind::NA);
    }
    let n = match as_f64(&args[0]) { Some(v) => v as i64, None => return Value::Error(ErrorKind::Value) };
    let k = match as_f64(&args[1]) { Some(v) => v as i64, None => return Value::Error(ErrorKind::Value) };
    if n < 0 || k <= 0 {
        return Value::Error(ErrorKind::Num);
    }
    let v = (n as f64).powi(k as i32);
    if !v.is_finite() {
        Value::Error(ErrorKind::Num)
    } else {
        Value::Number(v)
    }
}

// ---------------------------------------------------------------------------
// PROB
// ---------------------------------------------------------------------------
pub fn prob_fn(args: &[Value]) -> Value {
    if args.len() < 3 {
        return Value::Error(ErrorKind::NA);
    }
    let xs = collect_nums(std::slice::from_ref(&args[0]));
    let probs = collect_nums(std::slice::from_ref(&args[1]));
    let lo = match as_f64(&args[2]) { Some(v) => v, None => return Value::Error(ErrorKind::Value) };
    let hi = if args.len() >= 4 {
        match as_f64(&args[3]) { Some(v) => v, None => return Value::Error(ErrorKind::Value) }
    } else { lo };
    if xs.len() != probs.len() || xs.is_empty() {
        return Value::Error(ErrorKind::NA);
    }
    // Sum probabilities for values in [lo, hi]
    let total: f64 = xs.iter().zip(probs.iter())
        .filter(|(x, _)| **x >= lo && **x <= hi)
        .map(|(_, p)| p)
        .sum();
    Value::Number(total)
}

// ---------------------------------------------------------------------------
// Z.TEST / ZTEST
// ---------------------------------------------------------------------------
fn z_test_impl(args: &[Value]) -> Value {
    if args.len() < 2 {
        return Value::Error(ErrorKind::NA);
    }
    let data = collect_nums(std::slice::from_ref(&args[0]));
    let mu0 = match as_f64(&args[1]) { Some(v) => v, None => return Value::Error(ErrorKind::Value) };
    let n = data.len() as f64;
    if n < 1.0 {
        return Value::Error(ErrorKind::NA);
    }
    let sigma = if args.len() >= 3 {
        match as_f64(&args[2]) {
            Some(v) => {
                if v <= 0.0 { return Value::Error(ErrorKind::Num); }
                v
            }
            None => return Value::Error(ErrorKind::Value),
        }
    } else {
        // Use sample stdev
        let mean = data.iter().sum::<f64>() / n;
        let var = data.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / (n - 1.0);
        if n < 2.0 { return Value::Error(ErrorKind::DivByZero); }
        var.sqrt()
    };
    let mean = data.iter().sum::<f64>() / n;
    let z = (mean - mu0) / (sigma / n.sqrt());
    // One-tailed right-tail p-value
    Value::Number(1.0 - d::norm_s_cdf(z))
}

pub fn z_test_fn(args: &[Value]) -> Value {
    z_test_impl(args)
}

pub fn ztest_fn(args: &[Value]) -> Value {
    z_test_impl(args)
}

// ---------------------------------------------------------------------------
// MARGINOFERROR
// ---------------------------------------------------------------------------
pub fn marginoferror_fn(args: &[Value]) -> Value {
    if args.len() < 2 {
        return Value::Error(ErrorKind::NA);
    }
    let data = collect_nums(std::slice::from_ref(&args[0]));
    let confidence = match as_f64(&args[1]) { Some(v) => v, None => return Value::Error(ErrorKind::Value) };
    let n = data.len() as f64;
    if n < 2.0 {
        return Value::Error(ErrorKind::NA);
    }
    let mean = data.iter().sum::<f64>() / n;
    let var = data.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / (n - 1.0);
    if var == 0.0 {
        return Value::Error(ErrorKind::Num);
    }
    let stdev = var.sqrt();
    let alpha = 1.0 - confidence;
    // Use t-distribution (n-1 degrees of freedom)
    let df = n - 1.0;
    let t = d::t_inv(1.0 - alpha / 2.0, df);
    if !t.is_finite() {
        return Value::Error(ErrorKind::Num);
    }
    Value::Number(t * stdev / n.sqrt())
}

// ---------------------------------------------------------------------------
// COVAR (legacy, population covariance)
// ---------------------------------------------------------------------------
pub fn covar_fn(args: &[Value]) -> Value {
    if args.len() < 2 {
        return Value::Error(ErrorKind::NA);
    }
    let xs = collect_nums(std::slice::from_ref(&args[0]));
    let ys = collect_nums(std::slice::from_ref(&args[1]));
    if xs.len() != ys.len() || xs.is_empty() {
        return Value::Error(ErrorKind::NA);
    }
    let n = xs.len() as f64;
    let mean_x = xs.iter().sum::<f64>() / n;
    let mean_y = ys.iter().sum::<f64>() / n;
    let cov = xs.iter().zip(ys.iter())
        .map(|(x, y)| (x - mean_x) * (y - mean_y))
        .sum::<f64>() / n;
    Value::Number(cov)
}

#[cfg(test)]
mod tests;
