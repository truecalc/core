//! Statistical distribution math primitives.
//!
//! All implemented in pure Rust f64 math — no external crates required.

#![allow(clippy::excessive_precision)]
#![allow(clippy::manual_range_contains)]

use std::f64::consts::{PI, SQRT_2, E};

// ---------------------------------------------------------------------------
// Error function (erf / erfc)
// ---------------------------------------------------------------------------

/// Approximation of erf(x) using Horner's method (Abramowitz & Stegun 7.1.26).
/// Max error ≈ 1.5e-7.
pub fn erf(x: f64) -> f64 {
    if x == 0.0 {
        return 0.0;
    }
    if x < 0.0 {
        return -erf(-x);
    }
    let t = 1.0 / (1.0 + 0.3275911 * x);
    let poly = t * (0.254829592
        + t * (-0.284496736
            + t * (1.421413741 + t * (-1.453152027 + t * 1.061405429))));
    1.0 - poly * (-x * x).exp()
}

pub fn erfc(x: f64) -> f64 {
    1.0 - erf(x)
}

// ---------------------------------------------------------------------------
// Normal distribution
// ---------------------------------------------------------------------------

/// Standard normal CDF Φ(x).
pub fn norm_s_cdf(x: f64) -> f64 {
    0.5 * erfc(-x / SQRT_2)
}

/// Standard normal PDF φ(x).
pub fn norm_s_pdf(x: f64) -> f64 {
    (-0.5 * x * x).exp() / (2.0 * PI).sqrt()
}

/// Normal CDF with mean and stdev.
pub fn norm_cdf(x: f64, mean: f64, stdev: f64) -> f64 {
    norm_s_cdf((x - mean) / stdev)
}

/// Normal PDF with mean and stdev.
pub fn norm_pdf(x: f64, mean: f64, stdev: f64) -> f64 {
    norm_s_pdf((x - mean) / stdev) / stdev
}

/// Inverse standard normal CDF (Peter Acklam's rational approximation).
/// Max error < 1.15e-9. Handles p in (0, 1); returns ±∞ for edges.
pub fn norm_s_inv(p: f64) -> f64 {
    if p <= 0.0 {
        return f64::NEG_INFINITY;
    }
    if p >= 1.0 {
        return f64::INFINITY;
    }
    const A: [f64; 6] = [
        -3.969683028665376e+01, 2.209460984245205e+02,
        -2.759285104469687e+02, 1.383577518672690e+02,
        -3.066479806614716e+01, 2.506628277459239e+00,
    ];
    const B: [f64; 5] = [
        -5.447609879822406e+01, 1.615858368580409e+02,
        -1.556989798598866e+02, 6.680131188771972e+01,
        -1.328068155288572e+01,
    ];
    const C: [f64; 6] = [
        -7.784894002430293e-03, -3.223964580411365e-01,
        -2.400758277161838e+00, -2.549732539343734e+00,
         4.374664141464968e+00,  2.938163982698783e+00,
    ];
    const D: [f64; 4] = [
        7.784695709041462e-03, 3.224671290700398e-01,
        2.445134137142996e+00, 3.754408661907416e+00,
    ];
    const P_LOW: f64 = 0.02425;
    const P_HIGH: f64 = 1.0 - P_LOW;

    if (P_LOW..=P_HIGH).contains(&p) {
        let q = p - 0.5;
        let r = q * q;
        (((((A[0]*r+A[1])*r+A[2])*r+A[3])*r+A[4])*r+A[5])*q
            / (((((B[0]*r+B[1])*r+B[2])*r+B[3])*r+B[4])*r+1.0)
    } else {
        let q = if p < P_LOW {
            (-2.0 * p.ln()).sqrt()
        } else {
            (-2.0 * (1.0 - p).ln()).sqrt()
        };
        let x = (((((C[0]*q+C[1])*q+C[2])*q+C[3])*q+C[4])*q+C[5])
            / ((((D[0]*q+D[1])*q+D[2])*q+D[3])*q+1.0);
        if p < P_LOW { x } else { -x }
    }
}

/// Inverse normal CDF with mean/stdev.
pub fn norm_inv(p: f64, mean: f64, stdev: f64) -> f64 {
    mean + stdev * norm_s_inv(p)
}

