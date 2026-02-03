use crate::middleware::auth::service::AuthService;
use axum::{
    async_trait,
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
};

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

#[async_trait]
impl<S> FromRequestParts<S> for AuthPlayer
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, String);

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        // Extract Authorization header
        let auth_header = parts
            .headers
            .get("Authorization")
            .and_then(|v| v.to_str().ok())
            .ok_or((
                StatusCode::UNAUTHORIZED,
                "Missing Authorization header".to_string(),
            ))?;

        // Expect "Bearer <token>"
        let token = auth_header
            .strip_prefix("Bearer ")
            .ok_or((
                StatusCode::UNAUTHORIZED,
                "Invalid Authorization header format. Expected: Bearer <token>".to_string(),
            ))?;

        // Verify token
        let claims = AuthService::verify_token(token).map_err(|e| {
            (StatusCode::UNAUTHORIZED, e)
        })?;

        // Check expiration (jsonwebtoken does this, but explicit check for clarity)
        let now = chrono::Utc::now().timestamp();
        if claims.exp < now {
            return Err((StatusCode::UNAUTHORIZED, "Token expired".to_string()));
        }

        Ok(AuthPlayer {
            id: claims.player_id,
            wallet_address: claims.sub,
            name: claims.name,
        })
    }
}