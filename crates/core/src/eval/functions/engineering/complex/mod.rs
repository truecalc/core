use crate::eval::coercion::{to_number, to_string_val};
use crate::eval::functions::check_arity;
use crate::types::{ErrorKind, Value};

// ---------------------------------------------------------------------------
// Complex number helpers
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug)]
pub(crate) struct Cpx {
    pub re: f64,
    pub im: f64,
}

impl Cpx {
    fn new(re: f64, im: f64) -> Self {
        Cpx { re, im }
    }

    fn abs(self) -> f64 {
        self.re.hypot(self.im)
    }

    fn arg(self) -> f64 {
        self.im.atan2(self.re)
    }

    fn conj(self) -> Self {
        Cpx::new(self.re, -self.im)
    }

    fn add(self, rhs: Cpx) -> Self {
        Cpx::new(self.re + rhs.re, self.im + rhs.im)
    }

    fn sub(self, rhs: Cpx) -> Self {
        Cpx::new(self.re - rhs.re, self.im - rhs.im)
    }

    fn mul(self, rhs: Cpx) -> Self {
        Cpx::new(
            self.re * rhs.re - self.im * rhs.im,
            self.re * rhs.im + self.im * rhs.re,
        )
    }

    fn div(self, rhs: Cpx) -> Option<Self> {
        let denom = rhs.re * rhs.re + rhs.im * rhs.im;
        if denom == 0.0 {
            return None;
        }
        Some(Cpx::new(
            (self.re * rhs.re + self.im * rhs.im) / denom,
            (self.im * rhs.re - self.re * rhs.im) / denom,
        ))
    }

    fn exp(self) -> Self {
        let e_re = self.re.exp();
        Cpx::new(e_re * self.im.cos(), e_re * self.im.sin())
    }

    fn ln(self) -> Self {
        Cpx::new(self.abs().ln(), self.arg())
    }

    fn log10(self) -> Self {
        let ln10 = 10.0_f64.ln();
        let l = self.ln();
        Cpx::new(l.re / ln10, l.im / ln10)
    }

    fn log2(self) -> Self {
        let ln2 = 2.0_f64.ln();
        let l = self.ln();
        Cpx::new(l.re / ln2, l.im / ln2)
    }

    fn log_base(self, base: Cpx) -> Option<Self> {
        let ln_base = base.ln();
        let ln_self = self.ln();
        ln_self.div(ln_base)
    }

    fn pow(self, exp: Cpx) -> Self {
        if self.re == 0.0 && self.im == 0.0 {
            if exp.re == 0.0 && exp.im == 0.0 {
                return Cpx::new(1.0, 0.0);
            }
            return Cpx::new(0.0, 0.0);
        }
        let l = self.ln();
        let product = l.mul(exp);
        product.exp()
    }

    fn sqrt(self) -> Self {
        let r = self.abs();
        if r == 0.0 {
            return Cpx::new(0.0, 0.0);
        }
        let re = ((r + self.re) / 2.0).sqrt();
        let im = ((r - self.re) / 2.0).sqrt();
        let im = if self.im < 0.0 { -im } else { im };
        Cpx::new(re, im)
    }

    fn sin(self) -> Self {
        Cpx::new(
            self.re.sin() * self.im.cosh(),
            self.re.cos() * self.im.sinh(),
        )
    }

    fn cos(self) -> Self {
        Cpx::new(
            self.re.cos() * self.im.cosh(),
            -(self.re.sin() * self.im.sinh()),
        )
    }

    fn tan(self) -> Option<Self> {
        self.sin().div(self.cos())
    }

    fn cot(self) -> Option<Self> {
        self.cos().div(self.sin())
    }

    fn sec(self) -> Option<Self> {
        Cpx::new(1.0, 0.0).div(self.cos())
    }

    fn csc(self) -> Option<Self> {
        Cpx::new(1.0, 0.0).div(self.sin())
    }

    fn sinh(self) -> Self {
        Cpx::new(
            self.re.sinh() * self.im.cos(),
            self.re.cosh() * self.im.sin(),
        )
    }

