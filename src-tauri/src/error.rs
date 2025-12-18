use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("BibTeX parse error: {0}")]
    BibParse(String),

    #[error("Config error: {0}")]
    Config(String),

    #[error("File not found: {0}")]
    FileNotFound(String),

    #[error("Clipboard error: {0}")]
    Clipboard(String),

    #[error("Tauri error: {0}")]
    Tauri(String),

    #[error("Core error: {0}")]
    Core(String),
}

impl From<bibcitex_core::Error> for Error {
    fn from(e: bibcitex_core::Error) -> Self {
        Error::Core(e.to_string())
    }
}

impl From<biblatex::ParseError> for Error {
    fn from(e: biblatex::ParseError) -> Self {
        Error::BibParse(e.to_string())
    }
}

impl From<config::ConfigError> for Error {
    fn from(e: config::ConfigError) -> Self {
        Error::Config(e.to_string())
    }
}

impl From<arboard::Error> for Error {
    fn from(e: arboard::Error) -> Self {
        Error::Clipboard(e.to_string())
    }
}

impl serde::Serialize for Error {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        serializer.serialize_str(self.to_string().as_ref())
    }
}

pub type Result<T> = std::result::Result<T, Error>;
