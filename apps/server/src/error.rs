use poem::{error::ResponseError, http::StatusCode};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("db err")]
    Sqlx(#[from] sqlx::Error),
    #[error("unknown forge: {0}")]
    UnknownForge(uuid::Uuid),
    #[error("url parse err")]
    UrlParse(#[from] oauth2::url::ParseError),
    #[error("jwt error")]
    Jwt(#[from] jsonwebtoken::errors::Error),
    #[error("internal error: {0}")]
    InternalError(#[from] anyhow::Error),
    #[error("serde json error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("invalid token response")]
    InvalidTokenResponse,
}

pub type Result<T, E = AppError> = std::result::Result<T, E>;

impl ResponseError for AppError {
    fn status(&self) -> poem::http::StatusCode {
        match self {
            AppError::Sqlx(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::UnknownForge(_) => StatusCode::NOT_FOUND,
            AppError::UrlParse(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::InternalError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::Jwt(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::InvalidTokenResponse => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::Json(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}