    fn cosh(self) -> Self {
        Cpx::new(
            self.re.cosh() * self.im.cos(),
            self.re.sinh() * self.im.sin(),
        )
    }

    fn tanh(self) -> Option<Self> {
        self.sinh().div(self.cosh())
    }

    fn coth(self) -> Option<Self> {
        self.cosh().div(self.sinh())
    }

    fn sech(self) -> Option<Self> {
        Cpx::new(1.0, 0.0).div(self.cosh())
    }

    fn csch(self) -> Option<Self> {
        Cpx::new(1.0, 0.0).div(self.sinh())
    }
}

// ---------------------------------------------------------------------------
// Parse a complex number string: "3+4i", "3-4i", "5i", "-i", "3", etc.
// Also accepts "j" suffix.
// ---------------------------------------------------------------------------
pub(crate) fn parse_complex(s: &str) -> Option<Cpx> {
    let s = s.trim();
    if s.is_empty() {
        return None;
    }

    let last = s.chars().last()?;
    if last == 'i' || last == 'j' {
        let body = &s[..s.len() - 1];
        let split_pos = find_split(body)?;
        if split_pos == 0 {
            let im = parse_imag_coeff(body)?;
            Some(Cpx::new(0.0, im))
        } else {
            let (re_part, im_part) = body.split_at(split_pos);
            let re = re_part.parse::<f64>().ok()?;
            let im = parse_imag_coeff(im_part)?;
            Some(Cpx::new(re, im))
        }
    } else {
        let re = s.parse::<f64>().ok()?;
        Some(Cpx::new(re, 0.0))
    }
}

fn find_split(s: &str) -> Option<usize> {
    let bytes = s.as_bytes();
    let mut i = bytes.len();
    while i > 0 {
        i -= 1;
        let c = bytes[i] as char;
        if (c == '+' || c == '-') && i > 0 {
            let prev = bytes[i - 1] as char;
            if prev != 'e' && prev != 'E' {
                return Some(i);
            }
        }
    }
    Some(0)
}

fn parse_imag_coeff(s: &str) -> Option<f64> {
    match s {
        "" | "+" => Some(1.0),
        "-" => Some(-1.0),
        _ => s.parse::<f64>().ok(),
    }
}

// ---------------------------------------------------------------------------
// Format a complex number as a Value
// ---------------------------------------------------------------------------
pub(crate) fn format_complex(z: Cpx, suffix: char) -> Value {
    let re = clean(z.re);
    let im = clean(z.im);

    if im == 0.0 {
        return Value::Number(re);
    }
    if re == 0.0 {
        let s = if im == 1.0 {
            format!("{}", suffix)
        } else if im == -1.0 {
            format!("-{}", suffix)
        } else {
            format!("{}{}", format_coeff(im), suffix)
        };
        return Value::Text(s);
    }

    let im_str = if im == 1.0 {
        format!("+{}", suffix)
    } else if im == -1.0 {
        format!("-{}", suffix)
    } else if im < 0.0 {
        format!("{}{}", format_coeff(im), suffix)
    } else {
        format!("+{}{}", format_coeff(im), suffix)
    };

    Value::Text(format!("{}{}", format_coeff(re), im_str))
}

fn clean(v: f64) -> f64 {
    if v.abs() < 1e-10 { 0.0 } else { v }
}

fn format_coeff(v: f64) -> String {
    if v == v.trunc() && v.abs() < 1e15 {
        format!("{}", v as i64)
    } else {
        format!("{}", v)
    }
}

fn get_complex_arg(args: &[Value], idx: usize) -> Result<Cpx, Value> {
    let s = to_string_val(args[idx].clone())?;
    parse_complex(&s).ok_or(Value::Error(ErrorKind::Value))
}

// ---------------------------------------------------------------------------
// COMPLEX(real, imaginary, [suffix])
// ---------------------------------------------------------------------------
pub fn complex_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 2, 3) { return err; }
    let re = match to_number(args[0].clone()) { Ok(n) => n, Err(e) => return e };
    let im = match to_number(args[1].clone()) { Ok(n) => n, Err(e) => return e };
    let suffix = if args.len() == 3 {
        let s = match to_string_val(args[2].clone()) { Ok(v) => v, Err(e) => return e };
        match s.as_str() {
            "i" => 'i',
            "j" => 'j',
            _ => return Value::Error(ErrorKind::Value),
        }
    } else { 'i' };
    format_complex(Cpx::new(re, im), suffix)
}

