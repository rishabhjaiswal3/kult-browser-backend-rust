// src/moments/social_media/controller/social_media_controller.rs

use axum::extract::{Path, State};
use axum::response::{IntoResponse, Response};
use axum::Json;

use crate::handler::{ApiResponse, AppError};
use crate::middleware::AuthPlayer;
use crate::moments::social_media::dto::{RequeueSharedPostResponse, SubmitPostRequest};
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
        Err(PostServiceError::UnsupportedPlatform) => {
            AppError::BadRequest("This platform is not supported yet".to_string()).into_response()
        }
        Err(PostServiceError::InvalidPlatformUrl(message)) => {
            AppError::BadRequest(message).into_response()
        }
        Err(PostServiceError::MomentNotFound) => {
            AppError::NotFound("Moment not found".to_string()).into_response()
        }
        Err(PostServiceError::ForbiddenMomentAccess) => {
            AppError::Forbidden("You can only submit posts for your own moments".to_string())
                .into_response()
        }
        Err(PostServiceError::QueueUnavailable) => {
            AppError::Internal("Validation queue unavailable. Please try again later".to_string())
                .into_response()
        }
        Err(PostServiceError::QueuePushFailed(message)) => {
            AppError::Internal(message).into_response()
        }
        Err(e) => {
            tracing::error!(error = %e, "Failed to submit shared post");
            AppError::Internal(e.to_string()).into_response()
        }
    }
}

#[utoipa::path(
    get,
    path = "/api/moments/social-media/my-posts",
    security(
        ("bearer_auth" = [])
    ),
    responses(
        (status = 200, description = "Authenticated player's submitted posts", body = crate::openapi::SharedPostListApiResponse),
        (status = 401, description = "Missing or invalid bearer token", body = crate::openapi::ErrorResponse),
        (status = 500, description = "Internal server error", body = crate::openapi::ErrorResponse)
    ),
    tag = "Social Media"
)]
pub async fn list_my_posts(State(state): State<SocialMediaState>, auth: AuthPlayer) -> Response {
    match state
        .post_service
        .list_shared_posts(auth.wallet_address)
        .await
    {
        Ok(data) => ApiResponse::success(data).into_response(),
        Err(e) => {
            tracing::error!(error = %e, "Failed to list shared posts");
            AppError::Internal(e.to_string()).into_response()
        }
    }
}

#[utoipa::path(
    post,
    path = "/api/moments/social-media/posts/{post_id}/requeue",
    security(
        ("bearer_auth" = [])
    ),
    params(
        ("post_id" = String, Path, description = "Shared post document id")
    ),
    responses(
        (status = 200, description = "Shared post requeued", body = crate::openapi::RequeueSharedPostApiResponse),
        (status = 400, description = "Invalid post id, URL, or unsupported platform", body = crate::openapi::ErrorResponse),
        (status = 403, description = "Shared post is not owned by the authenticated player", body = crate::openapi::ErrorResponse),
        (status = 404, description = "Shared post not found", body = crate::openapi::ErrorResponse),
        (status = 401, description = "Missing or invalid bearer token", body = crate::openapi::ErrorResponse),
        (status = 500, description = "Internal server error", body = crate::openapi::ErrorResponse)
    ),
    tag = "Social Media"
)]
pub async fn requeue_post(
    State(state): State<SocialMediaState>,
    auth: AuthPlayer,
    Path(post_id): Path<String>,
) -> Response {
    match state
        .post_service
        .requeue_shared_post(post_id, auth.wallet_address)
        .await
    {
        Ok(post_id) => ApiResponse::success(RequeueSharedPostResponse {
            post_id: post_id.to_hex(),
            message: "Post requeued for validation".to_string(),
        })
        .into_response(),
        Err(PostServiceError::UnsupportedPlatform) => {
            AppError::BadRequest("This platform is not supported yet".to_string()).into_response()
        }
        Err(PostServiceError::InvalidPlatformUrl(message)) => {
            AppError::BadRequest(message).into_response()
        }
        Err(PostServiceError::ForbiddenMomentAccess) => {
            AppError::Forbidden("You can only access your own submitted posts".to_string())
                .into_response()
        }
        Err(PostServiceError::PostNotFound) => {
            AppError::NotFound("Shared post not found".to_string()).into_response()
        }
        Err(PostServiceError::QueueUnavailable) => {
            AppError::Internal("Validation queue unavailable. Please try again later".to_string())
                .into_response()
        }
        Err(PostServiceError::QueuePushFailed(message)) => {
            AppError::Internal(message).into_response()
        }
        Err(e) => {
            tracing::error!(error = %e, "Failed to requeue shared post");
            AppError::Internal(e.to_string()).into_response()
        }
    }
}
