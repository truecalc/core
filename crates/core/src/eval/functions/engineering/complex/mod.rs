use crate::eval::coercion::{to_number, to_string_val};
use crate::eval::functions::check_arity;
use crate::types::{ErrorKind, Value};

// ── Internal complex type ─────────────────────────────────────────────────────

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct Complex {
    pub re: f64,
    pub im: f64,
}

impl Complex {
    fn new(re: f64, im: f64) -> Self {
        Self { re, im }
    }

    fn abs(self) -> f64 {
        (self.re * self.re + self.im * self.im).sqrt()
    }

    fn arg(self) -> f64 {
        self.im.atan2(self.re)
    }

    fn mul(self, rhs: Self) -> Self {
        Self {
            re: self.re * rhs.re - self.im * rhs.im,
            im: self.re * rhs.im + self.im * rhs.re,
        }
    }

    fn pow(self, n: Self) -> Option<Self> {
        // c^n = exp(n * ln(c))
        let r = self.abs();
        if r == 0.0 {
            // 0^0 = 1 by convention; 0^n = 0 for n != 0
            if n.re == 0.0 && n.im == 0.0 {
                return Some(Complex::new(1.0, 0.0));
            }
            if n.re > 0.0 {
                return Some(Complex::new(0.0, 0.0));
            }
            return None; // 0^negative
        }
        let theta = self.arg();
        let ln_r = r.ln();
        // ln(c) = ln_r + i*theta
        // n * ln(c) = (n.re*ln_r - n.im*theta) + i*(n.im*ln_r + n.re*theta)
        let exp_re = n.re * ln_r - n.im * theta;
        let exp_im = n.im * ln_r + n.re * theta;
        let scale = exp_re.exp();
        Some(Complex::new(scale * exp_im.cos(), scale * exp_im.sin()))
    }

    fn sqrt(self) -> Self {
        // Principal square root
        let r = self.abs();
        let sqrt_r = r.sqrt();
        let theta = self.arg();
        Complex::new(sqrt_r * (theta / 2.0).cos(), sqrt_r * (theta / 2.0).sin())
    }

    fn ln(self) -> Option<Self> {
        let r = self.abs();
        if r == 0.0 {
            return None;
        }
        Some(Complex::new(r.ln(), self.arg()))
    }
}

// ── Complex string parsing / formatting ──────────────────────────────────────

/// Parse a complex number string like "3+4i", "3-4i", "i", "-i", "5", "2j", etc.
/// Returns None on parse failure.
pub(super) fn parse_complex(s: &str) -> Option<Complex> {
    let s = s.trim();
    if s.is_empty() {
        return None;
    }

    // Detect suffix ('i' or 'j')
    let suffix = if s.ends_with('i') || s.ends_with('j') {
        Some(s.chars().last().unwrap())
    } else {
        None
    };

    if suffix.is_none() {
        // Pure real number
        let re = s.parse::<f64>().ok()?;
        return Some(Complex::new(re, 0.0));
    }

    // Strip suffix
    let s = &s[..s.len() - 1];

    // Pure imaginary: "i", "-i", "+i"
    if s.is_empty() || s == "+" {
        return Some(Complex::new(0.0, 1.0));
    }
    if s == "-" {
        return Some(Complex::new(0.0, -1.0));
    }

    // Try parsing the whole thing as real (shouldn't happen but cover edge case)
    if !s.contains('+') && !s.contains('-') || s.starts_with('-') && s[1..].find(['+', '-']).is_none() {
        // E.g. "4i" or "-4i"
        let im = s.parse::<f64>().ok()?;
        return Some(Complex::new(0.0, im));
    }

    // Find the split point between real and imaginary parts.
    // We look for the last '+' or '-' that isn't at position 0 (sign of real).
    let bytes = s.as_bytes();
    let mut split = None;
    let start = if bytes[0] == b'-' || bytes[0] == b'+' { 1 } else { 0 };
    for i in (start + 1..bytes.len()).rev() {
        if bytes[i] == b'+' || bytes[i] == b'-' {
            split = Some(i);
            break;
        }
    }

    if let Some(idx) = split {
        let re_str = &s[..idx];
        let im_str = &s[idx..];

        let re = if re_str.is_empty() { 0.0 } else { re_str.parse::<f64>().ok()? };
        let im = if im_str == "+" || im_str.is_empty() {
            1.0
        } else if im_str == "-" {
            -1.0
        } else {
            im_str.parse::<f64>().ok()?
        };
        Some(Complex::new(re, im))
    } else {
        // No split found; it's pure imaginary like "4i"
        let im = s.parse::<f64>().ok()?;
        Some(Complex::new(0.0, im))
    }
}