pub fn imabs_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 1, 1) { return err; }
    match get_complex_arg(args, 0) {
        Ok(z) => Value::Number(z.abs()),
        Err(e) => e,
    }
}

pub fn imaginary_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 1, 1) { return err; }
    match get_complex_arg(args, 0) {
        Ok(z) => Value::Number(z.im),
        Err(e) => e,
    }
}

pub fn imreal_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 1, 1) { return err; }
    match get_complex_arg(args, 0) {
        Ok(z) => Value::Number(z.re),
        Err(e) => e,
    }
}

pub fn imargument_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 1, 1) { return err; }
    match get_complex_arg(args, 0) {
        Ok(z) => {
            if z.re == 0.0 && z.im == 0.0 {
                Value::Error(ErrorKind::DivByZero)
            } else {
                Value::Number(z.arg())
            }
        }
        Err(e) => e,
    }
}

pub fn imconjugate_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 1, 1) { return err; }
    match get_complex_arg(args, 0) {
        Ok(z) => format_complex(z.conj(), 'i'),
        Err(e) => e,
    }
}

pub fn imdiv_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 2, 2) { return err; }
    let a = match get_complex_arg(args, 0) { Ok(z) => z, Err(e) => return e };
    let b = match get_complex_arg(args, 1) { Ok(z) => z, Err(e) => return e };
    match a.div(b) {
        Some(z) => format_complex(z, 'i'),
        None => Value::Error(ErrorKind::DivByZero),
    }
}

pub fn imexp_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 1, 1) { return err; }
    match get_complex_arg(args, 0) {
        Ok(z) => format_complex(z.exp(), 'i'),
        Err(e) => e,
    }
}

pub fn imln_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 1, 1) { return err; }
    match get_complex_arg(args, 0) {
        Ok(z) => {
            if z.re == 0.0 && z.im == 0.0 { return Value::Error(ErrorKind::Num); }
            format_complex(z.ln(), 'i')
        }
        Err(e) => e,
    }
}

pub fn imlog10_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 1, 1) { return err; }
    match get_complex_arg(args, 0) {
        Ok(z) => {
            if z.re == 0.0 && z.im == 0.0 { return Value::Error(ErrorKind::DivByZero); }
            format_complex(z.log10(), 'i')
        }
        Err(e) => e,
    }
}

pub fn imlog2_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 1, 1) { return err; }
    match get_complex_arg(args, 0) {
        Ok(z) => {
            if z.re == 0.0 && z.im == 0.0 { return Value::Error(ErrorKind::Num); }
            format_complex(z.log2(), 'i')
        }
        Err(e) => e,
    }
}

pub fn imlog_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 2, 2) { return err; }
    let z = match get_complex_arg(args, 0) { Ok(z) => z, Err(e) => return e };
    let b = match get_complex_arg(args, 1) { Ok(z) => z, Err(e) => return e };
    if z.re == 0.0 && z.im == 0.0 { return Value::Error(ErrorKind::Num); }
    match z.log_base(b) {
        Some(result) => format_complex(result, 'i'),
        None => Value::Error(ErrorKind::DivByZero),
    }
}

pub fn impower_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 2, 2) { return err; }
    let base = match get_complex_arg(args, 0) { Ok(z) => z, Err(e) => return e };
    let exp = match get_complex_arg(args, 1) { Ok(z) => z, Err(e) => return e };
    format_complex(base.pow(exp), 'i')
}

pub fn improduct_fn(args: &[Value]) -> Value {
    if args.is_empty() { return Value::Error(ErrorKind::NA); }
    let mut result = Cpx::new(1.0, 0.0);
    for arg in args {
        let s = match to_string_val(arg.clone()) { Ok(v) => v, Err(e) => return e };
        let z = match parse_complex(&s) { Some(z) => z, None => return Value::Error(ErrorKind::Value) };
        result = result.mul(z);
    }
    format_complex(result, 'i')
}