// ---------------------------------------------------------------------------
// Gamma function (Lanczos approximation)
// ---------------------------------------------------------------------------

/// ln(Γ(x)) for x > 0 using Lanczos approximation (g=7).
pub fn lgamma(x: f64) -> f64 {
    if x <= 0.0 {
        return f64::INFINITY;
    }
    // Stirling for large x
    if x > 200.0 {
        return (x - 0.5) * (x).ln() - x + 0.5 * (2.0 * PI).ln()
            + 1.0 / (12.0 * x);
    }
    // Lanczos g=7, n=9 coefficients
    let g = 7.0_f64;
    let c = [
        0.99999999999980993_f64,
        676.5203681218851,
        -1259.1392167224028,
        771.32342877765313,
        -176.61502916214059,
        12.507343278686905,
        -0.13857109526572012,
        9.9843695780195716e-6,
        1.5056327351493116e-7,
    ];
    let mut x = x;
    if x < 0.5 {
        // Reflection: Γ(x)Γ(1-x) = π/sin(πx)
        return (PI / ((PI * x).sin())).ln() - lgamma(1.0 - x);
    }
    x -= 1.0;
    let mut a = c[0];
    let t = x + g + 0.5;
    for (i, &ci) in c[1..].iter().enumerate() {
        a += ci / (x + (i + 1) as f64);
    }
    0.5 * (2.0 * PI).ln() + (x + 0.5) * t.ln() - t + a.ln()
}

/// Γ(x) = exp(lgamma(x)).
pub fn gamma_fn(x: f64) -> f64 {
    if x <= 0.0 && x == x.floor() {
        return f64::INFINITY; // poles at non-positive integers
    }
    lgamma(x).exp()
}

// ---------------------------------------------------------------------------
// Regularized incomplete gamma functions
// ---------------------------------------------------------------------------

/// Regularized lower incomplete gamma P(a, x) = γ(a,x)/Γ(a).
/// P(a, x) is the CDF of the Gamma(a,1) distribution.
pub fn reg_inc_gamma_lower(a: f64, x: f64) -> f64 {
    if x < 0.0 {
        return 0.0;
    }
    if x == 0.0 {
        return 0.0;
    }
    if x < a + 1.0 {
        gamma_series(a, x)
    } else {
        1.0 - gamma_cf(a, x)
    }
}

/// Regularized upper incomplete gamma Q(a, x) = 1 - P(a, x).
pub fn reg_inc_gamma_upper(a: f64, x: f64) -> f64 {
    1.0 - reg_inc_gamma_lower(a, x)
}

/// Series expansion for P(a, x).
fn gamma_series(a: f64, x: f64) -> f64 {
    let max_iter = 200;
    let eps = 3.0e-14;
    let lga = lgamma(a);
    let mut ap = a;
    let mut sum = 1.0 / a;
    let mut delta = sum;
    for _ in 0..max_iter {
        ap += 1.0;
        delta *= x / ap;
        sum += delta;
        if delta.abs() < sum.abs() * eps {
            break;
        }
    }
    sum * (-x + a * x.ln() - lga).exp()
}

/// Continued fraction for Q(a, x) = 1 - P(a, x).
fn gamma_cf(a: f64, x: f64) -> f64 {
    let max_iter = 200;
    let eps = 3.0e-14;
    let lga = lgamma(a);
    let fpmin = 1e-300_f64;
    let mut b = x + 1.0 - a;
    let mut c = 1.0 / fpmin;
    let mut d = 1.0 / b;
    let mut h = d;
    for i in 1..=max_iter {
        let an = -(i as f64) * ((i as f64) - a);
        b += 2.0;
        d = an * d + b;
        if d.abs() < fpmin {
            d = fpmin;
        }
        c = b + an / c;
        if c.abs() < fpmin {
            c = fpmin;
        }
        d = 1.0 / d;
        let del = d * c;
        h *= del;
        if (del - 1.0).abs() < eps {
            break;
        }
    }
    h * (-x + a * x.ln() - lga).exp()
}

