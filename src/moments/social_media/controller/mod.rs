// src/moments/social_media/controller/social_media_controller.rs

use axum::extract::State;
use axum::response::{IntoResponse, Response};
use axum::Json;

use crate::handler::{ApiResponse, AppError};
use crate::middleware::AuthPlayer;
use crate::moments::social_media::dto::SubmitPostRequest;
use crate::moments::social_media::service::post_service::{PostService, PostServiceError};

/// Shared state for social media endpoints
#[derive(Clone)]
pub struct SocialMediaState {
    pub post_service: PostService,
}

/// POST /api/moments/social-media/submit-url
/// Submit a social media post URL for validation (auth required)
#[utoipa::path(
    post,
    path = "/api/moments/social-media/submit-url",
    security(
        ("bearer_auth" = [])
    ),
    request_body = SubmitPostRequest,
    responses(
        (status = 200, description = "Queued social media post validation", body = crate::openapi::SubmitSharedPostApiResponse),
        (status = 400, description = "Invalid or duplicate post submission", body = crate::openapi::ErrorResponse),
        (status = 403, description = "Moment is not owned by the authenticated player", body = crate::openapi::ErrorResponse),
        (status = 404, description = "Moment not found", body = crate::openapi::ErrorResponse),
        (status = 401, description = "Missing or invalid bearer token", body = crate::openapi::ErrorResponse),
        (status = 500, description = "Internal server error", body = crate::openapi::ErrorResponse)
    ),
    tag = "Social Media"
)]
pub async fn submit_post(
    State(state): State<SocialMediaState>,
    auth: AuthPlayer,
    payload: Result<Json<SubmitPostRequest>, axum::extract::rejection::JsonRejection>,
) -> Response {
    let Json(request) = match payload {
        Ok(p) => p,
        Err(rejection) => return AppError::BadRequest(rejection.body_text()).into_response(),
    };

    match state
        .post_service
        .submit_shared_post(
            request.moment_id,
            auth.wallet_address,
            request.platform,
            request.post_id,
            request.url,
        )
        .await
    {
        Ok(inserted_id) => ApiResponse::success(serde_json::json!({
            "postId": inserted_id.to_hex(),
            "message": "Post submitted successfully. Validation will be processed shortly."
        }))
        .into_response(),
        Err(PostServiceError::DuplicatePost) => {
            AppError::BadRequest("This post has already been submitted".to_string()).into_response()
        }
        Err(PostServiceError::MomentNotFound) => {
            AppError::NotFound("Moment not found".to_string()).into_response()
        }
        Err(PostServiceError::ForbiddenMomentAccess) => AppError::Forbidden(
            "You can only submit posts for your own moments".to_string(),
        )
        .into_response(),
        Err(e) => {
            tracing::error!(error = %e, "Failed to submit shared post");
            AppError::Internal(e.to_string()).into_response()
        }
    }
}
