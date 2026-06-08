use poem::{error::ResponseError, http::StatusCode};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("db err")]
    Sqlx(#[from] sqlx::Error),
}

pub type Result<T, E = AppError> = std::result::Result<T, E>;

impl ResponseError for AppError {
    fn status(&self) -> poem::http::StatusCode {
        match self {
            AppError::Sqlx(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}
