use super::Registry;
use super::FunctionMeta;

pub mod bitand;
pub mod bitor;
pub mod bitxor;
pub mod bitlshift;
pub mod bitrshift;
pub mod delta;
pub mod gestep;
pub mod complex;

pub mod bin2dec;
pub mod bin2hex;
pub mod bin2oct;
pub mod dec2bin;
pub mod dec2hex;
pub mod dec2oct;
pub mod hex2bin;
pub mod hex2dec;
pub mod hex2oct;
pub mod oct2bin;
pub mod oct2dec;
pub mod oct2hex;

/// Parse a binary string (up to 10 chars of 0/1) as 10-bit two's complement → i64.
/// Returns None if invalid chars or length > 10.
/// Empty string is treated as "0" (Google Sheets behaviour).
pub(crate) fn parse_bin(s: &str) -> Option<i64> {
    if s.is_empty() {
        return Some(0);
    }
    if s.len() > 10 {
        return None;
    }
    for c in s.chars() {
        if c != '0' && c != '1' {
            return None;
        }
    }
    let bits = u64::from_str_radix(s, 2).ok()?;
    // 10-bit two's complement: bit 9 is sign bit
    if bits & 0b10_0000_0000 != 0 {
        // negative: extend sign
        Some(bits as i64 - 1024)
    } else {
        Some(bits as i64)
    }
}

/// Parse an octal string (up to 10 chars of 0–7) as 29-bit two's complement → i64.
/// Returns None if invalid chars or length > 10.
/// Empty string is treated as "0" (Google Sheets behaviour).
pub(crate) fn parse_oct(s: &str) -> Option<i64> {
    if s.is_empty() {
        return Some(0);
    }
    if s.len() > 10 {
        return None;
    }
    for c in s.chars() {
        if !('0'..='7').contains(&c) {
            return None;
        }
    }
    let bits = u64::from_str_radix(s, 8).ok()?;
    // 29-bit two's complement: bit 29 sign (value 2^29 = 536870912)
    if bits & 0x2000_0000 != 0 {
        Some(bits as i64 - 0x4000_0000) // subtract 2^30
    } else {
        Some(bits as i64)
    }
}

/// Parse a hex string (up to 10 chars, case-insensitive) as 40-bit two's complement → i64.
/// Returns None if invalid chars or length > 10.
/// Empty string is treated as "0" (Google Sheets behaviour).
pub(crate) fn parse_hex(s: &str) -> Option<i64> {
    if s.is_empty() {
        return Some(0);
    }
    if s.len() > 10 {
        return None;
    }
    for c in s.chars() {
        if !c.is_ascii_hexdigit() {
            return None;
        }
    }
    let bits = u64::from_str_radix(s, 16).ok()?;
    // 40-bit two's complement: bit 39 is sign (value 2^39 = 549755813888)
    if bits & 0x80_0000_0000 != 0 {
        Some(bits as i64 - 0x100_0000_0000i64) // subtract 2^40
    } else {
        Some(bits as i64)
    }
}

/// Format i64 as binary string using 10-bit two's complement for negatives.
/// Returns Err(()) if places is invalid or result won't fit.
pub(crate) fn format_bin(n: i64, places: Option<usize>) -> Result<String, ()> {
    let bits: u64 = if n < 0 {
        // 10-bit two's complement
        (n + 1024) as u64
    } else {
        n as u64
    };
    let s = format!("{:b}", bits);
    apply_places(s, places, 10)
}

/// Format i64 as octal string using 10-digit two's complement for negatives.
/// Returns Err(()) if places is invalid or result won't fit.
pub(crate) fn format_oct(n: i64, places: Option<usize>) -> Result<String, ()> {
    let bits: u64 = if n < 0 {
        // 29-bit two's complement stored in 30 bits
        (n + 0x4000_0000) as u64
    } else {
        n as u64
    };
    let s = format!("{:o}", bits);
    apply_places(s, places, 10)
}