/// Format a complex number back to a string using the given suffix ('i' or 'j').
pub(super) fn format_complex(c: Complex, suffix: char) -> Value {
    let re = c.re;
    let im = c.im;

    // Clean up near-zero values
    let re = if re.abs() < 1e-10 { 0.0 } else { re };
    let im = if im.abs() < 1e-10 { 0.0 } else { im };

    if im == 0.0 {
        return Value::Number(re);
    }

    let re_str = if re == 0.0 {
        String::new()
    } else {
        format_num(re)
    };

    let im_str = if im == 1.0 {
        suffix.to_string()
    } else if im == -1.0 {
        format!("-{}", suffix)
    } else {
        format!("{}{}", format_num(im), suffix)
    };

    let result = if re == 0.0 {
        im_str
    } else if im > 0.0 || im == 1.0 {
        format!("{}+{}", re_str, im_str)
    } else {
        format!("{}{}", re_str, im_str)
    };

    Value::Text(result)
}

fn format_num(n: f64) -> String {
    // Use integer if it's a whole number
    if n.fract() == 0.0 && n.abs() < 1e15 {
        format!("{}", n as i64)
    } else {
        format!("{}", n)
    }
}

/// Parse a Value as a complex number. Accepts Text or Number.
fn value_to_complex(v: Value) -> Result<Complex, Value> {
    match v {
        Value::Number(n) | Value::Date(n) => Ok(Complex::new(n, 0.0)),
        Value::Text(s) => {
            parse_complex(&s).ok_or(Value::Error(ErrorKind::Value))
        }
        Value::Error(_) => Err(v),
        _ => {
            match to_number(v) {
                Ok(n) => Ok(Complex::new(n, 0.0)),
                Err(e) => Err(e),
            }
        }
    }
}

// ── COMPLEX ───────────────────────────────────────────────────────────────────

/// `COMPLEX(real, imaginary, [suffix])` — create a complex number string.
pub fn complex_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 2, 3) {
        return err;
    }
    let re = match to_number(args[0].clone()) {
        Err(e) => return e,
        Ok(v) => v,
    };
    let im = match to_number(args[1].clone()) {
        Err(e) => return e,
        Ok(v) => v,
    };
    let suffix = if args.len() == 3 {
        match to_string_val(args[2].clone()) {
            Err(e) => return e,
            Ok(s) => {
                if s == "i" || s == "j" {
                    s.chars().next().unwrap()
                } else {
                    return Value::Error(ErrorKind::Value);
                }
            }
        }
    } else {
        'i'
    };
    format_complex(Complex::new(re, im), suffix)
}

// ── IMREAL / IMAGINARY ────────────────────────────────────────────────────────

/// `IMREAL(complex)` — return real part of complex number.
pub fn imreal_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 1, 1) {
        return err;
    }
    match value_to_complex(args[0].clone()) {
        Err(e) => e,
        Ok(c) => Value::Number(c.re),
    }
}

/// `IMAGINARY(complex)` — return imaginary part of complex number.
pub fn imaginary_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 1, 1) {
        return err;
    }
    match value_to_complex(args[0].clone()) {
        Err(e) => e,
        Ok(c) => Value::Number(c.im),
    }
}

// ── IMABS ─────────────────────────────────────────────────────────────────────

/// `IMABS(complex)` — return absolute value (modulus) of complex number.
pub fn imabs_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 1, 1) {
        return err;
    }
    match value_to_complex(args[0].clone()) {
        Err(e) => e,
        Ok(c) => Value::Number(c.abs()),
    }
}

// ── IMPRODUCT ─────────────────────────────────────────────────────────────────

/// `IMPRODUCT(complex1, ...)` — product of complex numbers.
pub fn improduct_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 1, usize::MAX) {
        return err;
    }
    let mut result = Complex::new(1.0, 0.0);
    for arg in args {
        match value_to_complex(arg.clone()) {
            Err(e) => return e,
            Ok(c) => result = result.mul(c),
        }
    }
    format_complex(result, 'i')
}

// ── IMSUB ────────────────────────────────────────────────────────────────────

/// `IMSUB(complex1, complex2)` — subtract complex numbers.
pub fn imsub_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 2, 2) {
        return err;
    }
    let a = match value_to_complex(args[0].clone()) {
        Err(e) => return e,
        Ok(c) => c,
    };
    let b = match value_to_complex(args[1].clone()) {
        Err(e) => return e,
        Ok(c) => c,
    };
    format_complex(Complex::new(a.re - b.re, a.im - b.im), 'i')
}

