use std::fmt;

/// Result type alias for Frel compiler operations
pub type Result<T> = std::result::Result<T, Error>;

/// Compiler errors
#[derive(Debug)]
pub enum Error {
    /// Parse error
    ParseError(String),

    /// Semantic analysis error
    SemanticError {
        message: String,
        location: Option<Location>,
    },

    /// Type checking error
    TypeError {
        message: String,
        location: Option<Location>,
    },

    /// IO error
    IoError(std::io::Error),
}

/// Source code location
#[derive(Debug, Clone)]
pub struct Location {
    pub file: String,
    pub line: usize,
    pub column: usize,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::ParseError(msg) => write!(f, "Parse error: {}", msg),
            Error::SemanticError { message, location } => {
                if let Some(loc) = location {
                    write!(f, "Semantic error at {}:{}:{}: {}", loc.file, loc.line, loc.column, message)
                } else {
                    write!(f, "Semantic error: {}", message)
                }
            }
            Error::TypeError { message, location } => {
                if let Some(loc) = location {
                    write!(f, "Type error at {}:{}:{}: {}", loc.file, loc.line, loc.column, message)
                } else {
                    write!(f, "Type error: {}", message)
                }
            }
            Error::IoError(err) => write!(f, "IO error: {}", err),
        }
    }
}

impl std::error::Error for Error {}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::IoError(err)
    }
}