/// Inverse of regularized incomplete gamma P(a, x) = p → find x.
pub fn inv_reg_inc_gamma(a: f64, p: f64) -> f64 {
    if p <= 0.0 {
        return 0.0;
    }
    if p >= 1.0 {
        return f64::INFINITY;
    }
    // Initial approximation using Wilson-Hilferty
    let mut x = if a > 1.0 {
        let x0 = norm_s_inv(p);
        let h = 2.0 / (9.0 * a);
        let t = 1.0 - h + x0 * h.sqrt();
        if t > 0.0 { a * t.powi(3) } else { a * 0.5_f64.powf(1.0 / a) }
    } else {
        let t = 1.0 - a * (0.253 + a * 0.12);
        if p < t {
            (p / t).powf(1.0 / a)
        } else {
            1.0 - (1.0 - (p - t) / (1.0 - t)).ln()
        }
    };
    if x <= 0.0 {
        x = 0.1;
    }
    // Newton-Raphson with backtracking to prevent overshoot
    let lga = lgamma(a);
    for _ in 0..100 {
        let fx = reg_inc_gamma_lower(a, x) - p;
        // derivative = x^(a-1) * exp(-x) / Gamma(a)
        let log_dfx = (a - 1.0) * x.ln() - x - lga;
        if log_dfx < -500.0 {
            // Derivative underflowed — switch to bisection step
            x *= if fx < 0.0 { 2.0 } else { 0.5 };
            continue;
        }
        let dfx = log_dfx.exp();
        if dfx == 0.0 {
            break;
        }
        let dx = fx / dfx;
        // Backtracking: prevent x from going negative or moving too far
        let mut step = 1.0_f64;
        while x - step * dx <= 0.0 {
            step *= 0.5;
            if step < 1e-10 {
                break;
            }
        }
        x -= step * dx;
        if x <= 0.0 {
            x = 1e-10;
        }
        if (step * dx).abs() < 1e-10 * x.abs() + 1e-14 {
            break;
        }
    }
    x
}

// ---------------------------------------------------------------------------
// Chi-squared distribution (special case of Gamma)
// ---------------------------------------------------------------------------

/// Chi-squared CDF: P(χ² ≤ x | df).
pub fn chisq_cdf(x: f64, df: f64) -> f64 {
    if x < 0.0 {
        return 0.0;
    }
    reg_inc_gamma_lower(df / 2.0, x / 2.0)
}

/// Chi-squared PDF.
pub fn chisq_pdf(x: f64, df: f64) -> f64 {
    if x <= 0.0 {
        return 0.0;
    }
    let k = df / 2.0;
    (x.powf(k - 1.0) * (-x / 2.0).exp()) / (2.0_f64.powf(k) * gamma_fn(k))
}

/// Inverse chi-squared CDF (quantile function).
pub fn chisq_inv(p: f64, df: f64) -> f64 {
    2.0 * inv_reg_inc_gamma(df / 2.0, p)
}

// ---------------------------------------------------------------------------
// Regularized incomplete beta function
// ---------------------------------------------------------------------------

/// Regularized incomplete beta I_x(a, b).
pub fn reg_inc_beta(a: f64, b: f64, x: f64) -> f64 {
    if x <= 0.0 {
        return 0.0;
    }
    if x >= 1.0 {
        return 1.0;
    }
    let lbeta = lgamma(a) + lgamma(b) - lgamma(a + b);
    let bt = (a * x.ln() + b * (1.0 - x).ln() - lbeta).exp();
    if x < (a + 1.0) / (a + b + 2.0) {
        bt * betacf(a, b, x) / a
    } else {
        1.0 - bt * betacf(b, a, 1.0 - x) / b
    }
}

/// Continued fraction for regularized incomplete beta.
fn betacf(a: f64, b: f64, x: f64) -> f64 {
    let max_iter = 200;
    let eps = 3.0e-14;
    let fpmin = 1e-300_f64;
    let qab = a + b;
    let qap = a + 1.0;
    let qam = a - 1.0;
    let mut c = 1.0_f64;
    let mut d = 1.0 - qab * x / qap;
    if d.abs() < fpmin {
        d = fpmin;
    }
    d = 1.0 / d;
    let mut h = d;
    for m in 1..=max_iter {
        let m = m as f64;
        let m2 = 2.0 * m;
        // Even step
        let aa = m * (b - m) * x / ((qam + m2) * (a + m2));
        d = 1.0 + aa * d;
        if d.abs() < fpmin { d = fpmin; }
        c = 1.0 + aa / c;
        if c.abs() < fpmin { c = fpmin; }
        d = 1.0 / d;
        h *= d * c;
        // Odd step
        let aa = -(a + m) * (qab + m) * x / ((a + m2) * (qap + m2));
        d = 1.0 + aa * d;
        if d.abs() < fpmin { d = fpmin; }
        c = 1.0 + aa / c;
        if c.abs() < fpmin { c = fpmin; }
        d = 1.0 / d;
        let del = d * c;
        h *= del;
        if (del - 1.0).abs() < eps {
            break;
        }
    }
    h
}

