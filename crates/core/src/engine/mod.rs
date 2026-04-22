use std::collections::HashMap;

use crate::eval::{evaluate_expr, Context, EvalCtx};
use crate::eval::functions::Registry;
use crate::parser::parse;
use crate::types::{ErrorKind, Value};

pub struct Engine {
    registry: Registry,
}

impl Engine {
    pub fn google_sheets() -> Self {
        Self { registry: Registry::new() }
    }

    pub fn evaluate(&self, formula: &str, variables: &HashMap<String, Value>) -> Value {
        match parse(formula) {
            Err(_) => Value::Error(ErrorKind::Value),
            Ok(expr) => {
                let ctx = Context::new(variables.clone());
                let mut eval_ctx = EvalCtx::new(ctx, &self.registry);
                first_of_array(evaluate_expr(&expr, &mut eval_ctx))
            }
        }
    }
}

fn first_of_array(v: Value) -> Value {
    match v {
        Value::Array(elems) if !elems.is_empty() => {
            first_of_array(elems.into_iter().next().unwrap())
        }
        other => other,
    }
}

#[cfg(test)]
mod tests;