/// Format i64 as uppercase hex string using 10-digit two's complement for negatives.
/// Returns Err(()) if places is invalid or result won't fit.
pub(crate) fn format_hex(n: i64, places: Option<usize>) -> Result<String, ()> {
    let bits: u64 = if n < 0 {
        // 40-bit two's complement
        (n + 0x100_0000_0000i64) as u64
    } else {
        n as u64
    };
    let s = format!("{:X}", bits);
    apply_places(s, places, 10)
}

fn apply_places(s: String, places: Option<usize>, max_len: usize) -> Result<String, ()> {
    match places {
        None => Ok(s),
        Some(p) => {
            if p == 0 || s.len() > p || p > max_len {
                Err(())
            } else {
                Ok(format!("{:0>width$}", s, width = p))
            }
        }
    }
}

/// Extract optional places argument (2nd arg) from args slice.
/// Returns Ok(None) if not provided, Ok(Some(n)) if valid positive integer, Err(Value) on error.
pub(crate) fn get_places(args: &[crate::types::Value]) -> Result<Option<usize>, crate::types::Value> {
    use crate::eval::coercion::to_number;
    use crate::types::{ErrorKind, Value};
    if args.len() < 2 {
        return Ok(None);
    }
    let n = to_number(args[1].clone())?;
    let p = n.trunc() as i64;
    if p <= 0 {
        return Err(Value::Error(ErrorKind::Num));
    }
    Ok(Some(p as usize))
}

