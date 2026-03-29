use crate::middleware::auth::service::AuthService;
use axum::{
    async_trait,
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

/// Authenticated player extracted from JWT token.
/// Use this as an extractor in protected route handlers.
///
/// # Example
/// ```rust
/// async fn my_handler(auth: AuthPlayer) -> impl IntoResponse {
///     format!("Hello, {}", auth.wallet_address)
/// }
/// ```
#[derive(Debug, Clone)]
pub struct AuthPlayer {
    /// Player's MongoDB ObjectId (hex string)
    pub id: String,

    /// Wallet address (lowercase)
    pub wallet_address: String,

    /// Display name at token creation
    pub name: String,
}

/// Structured JSON rejection for auth failures
pub struct AuthRejection {
    status: StatusCode,
    message: String,
}

impl IntoResponse for AuthRejection {
    fn into_response(self) -> Response {
        (
            self.status,
            Json(json!({
                "ok": false,
                "message": self.message
            })),
        )
            .into_response()
    }
}

#[async_trait]
impl<S> FromRequestParts<S> for AuthPlayer
where
    S: Send + Sync,
{
    type Rejection = AuthRejection;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        // Extract Authorization header
        let auth_header = parts
            .headers
            .get("Authorization")
            .and_then(|v| v.to_str().ok())
            .ok_or(AuthRejection {
                status: StatusCode::UNAUTHORIZED,
                message: "Missing Authorization header".to_string(),
            })?;

        // Expect "Bearer <token>"
        let token = auth_header.strip_prefix("Bearer ").ok_or(AuthRejection {
            status: StatusCode::UNAUTHORIZED,
            message: "Invalid Authorization header format. Expected: Bearer <token>".to_string(),
        })?;

        // Verify token
        let claims = AuthService::verify_token(token).map_err(|e| AuthRejection {
            status: StatusCode::UNAUTHORIZED,
            message: e,
        })?;

        // Check expiration (jsonwebtoken does this, but explicit check for clarity)
        let now = chrono::Utc::now().timestamp();
        if claims.exp < now {
            return Err(AuthRejection {
                status: StatusCode::UNAUTHORIZED,
                message: "Token expired".to_string(),
            });
        }

        Ok(AuthPlayer {
            id: claims.player_id,
            wallet_address: claims.walletAddress,
            name: claims.name,
        })
    }
}
