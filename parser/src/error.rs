use std::fmt;
use human_lexer::Span;

#[derive(Debug, Clone)]
pub struct ParseError {
    pub span: Span,
    pub message: String,
}

impl ParseError {
    pub fn new(span: Span, message: impl Into<String>) -> Self {
        ParseError { span, message: message.into() }
    }

    pub fn display_with_file(&self, filename: &str) -> String {
        format!("{}:{}:{}: error: {}", filename, self.span.line, self.span.col, self.message)
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}: error: {}", self.span.line, self.span.col, self.message)
    }
}
