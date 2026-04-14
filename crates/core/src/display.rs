/// Format an f64 the way Excel does: up to 15 significant digits, no trailing zeros.
pub fn display_number(n: f64) -> String {
    if n.is_nan() || n.is_infinite() {
        return "#NUM!".to_string();
    }
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
    }
}
