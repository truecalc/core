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
fn nums_array(ns: &[f64]) -> Value {
    Value::Array(ns.iter().map(|&n| Value::Number(n)).collect())
}

#[test]
fn correl_perfect_negative() {
    // Reversed array → correlation = -1
    let v = unwrap_num(correl_fn(&[nums_array(&[1.0, 2.0, 3.0]), nums_array(&[3.0, 2.0, 1.0])]));
    assert!(approx_eq(v, -1.0), "expected ~-1.0, got {v}");
}

#[test]
fn fisher_roundtrip() {
    // fisher_inv(fisher(0.5)) ≈ 0.5
    let z = unwrap_num(fisher_fn(&[num(0.5)]));
    let x = unwrap_num(fisher_inv_fn(&[num(z)]));
    assert!(approx_eq(x, 0.5), "expected ~0.5, got {x}");
}