// ── IMSUM ────────────────────────────────────────────────────────────────────

/// `IMSUM(complex1, ...)` — sum of complex numbers.
pub fn imsum_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 1, usize::MAX) {
        return err;
    }
    let mut re = 0.0f64;
    let mut im = 0.0f64;
    for arg in args {
        match value_to_complex(arg.clone()) {
            Err(e) => return e,
            Ok(c) => {
                re += c.re;
                im += c.im;
            }
        }
    }
    format_complex(Complex::new(re, im), 'i')
}

// ── IMDIV ────────────────────────────────────────────────────────────────────

/// `IMDIV(complex1, complex2)` — divide complex numbers.
pub fn imdiv_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 2, 2) {
        return err;
    }
    let a = match value_to_complex(args[0].clone()) {
        Err(e) => return e,
        Ok(c) => c,
    };
    let b = match value_to_complex(args[1].clone()) {
        Err(e) => return e,
        Ok(c) => c,
    };
    let denom = b.re * b.re + b.im * b.im;
    if denom == 0.0 {
        return Value::Error(ErrorKind::DivByZero);
    }
    let re = (a.re * b.re + a.im * b.im) / denom;
    let im = (a.im * b.re - a.re * b.im) / denom;
    format_complex(Complex::new(re, im), 'i')
}

// ── IMCONJUGATE ───────────────────────────────────────────────────────────────

/// `IMCONJUGATE(complex)` — complex conjugate.
pub fn imconjugate_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 1, 1) {
        return err;
    }
    match value_to_complex(args[0].clone()) {
        Err(e) => e,
        Ok(c) => format_complex(Complex::new(c.re, -c.im), 'i'),
    }
}

// ── IMARGUMENT ────────────────────────────────────────────────────────────────

/// `IMARGUMENT(complex)` — argument (angle) of complex number.
pub fn imargument_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 1, 1) {
        return err;
    }
    match value_to_complex(args[0].clone()) {
        Err(e) => e,
        Ok(c) => {
            if c.re == 0.0 && c.im == 0.0 {
                Value::Error(ErrorKind::DivByZero)
            } else {
                Value::Number(c.arg())
            }
        }
    }
}

// ── IMLN ─────────────────────────────────────────────────────────────────────

/// `IMLN(complex)` — natural log of complex number.
pub fn imln_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 1, 1) {
        return err;
    }
    match value_to_complex(args[0].clone()) {
        Err(e) => e,
        Ok(c) => match c.ln() {
            None => Value::Error(ErrorKind::DivByZero),
            Some(result) => format_complex(result, 'i'),
        },
    }
}

// ── IMLOG10 / IMLOG2 / IMLOG ─────────────────────────────────────────────────

/// `IMLOG10(complex)` — base-10 log of complex number.
pub fn imlog10_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 1, 1) {
        return err;
    }
    match value_to_complex(args[0].clone()) {
        Err(e) => e,
        Ok(c) => match c.ln() {
            None => Value::Error(ErrorKind::DivByZero),
            Some(result) => {
                let ln10 = 10.0f64.ln();
                format_complex(Complex::new(result.re / ln10, result.im / ln10), 'i')
            }
        },
    }
}

/// `IMLOG2(complex)` — base-2 log of complex number.
pub fn imlog2_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 1, 1) {
        return err;
    }
    match value_to_complex(args[0].clone()) {
        Err(e) => e,
        Ok(c) => match c.ln() {
            None => Value::Error(ErrorKind::DivByZero),
            Some(result) => {
                let ln2 = 2.0f64.ln();
                format_complex(Complex::new(result.re / ln2, result.im / ln2), 'i')
            }
        },
    }
}

/// `IMLOG(complex, base)` — general log of complex number.
pub fn imlog_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 2, 2) {
        return err;
    }
    let c = match value_to_complex(args[0].clone()) {
        Err(e) => return e,
        Ok(v) => v,
    };
    let base = match to_number(args[1].clone()) {
        Err(e) => return e,
        Ok(v) => v,
    };
    if base <= 0.0 || base == 1.0 {
        return Value::Error(ErrorKind::Num);
    }
    match c.ln() {
        None => Value::Error(ErrorKind::DivByZero),
        Some(result) => {
            let ln_base = base.ln();
            format_complex(Complex::new(result.re / ln_base, result.im / ln_base), 'i')
        }
    }
}

// ── IMEXP ────────────────────────────────────────────────────────────────────

