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
        Some(Value::Text(s)) if s.len() == 7 => {
            let b = s.as_bytes();
            Ok([
                b[0] == b'1', b[1] == b'1', b[2] == b'1',
                b[3] == b'1', b[4] == b'1', b[5] == b'1', b[6] == b'1',
            ])
        }
        _ => Err(Value::Error(ErrorKind::Value)),
    }
}
