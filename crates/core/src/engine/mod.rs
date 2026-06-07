use std::collections::HashMap;

use crate::eval::functions::Registry;
use crate::eval::{evaluate_expr, Context, EvalCtx};
use crate::parser::{parse_formula, Expr};
use crate::types::{ErrorKind, ParseError, Value};

/// Which spreadsheet product's semantics the engine targets.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Flavor {
    Sheets,
    Excel,
}

pub struct Engine {
    flavor: Flavor,
    registry: Registry,
}

impl Engine {
    /// Engine targeting Google Sheets conformance.
    pub fn sheets() -> Self {
        Self { flavor: Flavor::Sheets, registry: Registry::new() }
    }

    /// Engine targeting Excel conformance.
    ///
    /// Excel evaluation semantics are not implemented yet (they land in a
    /// later phase): [`Engine::evaluate`] returns
    /// `Value::Error(ErrorKind::NA)` for every formula. [`Engine::parse`]
    /// and [`Engine::validate`] work.
    pub fn excel() -> Self {
        Self { flavor: Flavor::Excel, registry: Registry::new() }
    }

    /// Deprecated alias for [`Engine::sheets`].
    #[deprecated(note = "use Engine::sheets() — engine flavor is required; see ADR 2026-04-27")]
    pub fn google_sheets() -> Self {
        Self::sheets()
    }

    /// Parse a formula string into an expression tree.
    ///
    /// The formula may start with `=`. Returns a [`ParseError`] if the input
    /// is not a valid formula.
    pub fn parse(&self, formula: &str) -> Result<Expr, ParseError> {
        parse_formula(formula)
    }

    /// Validate that a formula string is syntactically correct without
    /// returning the AST.
    pub fn validate(&self, formula: &str) -> Result<(), ParseError> {
        self.parse(formula).map(|_| ())
    }

    pub fn evaluate(&self, formula: &str, variables: &HashMap<String, Value>) -> Value {
        if self.flavor == Flavor::Excel {
            // Excel evaluation semantics are not implemented yet.
            return Value::Error(ErrorKind::NA);
        }
        match parse_formula(formula) {
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
