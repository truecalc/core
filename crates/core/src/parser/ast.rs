/// Byte range of a node within the original formula string.
#[derive(Debug, Clone, PartialEq)]
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
    Variable(String, Span),
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
}

impl Expr {
    pub fn span(&self) -> &Span {
        match self {
            Expr::Number(_, s) | Expr::Text(_, s) | Expr::Bool(_, s) | Expr::Variable(_, s) => s,
            Expr::UnaryOp { span, .. }
            | Expr::BinaryOp { span, .. }
            | Expr::FunctionCall { span, .. } => span,
        }
    }
}