/// Inverse of regularized incomplete beta I_x(a, b) = p → find x.
pub fn inv_reg_inc_beta(a: f64, b: f64, p: f64) -> f64 {
    if p <= 0.0 {
        return 0.0;
    }
    if p >= 1.0 {
        return 1.0;
    }
    // Initial approximation via normal approximation
    let mut x = {
        let w = norm_s_inv(p);
        // Cornish-Fisher approximation
        let lam = a / (a + b);
        let t = w * (lam * (1.0 - lam) / (a + b + 1.0)).sqrt();
        let cf = lam + t;
        if cf > 0.0 && cf < 1.0 {
            cf
        } else {
            // Cornish-Fisher failed (common for extreme p); use a small-x
            // approximation: I_x(a,b) ≈ x^a / (a * B(a,b)) for small x
            let lbeta_ab = lgamma(a) + lgamma(b) - lgamma(a + b);
            let x0 = (p * a * lbeta_ab.exp()).powf(1.0 / a);
            x0.clamp(1e-14, 1.0 - 1e-14)
        }
    };
    // Newton-Raphson with bisection bounds
    let lbeta_ab = lgamma(a) + lgamma(b) - lgamma(a + b);
    let mut lo = 0.0_f64;
    let mut hi = 1.0_f64;
    for _ in 0..100 {
        let fx = reg_inc_beta(a, b, x) - p;
        // Update bisection bounds
        if fx < 0.0 { lo = x; } else { hi = x; }
        let log_dfx = (a - 1.0) * x.ln() + (b - 1.0) * (1.0 - x).ln() - lbeta_ab;
        if log_dfx < -500.0 || log_dfx.is_nan() {
            // Bisect
            x = (lo + hi) / 2.0;
            continue;
        }
        let dfx = log_dfx.exp();
        if dfx == 0.0 {
            x = (lo + hi) / 2.0;
            continue;
        }
        let dx = fx / dfx;
        let clamp_lo = lo + 1e-14;
        let clamp_hi = hi - 1e-14;
        let x_new = if clamp_lo >= clamp_hi {
            // Bisection interval collapsed — take midpoint and stop
            (lo + hi) / 2.0
        } else {
            (x - dx).clamp(clamp_lo, clamp_hi)
        };
        let converged = (x_new - x).abs() < 1e-12;
        x = x_new;
        if converged {
            break;
        }
    }
    x
}

// ---------------------------------------------------------------------------
// T-distribution
// ---------------------------------------------------------------------------

/// Student's t CDF P(T ≤ x | df) (two-sided, left-tail).
pub fn t_cdf(x: f64, df: f64) -> f64 {
    let p = reg_inc_beta(df / 2.0, 0.5, df / (df + x * x));
    if x >= 0.0 {
        1.0 - 0.5 * p
    } else {
        0.5 * p
    }
}

/// Student's t PDF.
pub fn t_pdf(x: f64, df: f64) -> f64 {
    let lbeta = lgamma(df / 2.0) + lgamma(0.5) - lgamma((df + 1.0) / 2.0);
    let coeff = (-lbeta).exp() / df.sqrt();
    coeff * (1.0 + x * x / df).powf(-(df + 1.0) / 2.0)
}

/// Inverse t CDF (one-tailed, left-tail p).
pub fn t_inv(p: f64, df: f64) -> f64 {
    if p <= 0.0 || p >= 1.0 || df <= 0.0 {
        return f64::NAN;
    }
    // Use inverse beta
    let x = inv_reg_inc_beta(df / 2.0, 0.5, 2.0 * p.min(1.0 - p));
    let t = (df * (1.0 - x) / x).sqrt();
    if p < 0.5 { -t } else { t }
}

