use thiserror::Error;

#[derive(Debug, Error)]
pub enum RvaError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("validation error: {0}")]
    Validation(String),
    #[error("render error: {0}")]
    Render(String),
}

pub type Result<T> = std::result::Result<T, RvaError>;
