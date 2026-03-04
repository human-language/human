use std::fmt;

#[derive(Debug, Clone)]
pub struct LexError {
    pub line: u32,
    pub col: u16,
    pub message: String,
}

impl LexError {
    pub fn new(line: u32, col: u16, message: impl Into<String>) -> Self {
        LexError { line, col, message: message.into() }
    }

    pub fn display_with_file(&self, filename: &str) -> String {
        format!("{}:{}:{}: error: {}", filename, self.line, self.col, self.message)
    }
}

impl fmt::Display for LexError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}: error: {}", self.line, self.col, self.message)
    }
}
