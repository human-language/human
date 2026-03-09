use std::fmt;
use std::path::{Path, PathBuf};

pub fn short_name(path: &Path) -> String {
    path.file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| path.display().to_string())
}
use human_lexer::{LexError, Span};
use human_parser::ParseError;

#[derive(Debug, Clone)]
pub struct ResolveError {
    pub file: PathBuf,
    pub line: Option<u32>,
    pub col: Option<u16>,
    pub message: String,
}

impl ResolveError {
    pub fn at_span(file: impl Into<PathBuf>, span: Span, message: impl Into<String>) -> Self {
        ResolveError {
            file: file.into(),
            line: Some(span.line),
            col: Some(span.col),
            message: message.into(),
        }
    }

    pub fn at_file(file: impl Into<PathBuf>, message: impl Into<String>) -> Self {
        ResolveError {
            file: file.into(),
            line: None,
            col: None,
            message: message.into(),
        }
    }

    pub fn from_lex_error(file: &Path, err: &LexError) -> Self {
        ResolveError {
            file: file.to_path_buf(),
            line: Some(err.line),
            col: Some(err.col),
            message: err.message.clone(),
        }
    }

    pub fn from_parse_error(file: &Path, err: &ParseError) -> Self {
        ResolveError {
            file: file.to_path_buf(),
            line: Some(err.span.line),
            col: Some(err.span.col),
            message: err.message.clone(),
        }
    }
}

impl fmt::Display for ResolveError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match (self.line, self.col) {
            (Some(line), Some(col)) => {
                write!(f, "{}:{}:{}: error: {}", self.file.display(), line, col, self.message)
            }
            _ => {
                write!(f, "{}: error: {}", self.file.display(), self.message)
            }
        }
    }
}
