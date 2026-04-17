use std::collections::HashMap;
use crate::types::Value;

/// Holds the named variable bindings for a formula evaluation.
///
/// All keys are stored and looked up in uppercase, so variable names are
/// case-insensitive (`A1`, `a1`, and `A1` all refer to the same binding).
pub struct Context {
    pub vars: HashMap<String, Value>,
}

impl Context {
    /// Create a `Context` from a map of variable name → value.
    /// Keys are normalised to uppercase on insertion.
    pub fn new(vars: HashMap<String, Value>) -> Self {
        let normalized = vars.into_iter()
            .map(|(k, v)| (k.to_uppercase(), v))
            .collect();
        Self { vars: normalized }
    }

    /// Create an empty `Context` with no variable bindings.
    pub fn empty() -> Self {
        Self { vars: HashMap::new() }
    }

    /// Look up a variable by name (case-insensitive). Returns `Value::Empty` if not found.
    pub fn get(&self, name: &str) -> Value {
        self.vars
            .get(&name.to_uppercase())
            .cloned()
            .unwrap_or(Value::Empty)
    }

    /// Set a variable binding (case-insensitive key).
    pub fn set(&mut self, name: String, value: Value) {
        self.vars.insert(name.to_uppercase(), value);
    }
}

#[cfg(test)]
mod tests;