pub fn imsqrt_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 1, 1) { return err; }
    match get_complex_arg(args, 0) {
        Ok(z) => format_complex(z.sqrt(), 'i'),
        Err(e) => e,
    }
}

pub fn imsub_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 2, 2) { return err; }
    let a = match get_complex_arg(args, 0) { Ok(z) => z, Err(e) => return e };
    let b = match get_complex_arg(args, 1) { Ok(z) => z, Err(e) => return e };
    format_complex(a.sub(b), 'i')
}

pub fn imsum_fn(args: &[Value]) -> Value {
    if args.is_empty() { return Value::Error(ErrorKind::NA); }
    let mut result = Cpx::new(0.0, 0.0);
    for arg in args {
        let s = match to_string_val(arg.clone()) { Ok(v) => v, Err(e) => return e };
        let z = match parse_complex(&s) { Some(z) => z, None => return Value::Error(ErrorKind::Value) };
        result = result.add(z);
    }
    format_complex(result, 'i')
}

pub fn imsin_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 1, 1) { return err; }
    match get_complex_arg(args, 0) { Ok(z) => format_complex(z.sin(), 'i'), Err(e) => e }
}

pub fn imcos_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 1, 1) { return err; }
    match get_complex_arg(args, 0) { Ok(z) => format_complex(z.cos(), 'i'), Err(e) => e }
}

pub fn imtan_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 1, 1) { return err; }
    match get_complex_arg(args, 0) {
        Ok(z) => match z.tan() { Some(r) => format_complex(r, 'i'), None => Value::Error(ErrorKind::DivByZero) },
        Err(e) => e,
    }
}

pub fn imsec_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 1, 1) { return err; }
    match get_complex_arg(args, 0) {
        Ok(z) => match z.sec() { Some(r) => format_complex(r, 'i'), None => Value::Error(ErrorKind::DivByZero) },
        Err(e) => e,
    }
}

pub fn imcsc_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 1, 1) { return err; }
    match get_complex_arg(args, 0) {
        Ok(z) => match z.csc() { Some(r) => format_complex(r, 'i'), None => Value::Error(ErrorKind::Num) },
        Err(e) => e,
    }
}

pub fn imcot_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 1, 1) { return err; }
    match get_complex_arg(args, 0) {
        Ok(z) => match z.cot() { Some(r) => format_complex(r, 'i'), None => Value::Error(ErrorKind::DivByZero) },
        Err(e) => e,
    }
}

pub fn imsinh_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 1, 1) { return err; }
    match get_complex_arg(args, 0) { Ok(z) => format_complex(z.sinh(), 'i'), Err(e) => e }
}

pub fn imcosh_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 1, 1) { return err; }
    match get_complex_arg(args, 0) { Ok(z) => format_complex(z.cosh(), 'i'), Err(e) => e }
}

pub fn imtanh_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 1, 1) { return err; }
    match get_complex_arg(args, 0) {
        Ok(z) => match z.tanh() { Some(r) => format_complex(r, 'i'), None => Value::Error(ErrorKind::DivByZero) },
        Err(e) => e,
    }
}

pub fn imsech_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 1, 1) { return err; }
    match get_complex_arg(args, 0) {
        Ok(z) => match z.sech() { Some(r) => format_complex(r, 'i'), None => Value::Error(ErrorKind::DivByZero) },
        Err(e) => e,
    }
}

pub fn imcsch_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 1, 1) { return err; }
    match get_complex_arg(args, 0) {
        Ok(z) => match z.csch() { Some(r) => format_complex(r, 'i'), None => Value::Error(ErrorKind::DivByZero) },
        Err(e) => e,
    }
}

pub fn imcoth_fn(args: &[Value]) -> Value {
    if let Some(err) = check_arity(args, 1, 1) { return err; }
    match get_complex_arg(args, 0) {
        Ok(z) => match z.coth() { Some(r) => format_complex(r, 'i'), None => Value::Error(ErrorKind::DivByZero) },
        Err(e) => e,
    }
}

#[cfg(test)]
mod tests;