// ---------------------------------------------------------------------------
// F-distribution
// ---------------------------------------------------------------------------

/// F CDF P(F ≤ x | df1, df2).
pub fn f_cdf(x: f64, df1: f64, df2: f64) -> f64 {
    if x <= 0.0 {
        return 0.0;
    }
    reg_inc_beta(df1 / 2.0, df2 / 2.0, df1 * x / (df1 * x + df2))
}

/// F PDF.
pub fn f_pdf(x: f64, df1: f64, df2: f64) -> f64 {
    if x <= 0.0 {
        return 0.0;
    }
    let lbeta = lgamma(df1 / 2.0) + lgamma(df2 / 2.0) - lgamma((df1 + df2) / 2.0);
    let lx = (df1 / 2.0) * (df1 / df2).ln()
        + (df1 / 2.0 - 1.0) * x.ln()
        - ((df1 + df2) / 2.0) * (1.0 + df1 * x / df2).ln()
        - lbeta;
    lx.exp()
}

/// Inverse F CDF.
pub fn f_inv(p: f64, df1: f64, df2: f64) -> f64 {
    if p <= 0.0 {
        return 0.0;
    }
    if p >= 1.0 {
        return f64::INFINITY;
    }
    let x = inv_reg_inc_beta(df1 / 2.0, df2 / 2.0, p);
    if x <= 0.0 || x >= 1.0 {
        return 0.0;
    }
    df2 * x / (df1 * (1.0 - x))
}

// ---------------------------------------------------------------------------
// Gamma distribution
// ---------------------------------------------------------------------------

/// Gamma(alpha, beta) CDF where beta is the scale parameter.
pub fn gamma_dist_cdf(x: f64, alpha: f64, beta: f64) -> f64 {
    if x <= 0.0 {
        return 0.0;
    }
    reg_inc_gamma_lower(alpha, x / beta)
}

/// Gamma(alpha, beta) PDF where beta is the scale parameter.
pub fn gamma_dist_pdf(x: f64, alpha: f64, beta: f64) -> f64 {
    if x <= 0.0 {
        return 0.0;
    }
    let lp = (alpha - 1.0) * x.ln() - x / beta - alpha * beta.ln() - lgamma(alpha);
    lp.exp()
}

/// Inverse Gamma(alpha, beta) CDF.
pub fn gamma_dist_inv(p: f64, alpha: f64, beta: f64) -> f64 {
    beta * inv_reg_inc_gamma(alpha, p)
}

// ---------------------------------------------------------------------------
// Beta distribution
// ---------------------------------------------------------------------------

/// Beta(alpha, beta) CDF on [0, 1].
pub fn beta_dist_cdf(x: f64, alpha: f64, beta: f64, lo: f64, hi: f64) -> f64 {
    let xn = (x - lo) / (hi - lo);
    reg_inc_beta(alpha, beta, xn)
}

/// Beta(alpha, beta) PDF.
pub fn beta_dist_pdf(x: f64, alpha: f64, beta: f64, lo: f64, hi: f64) -> f64 {
    let xn = (x - lo) / (hi - lo);
    if xn <= 0.0 || xn >= 1.0 {
        return 0.0;
    }
    let lbeta = lgamma(alpha) + lgamma(beta) - lgamma(alpha + beta);
    let lp = (alpha - 1.0) * xn.ln() + (beta - 1.0) * (1.0 - xn).ln() - lbeta
        - (hi - lo).ln();
    lp.exp()
}

/// Inverse Beta(alpha, beta) CDF on [lo, hi].
pub fn beta_dist_inv(p: f64, alpha: f64, beta: f64, lo: f64, hi: f64) -> f64 {
    lo + (hi - lo) * inv_reg_inc_beta(alpha, beta, p)
}

// ---------------------------------------------------------------------------
// Binomial distribution
// ---------------------------------------------------------------------------

/// Binomial PMF P(X = k | n, p).
pub fn binom_pmf(k: u64, n: u64, p: f64) -> f64 {
    if k > n {
        return 0.0;
    }
    binom_coeff_ln(n, k).exp() + (k as f64) * p.ln() + ((n - k) as f64) * (1.0 - p).ln()
}

