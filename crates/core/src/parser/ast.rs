use super::refs::Ref;
use crate::types::ErrorKind;

/// Byte range of a node within the original formula string.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Span {
    pub offset: usize, // byte offset from start of formula
    pub length: usize,
}

impl Span {
    pub fn new(offset: usize, length: usize) -> Self {
        Self { offset, length }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum UnaryOp {
    Neg,     // -x
    Percent, // x% → x/100
}

#[derive(Debug, Clone, PartialEq)]
pub enum BinaryOp {
    Add, Sub, Mul, Div, Pow,
    Concat,         // &
    Eq, Ne, Lt, Gt, Le, Ge,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Number(f64, Span),
    Text(String, Span),
    Bool(bool, Span),
    /// An error literal typed directly into a formula (`=#REF!`,
    /// `=#REF!+1`) — parses straight to its error value, the same way
    /// `Number`/`Text`/`Bool` parse straight to theirs. Distinct from an
    /// error *produced* by evaluation (e.g. `=1/0`), which never round-trips
    /// through this variant.
    Error(ErrorKind, Span),
    Variable(String, Span),
    /// Sheet-qualified reference: `Sheet1!A1`, `'Q2 Data'!A1:B2`.
    /// Bare cell/range references (`A1`, `A1:D4`) and bare names remain
    /// [`Expr::Variable`]; sheet-qualified forms always carry `sheet: Some(_)`.
    Reference(Ref, Span),
    UnaryOp {
        op: UnaryOp,
        operand: Box<Expr>,
        span: Span,
    },
    BinaryOp {
        op: BinaryOp,
        left: Box<Expr>,
        right: Box<Expr>,
        span: Span,
    },
    FunctionCall {
        name: String,   // always uppercased
        args: Vec<Expr>,
        span: Span,
    },
    Array(Vec<Expr>, Span),
    /// Immediately-invoked function application: `expr(call_args)`.
    /// Used for LAMBDA: `LAMBDA(x, x*2)(5)` → `Apply { func: LAMBDA(...), call_args: [5] }`.
    Apply {
        func: Box<Expr>,
        call_args: Vec<Expr>,
        span: Span,
    },
}

impl Expr {
    pub fn span(&self) -> &Span {
        match self {
            Expr::Number(_, s) | Expr::Text(_, s) | Expr::Bool(_, s) | Expr::Error(_, s) | Expr::Variable(_, s) | Expr::Reference(_, s) => s,
            Expr::UnaryOp { span, .. }
            | Expr::BinaryOp { span, .. }
            | Expr::FunctionCall { span, .. }
            | Expr::Apply { span, .. } => span,
            Expr::Array(_, span) => span,
        }
    }
}

#[cfg(test)]
mod tests;
