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
            Expr::Number(_, s) | Expr::Text(_, s) | Expr::Bool(_, s) | Expr::Variable(_, s) => s,
            Expr::UnaryOp { span, .. }
            | Expr::BinaryOp { span, .. }
            | Expr::FunctionCall { span, .. }
            | Expr::Apply { span, .. } => span,
            Expr::Array(_, span) => span,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn span_stores_offset_and_length() {
        let s = Span::new(5, 10);
        assert_eq!(s.offset, 5);
        assert_eq!(s.length, 10);
    }

    #[test]
    fn expr_number_span() {
        let e = Expr::Number(1.0, Span::new(0, 3));
        assert_eq!(e.span().offset, 0);
        assert_eq!(e.span().length, 3);
    }

    #[test]
    fn expr_text_span() {
        let e = Expr::Text("hello".into(), Span::new(2, 7));
        assert_eq!(e.span().offset, 2);
    }

    #[test]
    fn expr_bool_span() {
        let e = Expr::Bool(true, Span::new(1, 4));
        assert_eq!(e.span().offset, 1);
    }

    #[test]
    fn expr_function_call_span() {
        let e = Expr::FunctionCall {
            name: "SUM".into(),
            args: vec![],
            span: Span::new(0, 5),
        };
        assert_eq!(e.span().offset, 0);
        assert_eq!(e.span().length, 5);
    }

    #[test]
    fn unary_op_debug() {
        assert_eq!(format!("{:?}", UnaryOp::Neg), "Neg");
        assert_eq!(format!("{:?}", UnaryOp::Percent), "Percent");
    }

    #[test]
    fn binary_op_debug() {
        assert_eq!(format!("{:?}", BinaryOp::Add), "Add");
        assert_eq!(format!("{:?}", BinaryOp::Eq), "Eq");
    }

    #[test]
    fn expr_variable_span() {
        let e = Expr::Variable("x".into(), Span::new(0, 1));
        assert_eq!(e.span().offset, 0);
        assert_eq!(e.span().length, 1);
    }

    #[test]
    fn expr_unary_op_span() {
        let e = Expr::UnaryOp {
            op: UnaryOp::Neg,
            operand: Box::new(Expr::Number(1.0, Span::new(1, 1))),
            span: Span::new(0, 2),
        };
        assert_eq!(e.span().offset, 0);
        assert_eq!(e.span().length, 2);
    }

    #[test]
    fn expr_binary_op_span() {
        let e = Expr::BinaryOp {
            op: BinaryOp::Add,
            left: Box::new(Expr::Number(1.0, Span::new(0, 1))),
            right: Box::new(Expr::Number(2.0, Span::new(2, 1))),
            span: Span::new(0, 3),
        };
        assert_eq!(e.span().offset, 0);
        assert_eq!(e.span().length, 3);
    }

    #[test]
    fn expr_apply_span() {
        let e = Expr::Apply {
            func: Box::new(Expr::Variable("f".into(), Span::new(0, 1))),
            call_args: vec![],
            span: Span::new(0, 4),
        };
        assert_eq!(e.span().offset, 0);
        assert_eq!(e.span().length, 4);
    }
}
