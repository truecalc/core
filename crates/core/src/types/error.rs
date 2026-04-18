use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum ErrorKind {
    DivByZero,
    Value,
    Ref,
    Name,
    Num,
    NA,
    Null,
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            ErrorKind::DivByZero => "#DIV/0!",
            ErrorKind::Value     => "#VALUE!",
            ErrorKind::Ref       => "#REF!",
            ErrorKind::Name      => "#NAME?",
            ErrorKind::Num       => "#NUM!",
            ErrorKind::NA        => "#N/A",
            ErrorKind::Null      => "#NULL!",
        };
        write!(f, "{}", s)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParseError {
    pub message: String,
    pub position: usize, // byte offset in original formula
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} at position {}", self.message, self.position)
    }
}

impl std::error::Error for ParseError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_display() {
        assert_eq!(ErrorKind::DivByZero.to_string(), "#DIV/0!");
        assert_eq!(ErrorKind::Name.to_string(), "#NAME?");
        assert_eq!(ErrorKind::Num.to_string(), "#NUM!");
        assert_eq!(ErrorKind::NA.to_string(), "#N/A");
    }

    #[test]
    fn parse_error_display() {
        let e = ParseError { message: "Unmatched parenthesis".into(), position: 14 };
        assert_eq!(e.to_string(), "Unmatched parenthesis at position 14");
    }
}
