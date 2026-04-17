use super::super::*;
use crate::types::Value;

const EPS: f64 = 1e-4;

fn approx_eq(a: f64, b: f64) -> bool {
    (a - b).abs() < EPS
}

fn unwrap_num(v: Value) -> f64 {
    match v {
        Value::Number(n) => n,
        other => panic!("expected Number, got {other:?}"),
    }
}

fn num(n: f64) -> Value { Value::Number(n) }
fn bool_val(b: bool) -> Value { Value::Bool(b) }
fn nums_array(ns: &[f64]) -> Value {
    Value::Array(ns.iter().map(|&n| Value::Number(n)).collect())
}

#[test]
fn norm_s_dist_cdf_at_zero_is_half() {
    let v = unwrap_num(norm_s_dist_fn(&[num(0.0), bool_val(true)]));
    assert!(approx_eq(v, 0.5), "expected ~0.5, got {v}");
}

#[test]
fn norm_s_dist_pdf_at_zero() {
    let v = unwrap_num(norm_s_dist_fn(&[num(0.0), bool_val(false)]));
    assert!(approx_eq(v, 0.3989), "expected ~0.3989, got {v}");
}

#[test]
fn norm_dist_standard_cdf_at_zero() {
    let v = unwrap_num(norm_dist_fn(&[num(0.0), num(0.0), num(1.0), bool_val(true)]));
    assert!(approx_eq(v, 0.5), "expected ~0.5, got {v}");
}

#[test]
fn norm_dist_cdf_at_one_sigma() {
    let v = unwrap_num(norm_dist_fn(&[num(1.0), num(0.0), num(1.0), bool_val(true)]));
    assert!(approx_eq(v, 0.8413), "expected ~0.8413, got {v}");
}

#[test]
fn norm_s_inv_at_half_is_zero() {
    let v = unwrap_num(norm_s_inv_fn(&[num(0.5)]));
    assert!(approx_eq(v, 0.0), "expected ~0.0, got {v}");
}

#[test]
fn norm_s_inv_at_0975_is_1_96() {
    let v = unwrap_num(norm_s_inv_fn(&[num(0.975)]));
    assert!(approx_eq(v, 1.96), "expected ~1.96, got {v}");
}

#[test]
fn standardize_basic() {
    assert_eq!(standardize_fn(&[num(2.0), num(1.0), num(1.0)]), Value::Number(1.0));
}

#[test]
fn standardize_x_equals_mean_is_zero() {
    assert_eq!(standardize_fn(&[num(5.0), num(5.0), num(2.0)]), Value::Number(0.0));
}

#[test]
fn correl_perfect_positive() {
    let v = unwrap_num(correl_fn(&[nums_array(&[1.0, 2.0, 3.0]), nums_array(&[1.0, 2.0, 3.0])]));
    assert!(approx_eq(v, 1.0), "expected ~1.0, got {v}");
}

#[test]
fn pearson_perfect_positive() {
    let v = unwrap_num(pearson_fn(&[nums_array(&[1.0, 2.0, 3.0]), nums_array(&[1.0, 2.0, 3.0])]));
    assert!(approx_eq(v, 1.0), "expected ~1.0, got {v}");
}

#[test]
fn poisson_pmf_at_zero_lambda_1() {
    let v = unwrap_num(poisson_fn(&[num(0.0), num(1.0), bool_val(false)]));
    assert!(approx_eq(v, 0.3679), "expected ~0.3679, got {v}");
}

#[test]
fn poisson_cdf_at_zero_lambda_1() {
    let v = unwrap_num(poisson_fn(&[num(0.0), num(1.0), bool_val(true)]));
    assert!(approx_eq(v, 0.3679), "expected ~0.3679, got {v}");
}

#[test]
fn norm_inv_at_half_is_mean() {
    let v = unwrap_num(norm_inv_fn(&[num(0.5), num(3.0), num(1.0)]));
    assert!(approx_eq(v, 3.0), "expected ~3.0, got {v}");
}

#[test]
fn expon_dist_cdf_at_zero_is_zero() {
    assert_eq!(expon_dist_fn(&[num(0.0), num(1.0), bool_val(true)]), Value::Number(0.0));
}

#[test]
fn expon_dist_pdf_at_zero_equals_lambda() {
    assert_eq!(expon_dist_fn(&[num(0.0), num(2.0), bool_val(false)]), Value::Number(2.0));
}

#[test]
fn binom_dist_pmf_certain_success() {
    assert_eq!(binom_dist_fn(&[num(1.0), num(1.0), num(1.0), bool_val(false)]), Value::Number(1.0));
}

#[test]
fn binom_dist_cdf_all_successes() {
    assert_eq!(binom_dist_fn(&[num(3.0), num(3.0), num(0.5), bool_val(true)]), Value::Number(1.0));
}

#[test]
fn fisher_at_zero_is_zero() {
    assert_eq!(fisher_fn(&[num(0.0)]), Value::Number(0.0));
}

#[test]
fn fisher_inv_at_zero_is_zero() {
    assert_eq!(fisher_inv_fn(&[num(0.0)]), Value::Number(0.0));
}

#[test]
fn gauss_at_zero_is_zero() {
    let v = unwrap_num(gauss_fn(&[num(0.0)]));
    assert!(approx_eq(v, 0.0), "expected ~0.0, got {v}");
}

#[test]
fn covar_identical_arrays() {
    let v = unwrap_num(covar_fn(&[nums_array(&[1.0, 2.0, 3.0]), nums_array(&[1.0, 2.0, 3.0])]));
    assert!(approx_eq(v, 2.0 / 3.0), "expected ~0.6667, got {v}");
}
