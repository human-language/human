use std::fmt;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct CompileError {
    pub file: PathBuf,
    pub message: String,
}

impl CompileError {
    pub fn new(file: impl Into<PathBuf>, message: impl Into<String>) -> Self {
        Self {
            file: file.into(),
            message: message.into(),
        }
    }
}

impl fmt::Display for CompileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: error: {}", self.file.display(), self.message)
    }
}

impl std::error::Error for CompileError {}
