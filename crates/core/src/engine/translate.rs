//! `translate_formula`: shift a formula's relative cell/range references by
//! (d_row, d_col) — the fill/copy-paste reference-adjustment transform.
//! `$`-absolute axes are left unchanged. See
//! `docs/superpowers/specs/2026-07-14-translate-formula-design.md`.

use crate::eval::functions::lookup::indirect::{MAX_COL, MAX_ROW};
use crate::parser::ast::{Expr, Span};
use crate::parser::refs::write_sheet;
use crate::parser::{CellAddr, Ref};
use crate::types::ErrorKind;

/// Shift `addr` by `(d_row, d_col)`, skipping any axis marked `$`-absolute.
/// Returns `None` if a *relative* axis lands outside the Sheets grid
/// (`1..=MAX_COL` / `1..=MAX_ROW`) — the caller renders that as `#REF!`.
pub(crate) fn shift_addr(addr: CellAddr, d_row: i64, d_col: i64) -> Option<CellAddr> {
    let col = if addr.col_abs { addr.col as i64 } else { addr.col as i64 + d_col };
    let row = if addr.row_abs { addr.row as i64 } else { addr.row as i64 + d_row };
    if (1..=MAX_COL as i64).contains(&col) && (1..=MAX_ROW as i64).contains(&row) {
        Some(CellAddr::new(col as u32, row as u32).with_col_abs(addr.col_abs).with_row_abs(addr.row_abs))
    } else {
        None
    }
}

fn addr_text(addr: CellAddr, d_row: i64, d_col: i64) -> String {
    match shift_addr(addr, d_row, d_col) {
        Some(shifted) => shifted.to_string(),
        None => ErrorKind::Ref.to_string(),
    }
}

/// Render `r` shifted by `(d_row, d_col)` back to formula text. A corner
/// that shifts out of the Sheets grid becomes literal `#REF!`; the other
/// corner of a range (if in bounds) keeps its shifted address.
// TODO(#709): remove once translate_text (Task 4) calls this.
#[allow(dead_code)]
pub(crate) fn shift_ref_text(r: &Ref, d_row: i64, d_col: i64) -> String {
    match r {
        Ref::Cell { sheet, addr } => {
            let mut out = String::new();
            write_sheet(&mut out, sheet).expect("String::write_str is infallible");
            out.push_str(&addr_text(*addr, d_row, d_col));
            out
        }
        Ref::Range { sheet, start, end } => {
            let mut out = String::new();
            write_sheet(&mut out, sheet).expect("String::write_str is infallible");
            out.push_str(&addr_text(*start, d_row, d_col));
            out.push(':');
            out.push_str(&addr_text(*end, d_row, d_col));
            out
        }
        Ref::Name(name) => name.clone(), // never reached by the Task 3 traversal
    }
}

fn normalize_name(name: &str) -> String {
    name.to_uppercase().replace('$', "")
}

/// Collect every reference in `expr` that should be shifted: sheet-qualified
/// `Expr::Reference` nodes, and bare `Expr::Variable` nodes that classify as
/// `Ref::Cell`/`Ref::Range` and are not shadowed by an enclosing `LET`/
/// `LAMBDA` binding of the same (normalized) name.
// TODO(#709): remove once translate_text (Task 4) calls this.
#[allow(dead_code)]
pub(crate) fn collect_shiftable_refs(expr: &Expr) -> Vec<(Span, Ref)> {
    let mut out = Vec::new();
    let mut scope: Vec<String> = Vec::new();
    walk(expr, &mut scope, &mut out);
    out
}

// TODO(#709): remove once translate_text (Task 4) calls this.
#[allow(dead_code)]
fn walk(expr: &Expr, scope: &mut Vec<String>, out: &mut Vec<(Span, Ref)>) {
    match expr {
        Expr::Number(..) | Expr::Text(..) | Expr::Bool(..) => {}
        Expr::Reference(r, span) => out.push((span.clone(), r.clone())),
        Expr::Variable(name, span) => {
            if !scope.contains(&normalize_name(name)) {
                match Ref::classify(name) {
                    Ref::Name(_) => {}
                    r => out.push((span.clone(), r)),
                }
            }
        }
        Expr::UnaryOp { operand, .. } => walk(operand, scope, out),
        Expr::BinaryOp { left, right, .. } => {
            walk(left, scope, out);
            walk(right, scope, out);
        }
        Expr::FunctionCall { name, args, .. }
            if name == "LET" && args.len() >= 3 && args.len() % 2 == 1 =>
        {
            let pair_count = (args.len() - 1) / 2;
            let mut bound = 0;
            for i in 0..pair_count {
                // Value expr sees only bindings 0..i-1 (matches let_fn.rs:
                // evaluate_expr runs before ctx.set for this pair).
                walk(&args[i * 2 + 1], scope, out);
                match &args[i * 2] {
                    Expr::Variable(n, _) => {
                        scope.push(normalize_name(n));
                        bound += 1;
                    }
                    other => walk(other, scope, out), // malformed name slot
                }
            }
            walk(&args[args.len() - 1], scope, out); // body sees all bindings
            for _ in 0..bound {
                scope.pop();
            }
        }
        Expr::FunctionCall { name, args, .. } if name == "LAMBDA" && !args.is_empty() => {
            let param_count = args.len() - 1;
            let mut bound = 0;
            for param_expr in &args[..param_count] {
                match param_expr {
                    Expr::Variable(n, _) => {
                        scope.push(normalize_name(n));
                        bound += 1;
                    }
                    other => walk(other, scope, out), // malformed param slot
                }
            }
            walk(&args[args.len() - 1], scope, out); // body
            for _ in 0..bound {
                scope.pop();
            }
        }
        Expr::FunctionCall { args, .. } => {
            for arg in args {
                walk(arg, scope, out);
            }
        }
        Expr::Array(elems, _) => {
            for elem in elems {
                walk(elem, scope, out);
            }
        }
        Expr::Apply { func, call_args, .. } => {
            // call_args are evaluated in the caller's (outer) scope, before
            // func's own LAMBDA params are pushed.
            for arg in call_args {
                walk(arg, scope, out);
            }
            walk(func, scope, out);
        }
    }
}

#[cfg(test)]
mod tests;
