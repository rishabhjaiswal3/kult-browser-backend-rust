use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;

/// Success response wrapper: {"ok": true, "data": T}
///
/// Wraps any serializable data in a standard success format.
#[derive(Serialize)]
pub struct ApiResponse<T: Serialize> {
    ok: bool,
    data: T,
}

impl<T: Serialize> ApiResponse<T> {
    /// Create a success response
    pub fn success(data: T) -> Self {
        Self { ok: true, data }
    }
}

impl<T: Serialize> IntoResponse for ApiResponse<T> {
    fn into_response(self) -> Response {
        (StatusCode::OK, Json(self)).into_response()
    }
}

/// Type alias for handler return type
///
/// Use this as the return type for your handlers:
/// ```rust
/// pub async fn my_handler() -> ApiResult<MyData> {
///     let data = service.get_data()?;
///     Ok(ApiResponse::success(data))
/// }
/// ```
pub type ApiResult<T> = Result<ApiResponse<T>, super::AppError>;
