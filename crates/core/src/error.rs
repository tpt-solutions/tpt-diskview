use std::io;

#[derive(Debug, thiserror::Error)]
pub enum ScanError {
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),
    #[error("Path not found: {0}")]
    PathNotFound(String),
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
    #[error("Scan was cancelled")]
    Cancelled,
    #[error("{0}")]
    Other(String),
}

impl From<ScanError> for String {
    fn from(e: ScanError) -> String {
        e.to_string()
    }
}
