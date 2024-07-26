#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("database request failed: {0}")]
    TokioRusqliteFailed(tokio_rusqlite::Error),
    #[error("database request failed: {0}")]
    RusqliteFailed(rusqlite::Error),
    #[error("error reading file: {0}")]
    IOFailed(std::io::Error),
    #[error("error reading from jsonl file: {0}")]
    SerdeFailed(serde_json::Error),
    #[error("error getting value from json value {0} at line {1}")]
    GetValueFailed(serde_json::Value, usize),
    #[error("error converting json value {0} at line {1}")]
    ValueConversionFailed(serde_json::Value, usize),
    #[error("error when creating regex: {0}")]
    RegexFailed(regex::Error),
    #[error("json array is empty at line {0}")]
    EmptyJSONArray(usize),
}

impl From<tokio_rusqlite::Error> for Error {
    fn from(error: tokio_rusqlite::Error) -> Self {
        match error {
            tokio_rusqlite::Error::Other(error) => {
                if error.downcast_ref::<Self>().is_some() {
                    *error.downcast().unwrap()
                } else {
                    Self::TokioRusqliteFailed(tokio_rusqlite::Error::Other(error))
                }
            }
            _ => Self::TokioRusqliteFailed(error),
        }
    }
}

impl From<Error> for tokio_rusqlite::Error {
    fn from(error: Error) -> Self {
        Self::Other(Box::new(error))
    }
}

impl From<rusqlite::Error> for Error {
    fn from(error: rusqlite::Error) -> Self {
        Self::RusqliteFailed(error)
    }
}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        Self::IOFailed(error)
    }
}

impl From<serde_json::Error> for Error {
    fn from(error: serde_json::Error) -> Self {
        Self::SerdeFailed(error)
    }
}

impl From<regex::Error> for Error {
    fn from(error: regex::Error) -> Self {
        Self::RegexFailed(error)
    }
}