/// `IMEXP(complex)` — e raised to a complex power.
pub fn imexp_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 1, 1) {
        return err;
    }
    match value_to_complex(args[0].clone()) {
        Err(e) => e,
        Ok(c) => {
            let scale = c.re.exp();
            format_complex(Complex::new(scale * c.im.cos(), scale * c.im.sin()), 'i')
        }
    }
}

// ── IMPOWER ──────────────────────────────────────────────────────────────────

/// `IMPOWER(complex, number)` — complex number raised to a power.
pub fn impower_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 2, 2) {
        return err;
    }
    let base = match value_to_complex(args[0].clone()) {
        Err(e) => return e,
        Ok(c) => c,
    };
    let exp = match value_to_complex(args[1].clone()) {
        Err(e) => return e,
        Ok(c) => c,
    };
    match base.pow(exp) {
        None => Value::Error(ErrorKind::Num),
        Some(result) => format_complex(result, 'i'),
    }
}

// ── IMSQRT ───────────────────────────────────────────────────────────────────

/// `IMSQRT(complex)` — principal square root of complex number.
pub fn imsqrt_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 1, 1) {
        return err;
    }
    match value_to_complex(args[0].clone()) {
        Err(e) => e,
        Ok(c) => format_complex(c.sqrt(), 'i'),
    }
}

// ── Trig functions ────────────────────────────────────────────────────────────

/// `IMSIN(complex)` — sine of complex number.
pub fn imsin_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 1, 1) {
        return err;
    }
    match value_to_complex(args[0].clone()) {
        Err(e) => e,
        Ok(c) => {
            let re = c.re.sin() * c.im.cosh();
            let im = c.re.cos() * c.im.sinh();
            format_complex(Complex::new(re, im), 'i')
        }
    }
}

/// `IMCOS(complex)` — cosine of complex number.
pub fn imcos_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 1, 1) {
        return err;
    }
    match value_to_complex(args[0].clone()) {
        Err(e) => e,
        Ok(c) => {
            let re = c.re.cos() * c.im.cosh();
            let im = -(c.re.sin() * c.im.sinh());
            format_complex(Complex::new(re, im), 'i')
        }
    }
}

/// `IMTAN(complex)` — tangent of complex number.
pub fn imtan_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 1, 1) {
        return err;
    }
    match value_to_complex(args[0].clone()) {
        Err(e) => e,
        Ok(c) => {
            let sin_re = c.re.sin() * c.im.cosh();
            let sin_im = c.re.cos() * c.im.sinh();
            let cos_re = c.re.cos() * c.im.cosh();
            let cos_im = -(c.re.sin() * c.im.sinh());
            let denom = cos_re * cos_re + cos_im * cos_im;
            if denom == 0.0 {
                return Value::Error(ErrorKind::DivByZero);
            }
            let re = (sin_re * cos_re + sin_im * cos_im) / denom;
            let im = (sin_im * cos_re - sin_re * cos_im) / denom;
            format_complex(Complex::new(re, im), 'i')
        }
    }
}

/// `IMCOT(complex)` — cotangent of complex number.
pub fn imcot_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 1, 1) {
        return err;
    }
    match value_to_complex(args[0].clone()) {
        Err(e) => e,
        Ok(c) => {
            let sin_re = c.re.sin() * c.im.cosh();
            let sin_im = c.re.cos() * c.im.sinh();
            let cos_re = c.re.cos() * c.im.cosh();
            let cos_im = -(c.re.sin() * c.im.sinh());
            let denom = sin_re * sin_re + sin_im * sin_im;
            if denom == 0.0 {
                return Value::Error(ErrorKind::DivByZero);
            }
            let re = (cos_re * sin_re + cos_im * sin_im) / denom;
            let im = (cos_im * sin_re - cos_re * sin_im) / denom;
            format_complex(Complex::new(re, im), 'i')
        }
    }
}

/// `IMCSC(complex)` — cosecant of complex number.
pub fn imcsc_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 1, 1) {
        return err;
    }
    match value_to_complex(args[0].clone()) {
        Err(e) => e,
        Ok(c) => {
            let sin_re = c.re.sin() * c.im.cosh();
            let sin_im = c.re.cos() * c.im.sinh();
            let denom = sin_re * sin_re + sin_im * sin_im;
            if denom == 0.0 {
                return Value::Error(ErrorKind::Num);
            }
            let re = sin_re / denom;
            let im = -sin_im / denom;
            format_complex(Complex::new(re, im), 'i')
        }
    }
}

