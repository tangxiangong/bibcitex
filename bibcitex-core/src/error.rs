/// Specific Errors
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// IO Error
    #[error("{0}")]
    IOError(#[from] std::io::Error),
    /// BibTeX Parse Error
    #[error("{0}")]
    BibParseError(String),
    /// JSON Serialize Error
    #[error("{0}")]
    JSONError(#[from] serde_json::Error),
    /// Bibliography Not Found Error
    #[error("Bibliography {0} not found")]
    BibNotFound(String),
    /// Missing Field Error
    #[error("{0}")]
    MissingFiled(String),
    /// Field Type Error
    #[error("{0}")]
    FieldType(String),
    #[error("MSC Error: {0}")]
    MSCError(String),
}

impl From<biblatex::ParseError> for Error {
    fn from(value: biblatex::ParseError) -> Self {
        Error::BibParseError(value.to_string())
    }
}

impl From<biblatex::RetrievalError> for Error {
    fn from(value: biblatex::RetrievalError) -> Self {
        match value {
            biblatex::RetrievalError::Missing(key) => Error::BibNotFound(key),
            biblatex::RetrievalError::TypeError(key) => Error::FieldType(key.to_string()),
        }
    }
}

impl From<csv::Error> for Error {
    fn from(value: csv::Error) -> Self {
        Error::MSCError(value.to_string())
    }
}

/// Specific `Result` type
pub type Result<T> = std::result::Result<T, Error>;
