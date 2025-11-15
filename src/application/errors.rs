use anyhow::Error as AnyhowError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ErrorResponse {
    #[error("validation error: {0}")]
    Validation(String),

    #[error("not found: {0}")]
    NotFound(String),

    #[error("internal error: {0}")]
    InternalServerErr(String),
}

impl From<AnyhowError> for ErrorResponse {
    fn from(err: AnyhowError) -> Self {
        ErrorResponse::InternalServerErr(err.to_string())
    }
}
