use std::collections::HashMap;
use truecalc_core::{evaluate, Value};

fn run(formula: &str) -> Value {
    evaluate(formula, &HashMap::new())
}

fn as_number(v: &Value) -> Option<f64> {
    match v {
        Value::Number(n) => Some(*n),
        Value::Date(n) => Some(*n),
        _ => None,
    }
}

/// Flatten a 2-D Value::Array into a flat Vec<f64>, returning None if any cell
/// is not a number.
fn flatten_numbers(v: &Value) -> Option<Vec<f64>> {
    match v {
        Value::Array(rows) => {
            let mut out = Vec::new();
            for row in rows {
                match row {
                    Value::Array(cells) => {
                        for cell in cells {
                            out.push(as_number(cell)?);
                        }
                    }
                    other => out.push(as_number(other)?),
                }
            }
            Some(out)
        }
        other => as_number(other).map(|n| vec![n]),
    }
}

// 1. RAND returns a value in [0, 1)
#[test]
fn rand_returns_value_in_0_1() {
    for _ in 0..100 {
        let v = run("=RAND()");
        let n = as_number(&v).expect("RAND() should return a number");
        assert!(n >= 0.0 && n < 1.0, "RAND() out of range: {}", n);
    }
}

// 2. Two successive RAND() calls differ
#[test]
fn rand_successive_calls_differ() {
    let a = run("=RAND()");
    let b = run("=RAND()");
    let na = as_number(&a).expect("first RAND() should be a number");
    let nb = as_number(&b).expect("second RAND() should be a number");
    assert_ne!(na, nb, "two RAND() calls returned the same value: {}", na);
}

// 3. RANDARRAY cells are independent — no two cells share the same value
//    (BUG-06 regression guard)
#[test]
fn randarray_cells_are_independent() {
    let v = run("=RANDARRAY(3,3)");
    if let Some(nums) = flatten_numbers(&v) {
        // Check all pairs for equality
        for i in 0..nums.len() {
            for j in (i + 1)..nums.len() {
                assert_ne!(
                    nums[i], nums[j],
                    "RANDARRAY(3,3): cells[{}]={} and cells[{}]={} are equal (BUG-06)",
                    i, nums[i], j, nums[j]
                );
            }
        }
    }
    // If result isn't an array of numbers, skip — function may not be implemented
}

// 4. RANDARRAY(2,3) returns a 2×3 array (6 values total)
#[test]
fn randarray_shape_is_correct() {
    let v = run("=RANDARRAY(2,3)");
    if let Value::Array(rows) = &v {
        assert_eq!(rows.len(), 2, "RANDARRAY(2,3) should have 2 rows, got {}", rows.len());
        for (i, row) in rows.iter().enumerate() {
            match row {
                Value::Array(cells) => assert_eq!(
                    cells.len(), 3,
                    "RANDARRAY(2,3) row {} should have 3 cells, got {}",
                    i, cells.len()
                ),
                _ => panic!("RANDARRAY(2,3) row {} is not an array: {:?}", i, row),
            }
        }
    } else {
        panic!("RANDARRAY(2,3) did not return an array: {:?}", v);
    }
}

// 5. RANDBETWEEN(1,10) returns an integer in [1, 10] on each of 100 calls
#[test]
fn randbetween_returns_integer_in_range() {
    for _ in 0..100 {
        let v = run("=RANDBETWEEN(1,10)");
        let n = as_number(&v).expect("RANDBETWEEN(1,10) should return a number");
        assert!(
            n >= 1.0 && n <= 10.0,
            "RANDBETWEEN(1,10) out of range: {}",
            n
        );
        assert_eq!(n, n.floor(), "RANDBETWEEN(1,10) is not an integer: {}", n);
    }
}

// 6. TODAY() returns a serial date > 45000 (Jan 1 2023) and < 100000
#[test]
fn today_returns_reasonable_date_serial() {
    let v = run("=TODAY()");
    let n = as_number(&v).expect("TODAY() should return a number or date");
    assert!(
        n > 45000.0,
        "TODAY() serial {} is before Jan 1 2023 (45000)",
        n
    );
    assert!(n < 100000.0, "TODAY() serial {} is unreasonably large", n);
}

// 7. NOW() >= TODAY() and NOW() < TODAY() + 1
#[test]
fn now_is_greater_than_today() {
    let now_val = run("=NOW()");
    let today_val = run("=TODAY()");
    let now = as_number(&now_val).expect("NOW() should return a number or date");
    let today = as_number(&today_val).expect("TODAY() should return a number or date");
    assert!(
        now >= today,
        "NOW()={} should be >= TODAY()={}",
        now, today
    );
    assert!(
        now < today + 1.0,
        "NOW()={} should be < TODAY()+1={}",
        now,
        today + 1.0
    );
}