pub fn register_engineering(registry: &mut Registry) {
    registry.register_eager("BITAND",    bitand::bitand_fn,       FunctionMeta { category: "engineering", signature: "BITAND(number1, number2)",          description: "Bitwise AND of two integers" });
    registry.register_eager("BITOR",     bitor::bitor_fn,         FunctionMeta { category: "engineering", signature: "BITOR(number1, number2)",           description: "Bitwise OR of two integers" });
    registry.register_eager("BITXOR",    bitxor::bitxor_fn,       FunctionMeta { category: "engineering", signature: "BITXOR(number1, number2)",          description: "Bitwise XOR of two integers" });
    registry.register_eager("BITLSHIFT", bitlshift::bitlshift_fn, FunctionMeta { category: "engineering", signature: "BITLSHIFT(number, shift_amount)",   description: "Left-shift an integer by a number of bits" });
    registry.register_eager("BITRSHIFT", bitrshift::bitrshift_fn, FunctionMeta { category: "engineering", signature: "BITRSHIFT(number, shift_amount)",   description: "Right-shift an integer by a number of bits" });
    registry.register_eager("DELTA",     delta::delta_fn,         FunctionMeta { category: "engineering", signature: "DELTA(number1, [number2])",         description: "Test whether two values are equal" });
    registry.register_eager("GESTEP",    gestep::gestep_fn,       FunctionMeta { category: "engineering", signature: "GESTEP(number, [step])",            description: "Test whether a number is greater than or equal to a step value" });
    registry.register_eager("BIN2DEC", bin2dec::bin2dec_fn, FunctionMeta { category: "engineering", signature: "BIN2DEC(number)",           description: "Convert binary to decimal" });
    registry.register_eager("BIN2HEX", bin2hex::bin2hex_fn, FunctionMeta { category: "engineering", signature: "BIN2HEX(number, [places])", description: "Convert binary to hexadecimal" });
    registry.register_eager("BIN2OCT", bin2oct::bin2oct_fn, FunctionMeta { category: "engineering", signature: "BIN2OCT(number, [places])", description: "Convert binary to octal" });
    registry.register_eager("DEC2BIN", dec2bin::dec2bin_fn, FunctionMeta { category: "engineering", signature: "DEC2BIN(number, [places])", description: "Convert decimal to binary" });
    registry.register_eager("DEC2HEX", dec2hex::dec2hex_fn, FunctionMeta { category: "engineering", signature: "DEC2HEX(number, [places])", description: "Convert decimal to hexadecimal" });
    registry.register_eager("DEC2OCT", dec2oct::dec2oct_fn, FunctionMeta { category: "engineering", signature: "DEC2OCT(number, [places])", description: "Convert decimal to octal" });
    registry.register_eager("HEX2BIN", hex2bin::hex2bin_fn, FunctionMeta { category: "engineering", signature: "HEX2BIN(number, [places])", description: "Convert hexadecimal to binary" });
    registry.register_eager("HEX2DEC", hex2dec::hex2dec_fn, FunctionMeta { category: "engineering", signature: "HEX2DEC(number)",           description: "Convert hexadecimal to decimal" });
    registry.register_eager("HEX2OCT", hex2oct::hex2oct_fn, FunctionMeta { category: "engineering", signature: "HEX2OCT(number, [places])", description: "Convert hexadecimal to octal" });
    registry.register_eager("OCT2BIN", oct2bin::oct2bin_fn, FunctionMeta { category: "engineering", signature: "OCT2BIN(number, [places])", description: "Convert octal to binary" });
    registry.register_eager("OCT2DEC", oct2dec::oct2dec_fn, FunctionMeta { category: "engineering", signature: "OCT2DEC(number)",           description: "Convert octal to decimal" });
    registry.register_eager("OCT2HEX", oct2hex::oct2hex_fn, FunctionMeta { category: "engineering", signature: "OCT2HEX(number, [places])", description: "Convert octal to hexadecimal" });

    // Complex number functions
    registry.register_eager("COMPLEX",      complex::complex_fn,      FunctionMeta { category: "engineering", signature: "COMPLEX(real, imaginary, [suffix])", description: "Create a complex number string" });
    registry.register_eager("IMREAL",       complex::imreal_fn,       FunctionMeta { category: "engineering", signature: "IMREAL(complex)",                   description: "Real part of a complex number" });
    registry.register_eager("IMAGINARY",    complex::imaginary_fn,    FunctionMeta { category: "engineering", signature: "IMAGINARY(complex)",                description: "Imaginary part of a complex number" });
    registry.register_eager("IMABS",        complex::imabs_fn,        FunctionMeta { category: "engineering", signature: "IMABS(complex)",                    description: "Absolute value of a complex number" });
    registry.register_eager("IMPRODUCT",    complex::improduct_fn,    FunctionMeta { category: "engineering", signature: "IMPRODUCT(complex1, ...)",          description: "Product of complex numbers" });
    registry.register_eager("IMSUB",        complex::imsub_fn,        FunctionMeta { category: "engineering", signature: "IMSUB(complex1, complex2)",         description: "Subtract complex numbers" });
    registry.register_eager("IMSUM",        complex::imsum_fn,        FunctionMeta { category: "engineering", signature: "IMSUM(complex1, ...)",              description: "Sum of complex numbers" });
    registry.register_eager("IMDIV",        complex::imdiv_fn,        FunctionMeta { category: "engineering", signature: "IMDIV(complex1, complex2)",         description: "Divide complex numbers" });
    registry.register_eager("IMCONJUGATE",  complex::imconjugate_fn,  FunctionMeta { category: "engineering", signature: "IMCONJUGATE(complex)",             description: "Complex conjugate" });
    registry.register_eager("IMARGUMENT",   complex::imargument_fn,   FunctionMeta { category: "engineering", signature: "IMARGUMENT(complex)",              description: "Argument (angle) of a complex number" });
    registry.register_eager("IMLN",         complex::imln_fn,         FunctionMeta { category: "engineering", signature: "IMLN(complex)",                    description: "Natural log of a complex number" });
    registry.register_eager("IMLOG10",      complex::imlog10_fn,      FunctionMeta { category: "engineering", signature: "IMLOG10(complex)",                 description: "Base-10 log of a complex number" });
    registry.register_eager("IMLOG2",       complex::imlog2_fn,       FunctionMeta { category: "engineering", signature: "IMLOG2(complex)",                  description: "Base-2 log of a complex number" });
    registry.register_eager("IMLOG",        complex::imlog_fn,        FunctionMeta { category: "engineering", signature: "IMLOG(complex, base)",             description: "Logarithm of a complex number to a given base" });
    registry.register_eager("IMEXP",        complex::imexp_fn,        FunctionMeta { category: "engineering", signature: "IMEXP(complex)",                   description: "e raised to a complex power" });
    registry.register_eager("IMPOWER",      complex::impower_fn,      FunctionMeta { category: "engineering", signature: "IMPOWER(complex, number)",         description: "Complex number raised to a power" });
    registry.register_eager("IMSQRT",       complex::imsqrt_fn,       FunctionMeta { category: "engineering", signature: "IMSQRT(complex)",                  description: "Square root of a complex number" });
    registry.register_eager("IMSIN",        complex::imsin_fn,        FunctionMeta { category: "engineering", signature: "IMSIN(complex)",                   description: "Sine of a complex number" });
    registry.register_eager("IMCOS",        complex::imcos_fn,        FunctionMeta { category: "engineering", signature: "IMCOS(complex)",                   description: "Cosine of a complex number" });
    registry.register_eager("IMTAN",        complex::imtan_fn,        FunctionMeta { category: "engineering", signature: "IMTAN(complex)",                   description: "Tangent of a complex number" });
    registry.register_eager("IMCOT",        complex::imcot_fn,        FunctionMeta { category: "engineering", signature: "IMCOT(complex)",                   description: "Cotangent of a complex number" });
    registry.register_eager("IMCSC",        complex::imcsc_fn,        FunctionMeta { category: "engineering", signature: "IMCSC(complex)",                   description: "Cosecant of a complex number" });
    registry.register_eager("IMSEC",        complex::imsec_fn,        FunctionMeta { category: "engineering", signature: "IMSEC(complex)",                   description: "Secant of a complex number" });
    registry.register_eager("IMSINH",       complex::imsinh_fn,       FunctionMeta { category: "engineering", signature: "IMSINH(complex)",                  description: "Hyperbolic sine of a complex number" });
    registry.register_eager("IMCOSH",       complex::imcosh_fn,       FunctionMeta { category: "engineering", signature: "IMCOSH(complex)",                  description: "Hyperbolic cosine of a complex number" });
    registry.register_eager("IMTANH",       complex::imtanh_fn,       FunctionMeta { category: "engineering", signature: "IMTANH(complex)",                  description: "Hyperbolic tangent of a complex number" });
    registry.register_eager("IMCOTH",       complex::imcoth_fn,       FunctionMeta { category: "engineering", signature: "IMCOTH(complex)",                  description: "Hyperbolic cotangent of a complex number" });
    registry.register_eager("IMCSCH",       complex::imcsch_fn,       FunctionMeta { category: "engineering", signature: "IMCSCH(complex)",                  description: "Hyperbolic cosecant of a complex number" });
    registry.register_eager("IMSECH",       complex::imsech_fn,       FunctionMeta { category: "engineering", signature: "IMSECH(complex)",                  description: "Hyperbolic secant of a complex number" });

    // Error functions
    registry.register_eager("ERF",          erf::erf_fn,          FunctionMeta { category: "engineering", signature: "ERF(lower_limit, [upper_limit])", description: "Error function" });
    registry.register_eager("ERF.PRECISE",  erf::erf_precise_fn,  FunctionMeta { category: "engineering", signature: "ERF.PRECISE(x)",                 description: "Error function (precise)" });
    registry.register_eager("ERFC",         erf::erfc_fn,         FunctionMeta { category: "engineering", signature: "ERFC(x)",                        description: "Complementary error function" });
    registry.register_eager("ERFC.PRECISE", erf::erfc_precise_fn, FunctionMeta { category: "engineering", signature: "ERFC.PRECISE(x)",               description: "Complementary error function (precise)" });
}

pub mod erf;
