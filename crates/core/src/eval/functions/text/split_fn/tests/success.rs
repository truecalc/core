use super::super::split_fn;
use crate::types::Value;

#[test]
fn basic_split() {
    let result = split_fn(&[Value::Text("a,b,c".into()), Value::Text(",".into())]);
    assert_eq!(
        result,
        Value::Array(vec![
            Value::Text("a".into()),
            Value::Text("b".into()),
            Value::Text("c".into()),
        ])
    );
}