fn binom_coeff_ln(n: u64, k: u64) -> f64 {
    lgamma((n + 1) as f64) - lgamma((k + 1) as f64) - lgamma((n - k + 1) as f64)
}

pub fn binom_coeff_ln_pub(n: u64, k: u64) -> f64 {
    binom_coeff_ln(n, k)
}

/// Binomial CDF P(X ≤ k | n, p).
pub fn binom_cdf(k: u64, n: u64, p: f64) -> f64 {
    if k >= n {
        return 1.0;
    }
    // I_{1-p}(n-k, k+1) = regularized incomplete beta
    reg_inc_beta((n - k) as f64, (k + 1) as f64, 1.0 - p)
}

/// Inverse binomial CDF (smallest k such that CDF(k) >= p).
pub fn binom_inv(n: u64, prob: f64, target_p: f64) -> u64 {
    let mut cumulative = 0.0;
    for k in 0..=n {
        let pmf = (binom_coeff_ln(n, k) + (k as f64) * prob.ln()
            + ((n - k) as f64) * (1.0 - prob).ln())
        .exp();
        cumulative += pmf;
        if cumulative >= target_p {
            return k;
        }
    }
    n
}

// ---------------------------------------------------------------------------
// Poisson distribution
// ---------------------------------------------------------------------------

/// Poisson PMF P(X = k | lambda).
pub fn poisson_pmf(k: u64, lambda: f64) -> f64 {
    if lambda <= 0.0 {
        return if k == 0 { 1.0 } else { 0.0 };
    }
    (-lambda + (k as f64) * lambda.ln() - lgamma((k + 1) as f64)).exp()
}

/// Poisson CDF P(X ≤ k | lambda).
pub fn poisson_cdf(k: u64, lambda: f64) -> f64 {
    // P(X ≤ k) = 1 - P(k+1, lambda) (regularized upper gamma)
    reg_inc_gamma_upper(k as f64 + 1.0, lambda)
}

// ---------------------------------------------------------------------------
// Negative binomial
// ---------------------------------------------------------------------------

/// Negative binomial PMF P(X = x | r, p): x failures before r-th success.
pub fn negbinom_pmf(x: u64, r: u64, p: f64) -> f64 {
    // Handle edge cases for p=0 or p=1
    if p == 1.0 {
        return if x == 0 { 1.0 } else { 0.0 };
    }
    if p == 0.0 {
        return 0.0;
    }
    let lcoeff = lgamma((x + r) as f64) - lgamma(r as f64) - lgamma((x + 1) as f64);
    (lcoeff + (r as f64) * p.ln() + (x as f64) * (1.0 - p).ln()).exp()
}

/// Negative binomial CDF P(X ≤ x | r, p).
pub fn negbinom_cdf(x: u64, r: u64, p: f64) -> f64 {
    reg_inc_beta(r as f64, (x + 1) as f64, p)
}

// ---------------------------------------------------------------------------
// Hypergeometric distribution
// ---------------------------------------------------------------------------

/// Hypergeometric PMF P(X = x | population, success_in_pop, sample_size).
pub fn hypgeom_pmf(x: u64, pop: u64, k: u64, n: u64) -> f64 {
    if x > k || x > n || (n - x) > (pop - k) {
        return 0.0;
    }
    let lnum = lgamma((k + 1) as f64) - lgamma((x + 1) as f64) - lgamma((k - x + 1) as f64)
        + lgamma((pop - k + 1) as f64)
        - lgamma((n - x + 1) as f64)
        - lgamma((pop - k - (n - x) + 1) as f64);
    let lden = lgamma((pop + 1) as f64) - lgamma((n + 1) as f64) - lgamma((pop - n + 1) as f64);
    (lnum - lden).exp()
}

/// Hypergeometric CDF.
pub fn hypgeom_cdf(x: u64, pop: u64, k: u64, n: u64) -> f64 {
    let mut sum = 0.0;
    let max_x = x.min(k).min(n);
    for i in 0..=max_x {
        sum += hypgeom_pmf(i, pop, k, n);
    }
    sum
}

// ---------------------------------------------------------------------------
// Exponential distribution
// ---------------------------------------------------------------------------