/// `IMSEC(complex)` — secant of complex number.
pub fn imsec_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 1, 1) {
        return err;
    }
    match value_to_complex(args[0].clone()) {
        Err(e) => e,
        Ok(c) => {
            let cos_re = c.re.cos() * c.im.cosh();
            let cos_im = -(c.re.sin() * c.im.sinh());
            let denom = cos_re * cos_re + cos_im * cos_im;
            if denom == 0.0 {
                return Value::Error(ErrorKind::DivByZero);
            }
            let re = cos_re / denom;
            let im = -cos_im / denom;
            format_complex(Complex::new(re, im), 'i')
        }
    }
}

// ── Hyperbolic trig ────────────────────────────────────────────────────────────

/// `IMSINH(complex)` — hyperbolic sine of complex number.
pub fn imsinh_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 1, 1) {
        return err;
    }
    match value_to_complex(args[0].clone()) {
        Err(e) => e,
        Ok(c) => {
            let re = c.re.sinh() * c.im.cos();
            let im = c.re.cosh() * c.im.sin();
            format_complex(Complex::new(re, im), 'i')
        }
    }
}

/// `IMCOSH(complex)` — hyperbolic cosine of complex number.
pub fn imcosh_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 1, 1) {
        return err;
    }
    match value_to_complex(args[0].clone()) {
        Err(e) => e,
        Ok(c) => {
            let re = c.re.cosh() * c.im.cos();
            let im = c.re.sinh() * c.im.sin();
            format_complex(Complex::new(re, im), 'i')
        }
    }
}

/// `IMTANH(complex)` — hyperbolic tangent of complex number.
pub fn imtanh_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 1, 1) {
        return err;
    }
    match value_to_complex(args[0].clone()) {
        Err(e) => e,
        Ok(c) => {
            let sinh_re = c.re.sinh() * c.im.cos();
            let sinh_im = c.re.cosh() * c.im.sin();
            let cosh_re = c.re.cosh() * c.im.cos();
            let cosh_im = c.re.sinh() * c.im.sin();
            let denom = cosh_re * cosh_re + cosh_im * cosh_im;
            if denom == 0.0 {
                return Value::Error(ErrorKind::DivByZero);
            }
            let re = (sinh_re * cosh_re + sinh_im * cosh_im) / denom;
            let im = (sinh_im * cosh_re - sinh_re * cosh_im) / denom;
            format_complex(Complex::new(re, im), 'i')
        }
    }
}

/// `IMCOTH(complex)` — hyperbolic cotangent of complex number.
pub fn imcoth_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 1, 1) {
        return err;
    }
    match value_to_complex(args[0].clone()) {
        Err(e) => e,
        Ok(c) => {
            let sinh_re = c.re.sinh() * c.im.cos();
            let sinh_im = c.re.cosh() * c.im.sin();
            let cosh_re = c.re.cosh() * c.im.cos();
            let cosh_im = c.re.sinh() * c.im.sin();
            let denom = sinh_re * sinh_re + sinh_im * sinh_im;
            if denom == 0.0 {
                return Value::Error(ErrorKind::DivByZero);
            }
            let re = (cosh_re * sinh_re + cosh_im * sinh_im) / denom;
            let im = (cosh_im * sinh_re - cosh_re * sinh_im) / denom;
            format_complex(Complex::new(re, im), 'i')
        }
    }
}

/// `IMCSCH(complex)` — hyperbolic cosecant of complex number.
pub fn imcsch_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 1, 1) {
        return err;
    }
    match value_to_complex(args[0].clone()) {
        Err(e) => e,
        Ok(c) => {
            let sinh_re = c.re.sinh() * c.im.cos();
            let sinh_im = c.re.cosh() * c.im.sin();
            let denom = sinh_re * sinh_re + sinh_im * sinh_im;
            if denom == 0.0 {
                return Value::Error(ErrorKind::DivByZero);
            }
            let re = sinh_re / denom;
            let im = -sinh_im / denom;
            format_complex(Complex::new(re, im), 'i')
        }
    }
}

/// `IMSECH(complex)` — hyperbolic secant of complex number.
pub fn imsech_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 1, 1) {
        return err;
    }
    match value_to_complex(args[0].clone()) {
        Err(e) => e,
        Ok(c) => {
            let cosh_re = c.re.cosh() * c.im.cos();
            let cosh_im = c.re.sinh() * c.im.sin();
            let denom = cosh_re * cosh_re + cosh_im * cosh_im;
            if denom == 0.0 {
                return Value::Error(ErrorKind::DivByZero);
            }
            let re = cosh_re / denom;
            let im = -cosh_im / denom;
            format_complex(Complex::new(re, im), 'i')
        }
    }
}

#[cfg(test)]
mod tests;
