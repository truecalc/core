//! Reference resolution (P1.3, issue #525).
//!
//! Core is the *language*: it parses formulas and evaluates them, but it has
//! no notion of a sheet, a grid, or a named range. [`Resolver`] is the seam
//! through which an embedding application (the workbook crate, a host app, a
//! test harness) supplies the *value* a [`Ref`] points at.
//!
//! The evaluator calls [`Resolver::resolve`] every time a formula reads a
//! reference that is not already bound as a local variable (e.g. a LAMBDA
//! parameter). The resolver owns all workbook semantics:
//!
//! - a cross-sheet cell ref to a missing sheet -> [`ErrorKind::Ref`] (`#REF!`),
//! - a bare name that is not a defined name -> [`ErrorKind::Name`] (`#NAME?`),
//! - an empty cell -> [`Value::Empty`],
//! - a range -> [`Value::Array`] of the cells in reading order.
//!
//! Core never invents these; it simply forwards the [`Ref`] and returns
//! whatever the resolver decides.
//!
//! [`ErrorKind::Ref`]: crate::ErrorKind::Ref
//! [`ErrorKind::Name`]: crate::ErrorKind::Name
//!
//! ```
//! use std::collections::HashMap;
//! use truecalc_core::{Engine, Ref, Resolver, Value};
//!
//! /// A resolver backed by a flat map keyed by canonical reference text.
//! struct MapResolver(HashMap<String, Value>);
//!
//! impl Resolver for MapResolver {
//!     fn resolve(&mut self, r: &Ref) -> Value {
//!         self.0.get(&r.to_string()).cloned().unwrap_or(Value::Empty)
//!     }
//! }
//!
//! let mut cells = HashMap::new();
//! cells.insert("Data!A1".to_string(), Value::Number(10.0));
//! cells.insert("Data!B1".to_string(), Value::Number(5.0));
//! let mut resolver = MapResolver(cells);
//!
//! let engine = Engine::sheets();
//! let result = engine.evaluate_with_resolver("=Data!A1+Data!B1", &mut resolver);
//! assert_eq!(result, Value::Number(15.0));
//! ```

use crate::parser::ast::Expr;
use crate::parser::refs::Ref;
use crate::types::Value;

/// Resolves a [`Ref`] to the [`Value`] it points at.
///
/// Implementors own all workbook semantics: missing sheets, undefined names,
/// empty cells, and range materialization. See the module docs for the
/// expected error mapping (`#REF!` for missing sheets, `#NAME?` for undefined
/// names). The evaluator only calls this for references that are not shadowed
/// by a local binding such as a LAMBDA parameter.
pub trait Resolver {
    /// Return the value of `r`. The receiver is `&mut self` so resolvers may
    /// memoize, count reads, or lazily materialize ranges.
    fn resolve(&mut self, r: &Ref) -> Value;
}

/// Collect every [`Ref`] a formula reads, in left-to-right source order,
/// without re-parsing -- the input that lets the workbook build a dependency
/// graph (plan section 3.2).
///
/// Both reference shapes are reported as a [`Ref`]:
///
/// - sheet-qualified references ([`Expr::Reference`]) are returned verbatim;
/// - bare identifiers ([`Expr::Variable`]) -- which the parser keeps as
///   variable nodes for compatibility -- are classified with
///   [`Ref::classify`], so `A1` becomes [`Ref::Cell`], `A1:D4` becomes
///   [`Ref::Range`], and `TAX_RATE` becomes [`Ref::Name`].
///
/// Refs appearing in *any* argument position of a function call are found,
/// because the walk descends into every child expression. Duplicates are
/// preserved (a formula that reads `A1` twice yields two entries); callers
/// that want a dependency set deduplicate themselves.
///
/// ```
/// use truecalc_core::{extract_refs, CellAddr, Engine, Ref};
///
/// let expr = Engine::sheets().parse("=SUM(A1, Data!B2, TAX_RATE)").unwrap();
/// let refs = extract_refs(&expr);
/// assert_eq!(
///     refs,
///     vec![
///         Ref::Cell { sheet: None, addr: CellAddr { col: 1, row: 1 } },
///         Ref::Cell { sheet: Some("Data".to_string()), addr: CellAddr { col: 2, row: 2 } },
///         Ref::Name("TAX_RATE".to_string()),
///     ],
/// );
/// ```
pub fn extract_refs(expr: &Expr) -> Vec<Ref> {
    let mut refs = Vec::new();
    collect_refs(expr, &mut refs);
    refs
}

fn collect_refs(expr: &Expr, out: &mut Vec<Ref>) {
    match expr {
        Expr::Reference(r, _) => out.push(r.clone()),
        Expr::Variable(name, _) => out.push(Ref::classify(name)),
        Expr::Number(..) | Expr::Text(..) | Expr::Bool(..) => {}
        Expr::UnaryOp { operand, .. } => collect_refs(operand, out),
        Expr::BinaryOp { left, right, .. } => {
            collect_refs(left, out);
            collect_refs(right, out);
        }
        Expr::FunctionCall { args, .. } => {
            for arg in args {
                collect_refs(arg, out);
            }
        }
        Expr::Array(elems, _) => {
            for elem in elems {
                collect_refs(elem, out);
            }
        }
        Expr::Apply { func, call_args, .. } => {
            collect_refs(func, out);
            for arg in call_args {
                collect_refs(arg, out);
            }
        }
    }
}
