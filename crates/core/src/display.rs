/// Format an `f64` for display in a spreadsheet cell.
///
/// Uses up to 14 significant digits and strips trailing zeros.
/// Returns `"#NUM!"` for `NaN` or infinite values.
///
/// # Examples
/// ```
/// # use truecalc_core::display::display_number;
/// assert_eq!(display_number(1.0), "1");
/// assert_eq!(display_number(0.1 + 0.2), "0.3");
/// assert_eq!(display_number(f64::NAN), "#NUM!");
/// ```
pub fn display_number(n: f64) -> String {
    if n.is_nan() || n.is_infinite() {
        return "#NUM!".to_string();
    }
    // Normalize -0.0 to 0.0 (Excel displays both as "0")
    let n = if n == 0.0 { 0.0 } else { n };
    let abs = n.abs();
    let decimals: usize = if abs == 0.0 {
        14
    } else {
        let magnitude = abs.log10().floor() as i32;
        (14 - magnitude).max(0) as usize
    };
    let s = format!("{:.decimals$}", n, decimals = decimals);
    if s.contains('.') {
        s.trim_end_matches('0').trim_end_matches('.').to_string()
    } else {
        s
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn float_display() {
        assert_eq!(display_number(0.1 + 0.2), "0.3");
        assert_eq!(display_number(1.0), "1");
        assert_eq!(display_number(1234567890.12345), "1234567890.12345");
        assert_eq!(display_number(0.0), "0");
        assert_eq!(display_number(-3.5), "-3.5");
        assert_eq!(display_number(f64::NAN), "#NUM!");
        assert_eq!(display_number(f64::INFINITY), "#NUM!");
        assert_eq!(display_number(-0.0_f64), "0");
    }
}
