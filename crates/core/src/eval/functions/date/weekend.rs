use crate::types::{ErrorKind, Value};

/// Returns a [Mon, Tue, Wed, Thu, Fri, Sat, Sun] boolean mask where `true` = weekend day.
///
/// `weekend` may be:
/// - `None` / `Value::Number(1)` → Sat+Sun (default)
/// - `Value::Number(1..=17)` → predefined pattern
/// - `Value::Text(s)` where `s.len() == 7` → bitmask string (Mon=pos0, '1'=weekend)
pub fn weekend_mask(weekend: Option<&Value>) -> Result<[bool; 7], Value> {
    match weekend {
        None => Ok([false, false, false, false, false, true, true]),
        Some(Value::Number(n)) => {
            let code = *n as u32;
            let mask = match code {
                1  => [false, false, false, false, false, true,  true ],
                2  => [true,  false, false, false, false, false, true ],
                3  => [true,  true,  false, false, false, false, false],
                4  => [false, true,  true,  false, false, false, false],
                5  => [false, false, true,  true,  false, false, false],
                6  => [false, false, false, true,  true,  false, false],
                7  => [false, false, false, false, true,  true,  false],
                11 => [false, false, false, false, false, false, true ],
                12 => [true,  false, false, false, false, false, false],
                13 => [false, true,  false, false, false, false, false],
                14 => [false, false, true,  false, false, false, false],
                15 => [false, false, false, true,  false, false, false],
                16 => [false, false, false, false, true,  false, false],
                17 => [false, false, false, false, false, true,  false],
                _  => return Err(Value::Error(ErrorKind::Value)),
            };
            Ok(mask)
        }
        Some(Value::Text(s)) => {
            // Wrong length → #NUM!
            if s.len() != 7 {
                return Err(Value::Error(ErrorKind::Num));
            }
            let b = s.as_bytes();
            // Invalid characters (not '0' or '1') → #NUM!
            if b.iter().any(|&c| c != b'0' && c != b'1') {
                return Err(Value::Error(ErrorKind::Num));
            }
            let mask = [
                b[0] == b'1', b[1] == b'1', b[2] == b'1',
                b[3] == b'1', b[4] == b'1', b[5] == b'1', b[6] == b'1',
            ];
            // All-ones → every day is a weekend → #NUM!
            if mask.iter().all(|&x| x) {
                return Err(Value::Error(ErrorKind::Num));
            }
            Ok(mask)
        }
        _ => Err(Value::Error(ErrorKind::Value)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{ErrorKind, Value};

    #[test]
    fn code_1_sat_sun() {
        let mask = weekend_mask(Some(&Value::Number(1.0))).unwrap();
        assert_eq!(mask, [false, false, false, false, false, true, true]);
    }

    #[test]
    fn code_2_sun_mon() {
        let mask = weekend_mask(Some(&Value::Number(2.0))).unwrap();
        assert_eq!(mask, [true, false, false, false, false, false, true]);
    }

    #[test]
    fn code_3_mon_tue() {
        let mask = weekend_mask(Some(&Value::Number(3.0))).unwrap();
        assert_eq!(mask, [true, true, false, false, false, false, false]);
    }

    #[test]
    fn code_4_tue_wed() {
        let mask = weekend_mask(Some(&Value::Number(4.0))).unwrap();
        assert_eq!(mask, [false, true, true, false, false, false, false]);
    }

    #[test]
    fn code_5_wed_thu() {
        let mask = weekend_mask(Some(&Value::Number(5.0))).unwrap();
        assert_eq!(mask, [false, false, true, true, false, false, false]);
    }

    #[test]
    fn code_6_thu_fri() {
        let mask = weekend_mask(Some(&Value::Number(6.0))).unwrap();
        assert_eq!(mask, [false, false, false, true, true, false, false]);
    }

    #[test]
    fn code_7_fri_sat() {
        let mask = weekend_mask(Some(&Value::Number(7.0))).unwrap();
        assert_eq!(mask, [false, false, false, false, true, true, false]);
    }

    #[test]
    fn code_11_sun_only() {
        let mask = weekend_mask(Some(&Value::Number(11.0))).unwrap();
        assert_eq!(mask, [false, false, false, false, false, false, true]);
    }

    #[test]
    fn code_12_mon_only() {
        let mask = weekend_mask(Some(&Value::Number(12.0))).unwrap();
        assert_eq!(mask, [true, false, false, false, false, false, false]);
    }

    #[test]
    fn code_13_tue_only() {
        let mask = weekend_mask(Some(&Value::Number(13.0))).unwrap();
        assert_eq!(mask, [false, true, false, false, false, false, false]);
    }

    #[test]
    fn code_14_wed_only() {
        let mask = weekend_mask(Some(&Value::Number(14.0))).unwrap();
        assert_eq!(mask, [false, false, true, false, false, false, false]);
    }

    #[test]
    fn code_15_thu_only() {
        let mask = weekend_mask(Some(&Value::Number(15.0))).unwrap();
        assert_eq!(mask, [false, false, false, true, false, false, false]);
    }

    #[test]
    fn code_16_fri_only() {
        let mask = weekend_mask(Some(&Value::Number(16.0))).unwrap();
        assert_eq!(mask, [false, false, false, false, true, false, false]);
    }

    #[test]
    fn code_17_sat_only() {
        let mask = weekend_mask(Some(&Value::Number(17.0))).unwrap();
        assert_eq!(mask, [false, false, false, false, false, true, false]);
    }

    #[test]
    fn none_defaults_to_sat_sun() {
        let mask = weekend_mask(None).unwrap();
        assert_eq!(mask, [false, false, false, false, false, true, true]);
    }

    #[test]
    fn string_bitmask_all_zeros_no_weekend() {
        let mask = weekend_mask(Some(&Value::Text("0000000".into()))).unwrap();
        assert_eq!(mask, [false; 7]);
    }

    #[test]
    fn string_bitmask_all_ones_returns_num_error() {
        let result = weekend_mask(Some(&Value::Text("1111111".into())));
        assert_eq!(result, Err(Value::Error(ErrorKind::Num)));
    }

    #[test]
    fn string_bitmask_selective() {
        let mask = weekend_mask(Some(&Value::Text("1010100".into()))).unwrap();
        assert_eq!(mask, [true, false, true, false, true, false, false]);
    }

    #[test]
    fn invalid_code_returns_value_error() {
        let result = weekend_mask(Some(&Value::Number(8.0)));
        assert_eq!(result, Err(Value::Error(ErrorKind::Value)));
    }

    #[test]
    fn code_10_invalid_returns_value_error() {
        let result = weekend_mask(Some(&Value::Number(10.0)));
        assert_eq!(result, Err(Value::Error(ErrorKind::Value)));
    }

    #[test]
    fn wrong_string_length_returns_num_error() {
        let result = weekend_mask(Some(&Value::Text("011".into())));
        assert_eq!(result, Err(Value::Error(ErrorKind::Num)));
    }
}
