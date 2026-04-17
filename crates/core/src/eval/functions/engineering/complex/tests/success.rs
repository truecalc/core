use super::super::{
    complex_fn, imabs_fn, imaginary_fn, imreal_fn, improduct_fn, imsqrt_fn,
};
use crate::types::Value;

fn text(s: &str) -> Value {
    Value::Text(s.to_string())
}

// ---------------------------------------------------------------------------
// COMPLEX
// ---------------------------------------------------------------------------

#[test]
fn complex_real_pos_imag() {
    // COMPLEX(3, 4) → "3+4i"
    assert_eq!(
        complex_fn(&[Value::Number(3.0), Value::Number(4.0)]),
        text("3+4i")
    );
}

#[test]
fn complex_real_neg_imag() {
    // COMPLEX(3, -4) → "3-4i"
    assert_eq!(
        complex_fn(&[Value::Number(3.0), Value::Number(-4.0)]),
        text("3-4i")
    );
}

#[test]
fn complex_zero_real_unit_imag() {
    // COMPLEX(0, 1) → "i"
    assert_eq!(
        complex_fn(&[Value::Number(0.0), Value::Number(1.0)]),
        text("i")
    );
}

#[test]
fn complex_pure_real() {
    // COMPLEX(5, 0) → 5 (returns Number, no imaginary part)
    assert_eq!(
        complex_fn(&[Value::Number(5.0), Value::Number(0.0)]),
        Value::Number(5.0)
    );
}

#[test]
fn complex_j_suffix() {
    // COMPLEX(3, 4, "j") → "3+4j"
    assert_eq!(
        complex_fn(&[Value::Number(3.0), Value::Number(4.0), text("j")]),
        text("3+4j")
    );
}

// ---------------------------------------------------------------------------
// IMABS
// ---------------------------------------------------------------------------

#[test]
fn imabs_3_4i() {
    // |3+4i| = 5
    assert_eq!(imabs_fn(&[text("3+4i")]), Value::Number(5.0));
}

#[test]
fn imabs_zero() {
    // |0| = 0
    assert_eq!(imabs_fn(&[text("0")]), Value::Number(0.0));
}

// ---------------------------------------------------------------------------
// IMREAL / IMAGINARY
// ---------------------------------------------------------------------------

#[test]
fn imreal_3_4i() {
    assert_eq!(imreal_fn(&[text("3+4i")]), Value::Number(3.0));
}

#[test]
fn imaginary_3_4i() {
    assert_eq!(imaginary_fn(&[text("3+4i")]), Value::Number(4.0));
}

#[test]
fn imreal_pure_real() {
    // "5" has no imaginary part; real part is 5
    assert_eq!(imreal_fn(&[text("5")]), Value::Number(5.0));
}

#[test]
fn imaginary_pure_imag() {
    // "3i" has no real part; imaginary part is 3
    assert_eq!(imaginary_fn(&[text("3i")]), Value::Number(3.0));
}

// ---------------------------------------------------------------------------
// IMPRODUCT
// ---------------------------------------------------------------------------

#[test]
fn improduct_rotation_90() {
    // (1+0i) * (0+1i) = 0+1i → "i"
    assert_eq!(
        improduct_fn(&[text("1+0i"), text("0+1i")]),
        text("i")
    );
}

#[test]
fn improduct_1_2i_times_3_4i() {
    // (1+2i)(3+4i) = (3-8)+(4+6)i = -5+10i
    let result = improduct_fn(&[text("1+2i"), text("3+4i")]);
    assert_eq!(result, text("-5+10i"));
}

// ---------------------------------------------------------------------------
// IMSQRT
// ---------------------------------------------------------------------------

#[test]
fn imsqrt_neg_one() {
    // sqrt(-1) = i
    // The result should have re ≈ 0 and im ≈ 1; format_complex emits "i"
    assert_eq!(imsqrt_fn(&[text("-1")]), text("i"));
}