/// Exponential CDF: P(X ≤ x | lambda).
pub fn expon_cdf(x: f64, lambda: f64) -> f64 {
    if x < 0.0 {
        0.0
    } else {
        1.0 - (-lambda * x).exp()
    }
}

/// Exponential PDF: lambda * exp(-lambda * x).
pub fn expon_pdf(x: f64, lambda: f64) -> f64 {
    if x < 0.0 {
        0.0
    } else {
        lambda * (-lambda * x).exp()
    }
}

// ---------------------------------------------------------------------------
// Weibull distribution
// ---------------------------------------------------------------------------

/// Weibull CDF.
pub fn weibull_cdf(x: f64, alpha: f64, beta: f64) -> f64 {
    if x < 0.0 {
        0.0
    } else {
        1.0 - (-(x / beta).powf(alpha)).exp()
    }
}

/// Weibull PDF.
pub fn weibull_pdf(x: f64, alpha: f64, beta: f64) -> f64 {
    if x < 0.0 {
        0.0
    } else {
        (alpha / beta) * (x / beta).powf(alpha - 1.0) * (-(x / beta).powf(alpha)).exp()
    }
}

// ---------------------------------------------------------------------------
// Lognormal distribution
// ---------------------------------------------------------------------------

/// Lognormal CDF.
pub fn lognorm_cdf(x: f64, mean: f64, stdev: f64) -> f64 {
    if x <= 0.0 {
        return 0.0;
    }
    norm_s_cdf((x.ln() - mean) / stdev)
}

/// Lognormal PDF.
pub fn lognorm_pdf(x: f64, mean: f64, stdev: f64) -> f64 {
    if x <= 0.0 {
        return 0.0;
    }
    let z = (x.ln() - mean) / stdev;
    (-0.5 * z * z).exp() / (x * stdev * (2.0 * PI).sqrt())
}

/// Inverse lognormal CDF.
pub fn lognorm_inv(p: f64, mean: f64, stdev: f64) -> f64 {
    (mean + stdev * norm_s_inv(p)).exp()
}

// ---------------------------------------------------------------------------
// Fisher z-transform
// ---------------------------------------------------------------------------

pub fn fisher(x: f64) -> f64 {
    0.5 * ((1.0 + x) / (1.0 - x)).ln()
}

pub fn fisher_inv(y: f64) -> f64 {
    (E.powf(2.0 * y) - 1.0) / (E.powf(2.0 * y) + 1.0)
}

// ---------------------------------------------------------------------------
// Linear regression helpers
// ---------------------------------------------------------------------------

/// Compute slope and intercept of linear regression y on x.
/// Returns (slope, intercept) or None if denominator is zero.
pub fn linear_regression(xs: &[f64], ys: &[f64]) -> Option<(f64, f64)> {
    let n = xs.len() as f64;
    if n == 0.0 || xs.len() != ys.len() {
        return None;
    }
    let mean_x = xs.iter().sum::<f64>() / n;
    let mean_y = ys.iter().sum::<f64>() / n;
    let ss_xx: f64 = xs.iter().map(|&x| (x - mean_x).powi(2)).sum();
    if ss_xx == 0.0 {
        return None;
    }
    let ss_xy: f64 = xs.iter().zip(ys.iter()).map(|(&x, &y)| (x - mean_x) * (y - mean_y)).sum();
    let slope = ss_xy / ss_xx;
    let intercept = mean_y - slope * mean_x;
    Some((slope, intercept))
}

/// Pearson correlation coefficient.
pub fn pearson_corr(xs: &[f64], ys: &[f64]) -> Option<f64> {
    let n = xs.len() as f64;
    if n < 2.0 || xs.len() != ys.len() {
        return None;
    }
    let mean_x = xs.iter().sum::<f64>() / n;
    let mean_y = ys.iter().sum::<f64>() / n;
    let ss_xx: f64 = xs.iter().map(|&x| (x - mean_x).powi(2)).sum();
    let ss_yy: f64 = ys.iter().map(|&y| (y - mean_y).powi(2)).sum();
    let ss_xy: f64 = xs.iter().zip(ys.iter()).map(|(&x, &y)| (x - mean_x) * (y - mean_y)).sum();
    let denom = (ss_xx * ss_yy).sqrt();
    if denom == 0.0 {
        None
    } else {
        Some(ss_xy / denom)
    }
}
