use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;

#[derive(Debug)]
pub enum AppError {
    BadRequest(String),
    Unauthorized(String),
    Forbidden(String),
    Conflict(String),
    Internal(String),
}

#[derive(Serialize)]
struct ErrorBody {
    error: String,
}

impl AppError {
    fn status(&self) -> StatusCode {
        match self {
            Self::BadRequest(_) => StatusCode::BAD_REQUEST,
            Self::Unauthorized(_) => StatusCode::UNAUTHORIZED,
            Self::Forbidden(_) => StatusCode::FORBIDDEN,
            Self::Conflict(_) => StatusCode::CONFLICT,
            Self::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn message(&self) -> &str {
        match self {
            Self::BadRequest(m)
            | Self::Unauthorized(m)
            | Self::Forbidden(m)
            | Self::Conflict(m)
            | Self::Internal(m) => m,
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        if matches!(self, Self::Internal(_)) {
            tracing::error!("internal error: {}", self.message());
        }
        (
            self.status(),
            Json(ErrorBody {
                error: self.message().to_string(),
            }),
        )
            .into_response()
    }
}

impl From<sea_orm::DbErr> for AppError {
    fn from(err: sea_orm::DbErr) -> Self {
        tracing::error!("database error: {err}");
        Self::Internal("database error".into())
    }
}

impl From<argon2::password_hash::Error> for AppError {
    fn from(err: argon2::password_hash::Error) -> Self {
        tracing::error!("password hash error: {err}");
        Self::Internal("password processing error".into())
    }
}
