use super::super::*;
use crate::types::{ErrorKind, Value};

fn num(n: f64) -> Value { Value::Number(n) }
fn bool_val(b: bool) -> Value { Value::Bool(b) }
fn nums_array(ns: &[f64]) -> Value {
    Value::Array(ns.iter().map(|&n| Value::Number(n)).collect())
}

#[test]
fn norm_s_dist_no_args_is_na() {
    assert_eq!(norm_s_dist_fn(&[]), Value::Error(ErrorKind::NA));
}

#[test]
fn norm_dist_stdev_zero_is_num_error() {
    assert_eq!(norm_dist_fn(&[num(0.0), num(0.0), num(0.0), bool_val(true)]), Value::Error(ErrorKind::Num));
}

#[test]
fn norm_dist_too_few_args_is_na() {
    assert_eq!(norm_dist_fn(&[num(0.0), num(0.0)]), Value::Error(ErrorKind::NA));
}

#[test]
fn norm_s_inv_out_of_range_is_num_error() {
    assert_eq!(norm_s_inv_fn(&[num(0.0)]), Value::Error(ErrorKind::Num));
    assert_eq!(norm_s_inv_fn(&[num(1.0)]), Value::Error(ErrorKind::Num));
}

#[test]
fn norm_s_inv_no_args_is_na() {
    assert_eq!(norm_s_inv_fn(&[]), Value::Error(ErrorKind::NA));
}

#[test]
fn standardize_stdev_zero_is_num_error() {
    assert_eq!(standardize_fn(&[num(2.0), num(1.0), num(0.0)]), Value::Error(ErrorKind::Num));
}

#[test]
fn standardize_too_few_args_is_na() {
    assert_eq!(standardize_fn(&[num(2.0)]), Value::Error(ErrorKind::NA));
}

#[test]
fn correl_too_few_args_is_na() {
    assert_eq!(correl_fn(&[nums_array(&[1.0, 2.0, 3.0])]), Value::Error(ErrorKind::NA));
}

#[test]
fn poisson_too_few_args_is_na() {
    assert_eq!(poisson_fn(&[num(0.0), num(1.0)]), Value::Error(ErrorKind::NA));
}

#[test]
fn norm_inv_stdev_zero_is_num_error() {
    assert_eq!(norm_inv_fn(&[num(0.5), num(0.0), num(0.0)]), Value::Error(ErrorKind::Num));
}

#[test]
fn expon_dist_lambda_zero_is_num_error() {
    assert_eq!(expon_dist_fn(&[num(1.0), num(0.0), bool_val(true)]), Value::Error(ErrorKind::Num));
}

#[test]
fn binom_dist_k_gt_n_is_num_error() {
    assert_eq!(binom_dist_fn(&[num(4.0), num(3.0), num(0.5), bool_val(false)]), Value::Error(ErrorKind::Num));
}

#[test]
fn fisher_out_of_range_is_num_error() {
    assert_eq!(fisher_fn(&[num(1.0)]), Value::Error(ErrorKind::Num));
    assert_eq!(fisher_fn(&[num(-1.0)]), Value::Error(ErrorKind::Num));
}

#[test]
fn covar_too_few_args_is_na() {
    assert_eq!(covar_fn(&[nums_array(&[1.0, 2.0])]), Value::Error(ErrorKind::NA));
}
