use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

/// Application error types with associated HTTP status codes.
#[derive(Debug)]
pub enum AppError {
    BadRequest(String),
    Unauthorized(String),
    Forbidden(String),
    NotFound(String),
    Conflict(String),
    Internal(String),
}

impl AppError {
    /// Get HTTP status code and message for this error
    fn status_and_message(&self) -> (StatusCode, &str) {
        match self {
            Self::BadRequest(m) => (StatusCode::BAD_REQUEST, m),
            Self::Unauthorized(m) => (StatusCode::UNAUTHORIZED, m),
            Self::Forbidden(m) => (StatusCode::FORBIDDEN, m),
            Self::NotFound(m) => (StatusCode::NOT_FOUND, m),
            Self::Conflict(m) => (StatusCode::CONFLICT, m),
            Self::Internal(m) => (StatusCode::INTERNAL_SERVER_ERROR, m),
        }
    }
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.status_and_message().1)
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = self.status_and_message();

        if matches!(self, AppError::Internal(_)) {
            tracing::error!(error = %message, "Internal server error");
        }

        (status, Json(json!({ "ok": false, "message": message }))).into_response()
    }
}

// Conversions for ergonomic error handling
impl From<String> for AppError {
    fn from(msg: String) -> Self {
        Self::Internal(msg)
    }
}

impl From<&str> for AppError {
    fn from(msg: &str) -> Self {
        Self::Internal(msg.into())
    }
}

impl From<mongodb::error::Error> for AppError {
    fn from(e: mongodb::error::Error) -> Self {
        Self::Internal(format!("Database error: {}", e))
    }
}

// Handle axum's built-in rejections for Query params
impl From<axum::extract::rejection::QueryRejection> for AppError {
    fn from(rejection: axum::extract::rejection::QueryRejection) -> Self {
        Self::BadRequest(format!("Invalid query parameters: {}", rejection))
    }
}

// Handle axum's built-in rejections for JSON body
impl From<axum::extract::rejection::JsonRejection> for AppError {
    fn from(rejection: axum::extract::rejection::JsonRejection) -> Self {
        Self::BadRequest(format!("Invalid JSON body: {}", rejection))
    }
}
